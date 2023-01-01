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
type PeerMap = Arc<LockFreeCuckooHash<SocketAddr, Tx>>;

// 所有匹配池的列表
type Arenas = Arc<dashmap::DashMap<String, Arena<String>>>;

async fn handle_connection(
    peer_map: PeerMap,
    arenas: Arenas,
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
    peer_map.insert(addr, tx);

    let (outgoing, incoming) = ws_stream.split();

    let broadcast_incoming = incoming.try_for_each(|msg| {
        let text = msg.to_text().unwrap();
        let packet = Packet::from_str(text);
        match packet {
            Ok(Packet::AddArena(arena)) => {
                let entry = arenas.entry(arena.clone());
                entry.or_insert_with(|| Arena::new());
                println!("地址{addr}已注册匹配池{arena}。");
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
                    arena_.insert(player.clone(), rank as usize);
                    println!("地址{addr}成功向匹配池{arena}添加玩家{player}（分数为{rank}）。");
                } else {
                    println!("地址{addr}正在向{arena}添加玩家{player}（分数为{rank}），但此匹配池不存在。");
                }
            },
            Ok(Packet::RemovePlayer { arena, player }) => {
                let try_arena = arenas.get(&arena);
                if let Some(arena_) = try_arena {
                    arena_.remove(&player);
                    println!("地址{addr}成功从匹配池{arena}删除玩家{player}。");
                } else {
                    println!("地址{addr}正在向{arena}删除玩家{player}，但此匹配池不存在。");
                }
            },
            _ => todo!()
        }

        let peers = Arc::clone(&peer_map);

        future::ok(())
    });

    let receive_from_others = rx.map(Ok).forward(outgoing);

    pin_mut!(broadcast_incoming, receive_from_others);
    future::select(broadcast_incoming, receive_from_others).await;

    println!("地址{}已经断开WebSocket连接。", &addr);
    peer_map.remove(&addr);
}

async fn rank_timer(arenas: Arenas) {
    let mut interval = time::interval(time::Duration::from_secs(1));
    println!("排位定时器开始工作！");
    loop {
        // todo: 收到信号停止
        println!("+1s");
        for arena in arenas.iter() {
            // if arena.rank_match()
            arena.rank_update();
        }
        interval.tick().await;
    }
}

#[tokio::main]
async fn main() {
    println!("启动排位匹配服务器……");

    let state = Arc::new(LockFreeCuckooHash::new());
    let arenas = Arc::new(DashMap::new());

    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "[::1]:12310".to_string());
    let try_socket = TcpListener::bind(&addr).await;
    let listener = match try_socket {
        Ok(s) => s,
        Err(e) => {
            panic!("监听失败啦！错误信息：{}", e);
        }
    };
    println!("正在监听的地址是{}。", addr);

    tokio::spawn(rank_timer(Arc::clone(&arenas)));

    println!("开始接受排位客户端（大厅服务器）连接！");
    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_connection(
            Arc::clone(&state),
            Arc::clone(&arenas),
            stream,
            addr,
        ));
    }
}
