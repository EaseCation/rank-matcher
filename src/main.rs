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
    println!("来自{addr}的新TCP连接已建立，正在尝试连接为WebSocket……");

    let try_ws_stream = tokio_tungstenite::accept_async(raw_stream).await;

    let ws_stream = match try_ws_stream {
        Ok(ws_stream) => ws_stream,
        Err(e) => {
            println!("连接WebSocket流时发生错误，连接即将终止！错误：{e}");
            return;
        }
    };

    println!("通向地址{addr}的WebSocket连接已建立。");

    // 把写部分存到客户端表里面
    let (tx, rx) = mpsc::unbounded();
    peer_map.insert(addr, tx.clone());

    let (outgoing, incoming) = ws_stream.split();

    let broadcast_incoming = incoming.try_for_each(|msg| {
        let text = msg.to_text().unwrap();
        let packet = Packet::from_str(text);
        match packet {
            Ok(Packet::AddArena { arena, num_players }) => {
                if num_players == 0 {
                    println!("地址{addr}尝试注册匹配池{arena}，但匹配池的每局玩家数为0，创建失败！");
                } else {
                    let entry = arenas.entry(arena.clone());
                    entry.or_insert_with(|| (num_players, Arena::new()));
                    println!("地址{addr}已注册匹配池{arena}，达到{num_players}位玩家时，此匹配池将返回匹配结果。");
                }
            },
            Ok(Packet::RemoveArena(arena)) => {
                let removed = arenas.remove(&arena);
                if removed.is_some() {
                    println!("地址{addr}已删除匹配池{arena}。");
                } else {
                    println!("地址{addr}正在删除匹配池{arena}，此匹配池已不存在。")
                }
            },
            Ok(Packet::AddPlayer { arena, player, rank }) => {
                let try_arena = arenas.get(&arena);
                if let Some(arena_) = try_arena {
                    arena_.1.insert(player.clone(), rank as usize);
                    senders.insert(player.clone(), addr);
                    println!("地址{addr}成功向匹配池{arena}添加玩家{player}（分数为{rank}）。");
                } else {
                    println!("地址{addr}正在向{arena}添加玩家{player}（分数为{rank}），但此匹配池不存在。");
                }
            },
            Ok(Packet::RemovePlayer { arena, player }) => {
                let try_arena = arenas.get(&arena);
                if let Some(arena_) = try_arena {
                    arena_.1.remove(&player);
                    senders.remove(&player);
                    println!("地址{addr}成功从匹配池{arena}删除玩家{player}。");
                } else {
                    println!("地址{addr}正在向{arena}删除玩家{player}，但此匹配池不存在。");
                }
            },
            Err(e) => {
                println!("地址{addr}发送的包发生了格式错误：{}", e.0);
                let packet = Packet::FormatError { error: e.0.to_string() };
                let string = packet.to_string();
                let try_send = tx.unbounded_send(Message::Text(string));
                if let Err(e) = try_send {
                    println!("内部错误：{e}");
                }
            },
            _ => todo!()
        }

        future::ok(())
    });

    let receive_from_others = rx.map(Ok).forward(outgoing);

    pin_mut!(broadcast_incoming, receive_from_others);
    future::select(broadcast_incoming, receive_from_others).await;

    println!("地址{}已经断开WebSocket连接。", &addr);
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
    println!("已移除从地址{}注册的玩家，列表是：{:?}。", addr, players);

    peer_map.remove(&addr);
    println!("地址{}已经从排位匹配服务器解除注册，再见！", addr);
}

async fn rank_timer(peers: Peers, arenas: Arenas, senders: Senders) {
    let mut interval = time::interval(time::Duration::from_secs(1));
    println!("排位定时器开始工作！");
    loop {
        for arena_ref in arenas.iter() {
            let (num_players, arena) = arena_ref.value();
            let matched = arena.rank_match();
            if matched.len() >= *num_players as usize {
                // 匹配成功
                println!(
                    "匹配池{}成功匹配了{}位玩家：{:?}",
                    arena_ref.key(),
                    matched.len(),
                    matched
                );
                let collected: DashMap<SocketAddr, Vec<String>> = DashMap::new();
                for player in matched.clone() {
                    let try_addr = senders.get(&player);
                    if let Some(addr) = try_addr {
                        collected
                            .entry(addr.clone())
                            .and_modify(|v| v.push(player.clone()))
                            .or_insert_with(|| vec![player.clone()]);
                    }
                }
                for item_collected in collected {
                    let (addr, players) = item_collected;
                    println!("发送给地址{addr}的玩家列表：{:?}", players);
                    let packet = Packet::MatchSuccess {
                        arena: arena_ref.key().clone(),
                        players,
                    };
                    let string = packet.to_string();
                    let guard = lockfree_cuckoohash::pin();
                    if let Some(peer) = peers.get(&addr, &guard) {
                        let try_send = peer.unbounded_send(Message::Text(string));
                        if let Err(e) = try_send {
                            println!("内部错误：{e}");
                        }
                    }
                    drop(guard);
                }
                let guard = lockfree_cuckoohash::pin();
                for player in &matched {
                    arena.remove(player);
                }
                drop(guard);
                for player in &matched {
                    senders.remove(player);
                }
            }
            arena.rank_update();
        }
        interval.tick().await;
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
    println!("正在监听的地址是{}。", addr);

    tokio::spawn(rank_timer(
        Arc::clone(&peers),
        Arc::clone(&arenas),
        Arc::clone(&senders),
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
