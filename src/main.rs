mod arena;
mod packet;

use arena::Arena;
use dashmap::DashMap;
use futures_channel::mpsc::{self, UnboundedSender};
use futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt};
use lockfree_cuckoohash::LockFreeCuckooHash;
use packet::Packet;
use std::{env, net::SocketAddr, str::FromStr, sync::Arc};
use tokio::{
    net::{TcpListener, TcpStream},
    time,
};
use tungstenite::protocol::Message;

// 客户端，也就是大厅服务器
type Tx = UnboundedSender<Message>;
type Peers = Arc<LockFreeCuckooHash<SocketAddr, Tx>>;
// 哪个玩家是哪个大厅服务器记录的
type Senders = Arc<dashmap::DashMap<String, SocketAddr>>;

// 所有匹配池的列表。u64是这个匹配池一局的玩家数，超过这个数就匹配成功
type Arenas = Arc<dashmap::DashMap<String, (u64, Arena<String>)>>;

async fn handle_connection(
    peer_map: Peers,
    arenas: Arenas,
    senders: Senders,
    raw_stream: TcpStream,
    addr: SocketAddr,
) {
    println!("[客户端]({addr}) 的新TCP连接已建立，正在尝试连接为WebSocket……");

    let try_ws_stream = tokio_tungstenite::accept_async(raw_stream).await;

    let ws_stream = match try_ws_stream {
        Ok(ws_stream) => ws_stream,
        Err(e) => {
            println!("[客户端]({addr}) 连接WebSocket流时发生错误，连接即将终止！错误：{e}");
            return;
        }
    };

    println!("[客户端]({addr}) 通向地址 {addr} 的WebSocket连接已建立。");

    // 把写部分存到客户端表里面
    let (tx, rx) = mpsc::unbounded();
    peer_map.insert(addr, tx.clone());

    // 反馈定时器
    let (mut dur_tx, dur_rx) = mpsc::channel(1);
    let state_feedback = state_feedback_timer(tx.clone(), Arc::clone(&arenas), addr, dur_rx);

    // websocket流和处理函数
    let (outgoing, incoming) = ws_stream.split();

    let process_incoming = incoming.try_for_each(|msg| {
        let text = msg.to_text().unwrap();
        let packet = Packet::from_str(text);
        match packet {
            Ok(Packet::AddArena { arena, num_players }) => {
                if num_players == 0 {
                    println!("[匹配池]({addr}) 尝试注册匹配池 {arena}，但匹配池的每局玩家数为0，创建失败！");
                } else {
                    let entry = arenas.entry(arena.clone());
                    entry.or_insert_with(|| (num_players, Arena::new()));
                    println!("[匹配池]({addr}) 已注册匹配池 {arena}，达到 {num_players} 位玩家时，此匹配池将返回匹配结果。");
                }
            },
            Ok(Packet::RemoveArena(arena)) => {
                let removed = arenas.remove(&arena);
                if removed.is_some() {
                    println!("[匹配池]({addr}) 已删除匹配池 {arena}。");
                } else {
                    println!("[匹配池]({addr}) 正在删除匹配池 {arena}，此匹配池已不存在。")
                }
            },
            Ok(Packet::AddPlayer { arena, player, rank, length, init_rank_diff }) => {
                let try_arena = arenas.get(&arena);
                if let Some(arena_) = try_arena {
                    let rank_min = rank - init_rank_diff;
                    let rank_max = rank + init_rank_diff;
                    arena_.1.insert(player.clone(), length as usize, rank_min as usize, rank_max as usize);
                    senders.insert(player.clone(), addr);
                    println!("[玩家匹配]({addr}) 成功向匹配池 {arena} 添加玩家 {player}（分数为 {rank}，初始区间为 {rank_min}至{rank_max}，数量为 {length}）。");
                } else {
                    println!("[玩家匹配]({addr}) 正在向 {arena} 添加玩家 {player}（分数为 {rank}，数量为 {length}，区间差值为{init_rank_diff}），但此匹配池不存在。");
                }
            },
            Ok(Packet::RemovePlayer { arena, player }) => {
                let try_arena = arenas.get(&arena);
                if let Some(arena_) = try_arena {
                    arena_.1.remove(&player);
                    senders.remove(&player);
                    println!("[玩家匹配]({addr}) 成功从匹配池 {arena} 删除玩家 {player}。");
                } else {
                    println!("[玩家匹配]({addr}) 正在向 {arena} 删除玩家 {player}，但此匹配池不存在。");
                }
            },
            Ok(Packet::GetOrSubscribeState { period }) => {
                let period = if period == 0 {
                    None
                } else {
                    Some(time::Duration::from_secs(period))
                };
                match dur_tx.try_send(period) {
                    Ok(_) => if let Some(duration) = period {
                        println!("[订阅]({addr}) 修改订阅周期为 {} 秒", duration.as_secs())
                    } else {
                        println!("[订阅]({addr}) 已取消订阅")
                    },
                    Err(e) => println!("内部错误：{e}"),
                }
            },
            Err(e) => {
                println!("[错误]({addr}) 包格式错误：{}", e.0);
                let packet = Packet::FormatError { error: e.0.to_string() };
                let string = packet.to_string();
                let try_send = tx.unbounded_send(Message::Text(string));
                if let Err(e) = try_send {
                    println!("[错误]({addr}) 内部错误：{e}");
                }
            },
            _ => println!("[错误]({addr}) 内部错误：客户端发送了非法包格式！"),
        }

        future::ok(())
    });

    let receive_from_others = rx.map(Ok).forward(outgoing);

    pin_mut!(process_incoming, receive_from_others);
    tokio::spawn(state_feedback);
    future::select(process_incoming, receive_from_others).await;

    println!("[客户端]({}) 已经断开WebSocket连接。", &addr);

    // 关闭排位反馈定时器
    dur_tx.close_channel();

    // 移除此连接的玩家
    let mut players = Vec::new();
    for sender_ref in senders.iter() {
        if sender_ref.value() == &addr {
            let player = sender_ref.key();
            players.push(player.clone());
        }
    }
    for player in players.iter() {
        for arena_ref in arenas.iter() {
            arena_ref.value().1.remove(player);
        }
    }
    senders.retain(|_player, addr_for_this_player| &addr != addr_for_this_player);
    println!(
        "[客户端]({}) 已移除该客户端注册的玩家，列表是：{:?}。",
        addr, players
    );

    peer_map.remove(&addr);
    println!("[客户端]({}) 已经从排位匹配服务器解除注册，再见！", addr);
}

async fn state_feedback_timer(
    peer: Tx,
    arenas: Arenas,
    addr: SocketAddr,
    mut period: mpsc::Receiver<Option<time::Duration>>,
) {
    println!("地址{addr}的排位状态反馈服务开始工作！");
    let mut last_duration = None;
    loop {
        match period.try_next() {
            Ok(Some(duration)) => last_duration = duration,
            // Ok(None) 关闭管道来退出定时器
            Ok(None) => break,
            // Err(_) 管道中暂未收到数据
            Err(_) => {}
        }
        if let Some(duration) = last_duration {
            let player_info = DashMap::new();
            for arena_ref in arenas.iter() {
                let (_num_players, arena) = arena_ref.value();
                for state in arena.get_player_states().iter() {
                    let player = state.key();
                    let current_count = state.value();
                    player_info.insert(
                        player.to_string(),
                        (arena_ref.key().to_string(), *current_count),
                    );
                }
            }
            println!("向{}发送排位反馈状态:{:?}", addr, player_info);
            let packet = Packet::ConnectionState { player_info };
            let string = packet.to_string();
            let try_send = peer.unbounded_send(Message::Text(string));
            if let Err(e) = try_send {
                println!("内部错误：{e}");
            }
            time::sleep(duration).await;
        } else {
            // 传入None定时器休眠
            tokio::task::yield_now().await;
        }
    }
    println!("地址{addr}的排位状态反馈服务停止工作！");
}

async fn rank_timer(peers: Peers, arenas: Arenas, senders: Senders, http_client: reqwest::Client) {
    let mut interval = time::interval(time::Duration::from_secs(1));
    println!("排位定时器开始工作！");
    loop {
        for arena_ref in arenas.iter() {
            let (num_players, arena) = arena_ref.value();
            let matched = arena.rank_match();
            let num_matched: usize = matched.iter().map(|(_name, length)| length).sum();
            if num_matched >= *num_players as usize {
                // 匹配成功
                println!(
                    "[匹配池] {} 成功匹配了 {} 位玩家：{:?}",
                    arena_ref.key(),
                    matched.len(),
                    matched
                );
                let collected: DashMap<SocketAddr, Vec<(String, u64)>> = DashMap::new();
                for (player, length) in matched.clone() {
                    let try_addr = senders.get(&player);
                    if let Some(addr) = try_addr {
                        collected
                            .entry(addr.clone())
                            .and_modify(|v| v.push((player.clone(), length as u64)))
                            .or_insert_with(|| vec![(player.clone(), length as u64)]);
                    }
                }
                tokio::spawn(request_http_and_send_id(
                    Arc::clone(&peers),
                    arena_ref.key().clone(),
                    collected,
                    http_client.clone(),
                ));
                let guard = lockfree_cuckoohash::pin();
                for (player, _length) in &matched {
                    arena.remove(player);
                }
                drop(guard);
                for (player, _length) in &matched {
                    senders.remove(player);
                }
            }
            arena.rank_update();
        }
        interval.tick().await;
    }
}

#[derive(serde::Serialize)]
struct CreateStageRequest {
    game: String,
    matching: String,
}

#[derive(serde::Deserialize)]
#[serde(untagged)]
enum CreateStageResponse {
    Success { request_id: u64 },
    Error { error_id: u64, error_msg: String },
}

async fn request_http_and_send_id(
    peers: Peers,
    arena: String,
    collected: DashMap<SocketAddr, Vec<(String, u64)>>,
    http_client: reqwest::Client,
) {
    let response = http_client
        .post("http://example.com")
        .json(&CreateStageRequest {
            game: arena.clone(),
            matching: format!("Rank#{}", rand::random::<u32>()),
        })
        .send()
        .await;
    let stage_request_id = match response {
        Ok(a) => match a.json::<CreateStageResponse>().await {
            Ok(CreateStageResponse::Success { request_id }) => request_id,
            Ok(CreateStageResponse::Error {
                error_id,
                error_msg,
            }) => {
                // FIXME: 这里的代码还能写得更简单
                println!("[匹配池] 中心服务器返回了错误！错误代码{error_id}，错误信息{error_msg}");
                for item_collected in collected {
                    let (addr, players) = item_collected;
                    let packet = Packet::MatchFailure {
                        arena: arena.clone(),
                        error_id,
                        error_msg: error_msg.clone(),
                        players,
                    };
                    let string = packet.to_string();
                    let guard = lockfree_cuckoohash::pin();
                    if let Some(peer) = peers.get(&addr, &guard) {
                        let try_send = peer.unbounded_send(Message::Text(string));
                        if let Err(e) = try_send {
                            println!("[匹配池] 内部错误：{e}");
                        }
                    }
                    drop(guard);
                }
                return;
            }
            Err(e) => {
                println!("[匹配池] 中心服务器返回的新增房间回复不是json格式！{e}");
                for item_collected in collected {
                    let (addr, players) = item_collected;
                    let packet = Packet::MatchFailure {
                        arena: arena.clone(),
                        error_id: 9000,
                        error_msg: format!("中心服务器返回的新增房间回复不是json格式：{e}"),
                        players,
                    };
                    let string = packet.to_string();
                    let guard = lockfree_cuckoohash::pin();
                    if let Some(peer) = peers.get(&addr, &guard) {
                        let try_send = peer.unbounded_send(Message::Text(string));
                        if let Err(e) = try_send {
                            println!("[匹配池] 内部错误：{e}");
                        }
                    }
                    drop(guard);
                }
                return;
            }
        },
        Err(e) => {
            println!("[匹配池] 内部错误，无法连接到中心服务器！{e}");
            for item_collected in collected {
                let (addr, players) = item_collected;
                let packet = Packet::MatchFailure {
                    arena: arena.clone(),
                    error_id: 9001,
                    error_msg: format!("无法连接到中心服务器：{e}"),
                    players,
                };
                let string = packet.to_string();
                let guard = lockfree_cuckoohash::pin();
                if let Some(peer) = peers.get(&addr, &guard) {
                    let try_send = peer.unbounded_send(Message::Text(string));
                    if let Err(e) = try_send {
                        println!("[匹配池] 内部错误：{e}");
                    }
                }
                drop(guard);
            }
            return;
        }
    };
    for item_collected in collected {
        let (addr, players) = item_collected;
        println!("[匹配池] 发送给地址 {addr} 的玩家列表：{:?}", players);
        let packet = Packet::MatchSuccess {
            arena: arena.clone(),
            stage_request_id,
            players,
        };
        let string = packet.to_string();
        let guard = lockfree_cuckoohash::pin();
        if let Some(peer) = peers.get(&addr, &guard) {
            let try_send = peer.unbounded_send(Message::Text(string));
            if let Err(e) = try_send {
                println!("[匹配池] 内部错误：{e}");
            }
        }
        drop(guard);
    }
}

#[tokio::main]
async fn main() {
    println!("启动排位匹配服务器……");

    let peers = Arc::new(LockFreeCuckooHash::new());
    let arenas = Arc::new(DashMap::new());
    let senders = Arc::new(DashMap::new());

    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "[::]:12310".to_string());
    let try_socket = TcpListener::bind(&addr).await;
    let listener = match try_socket {
        Ok(s) => s,
        Err(e) => {
            panic!("监听失败啦！错误信息：{}", e);
        }
    };
    println!("正在监听: {}", addr);

    let http_client = match reqwest::Client::builder().build() {
        Ok(ans) => ans,
        Err(e) => panic!("无法创建http客户端！错误：{e}"),
    };

    tokio::spawn(rank_timer(
        Arc::clone(&peers),
        Arc::clone(&arenas),
        Arc::clone(&senders),
        http_client.clone(),
    ));

    println!("开始接受排位客户端（大厅服务器）连接！");
    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_connection(
            Arc::clone(&peers),
            Arc::clone(&arenas),
            Arc::clone(&senders),
            stream,
            addr,
        ));
    }
}
