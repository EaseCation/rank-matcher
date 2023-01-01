mod arena;
mod packet;

use arena::Arena;
use futures_channel::mpsc::{self, UnboundedSender};
use futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt};
use lockfree_cuckoohash::LockFreeCuckooHash;
use std::{env, net::SocketAddr, sync::Arc, str::FromStr};
use tokio::net::{TcpListener, TcpStream};
use tungstenite::protocol::Message;

// 客户端，也就是大厅服务器
type Tx = UnboundedSender<Message>;
type PeerMap = Arc<LockFreeCuckooHash<SocketAddr, Tx>>;

// 所有匹配池的列表
type Arenas = Arc<LockFreeCuckooHash<String, Arena<String>>>;

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
        println!(
            "Received a message from {}: {}",
            addr,
            msg.to_text().unwrap()
        );
        let text = msg.to_text().unwrap();
        let packet = packet::Packet::from_str(text);
        println!("packet: {:?}", packet);
        

        let peers = Arc::clone(&peer_map);

        future::ok(())
    });

    let receive_from_others = rx.map(Ok).forward(outgoing);

    pin_mut!(broadcast_incoming, receive_from_others);
    future::select(broadcast_incoming, receive_from_others).await;

    println!("地址{}已经断开WebSocket连接。", &addr);
    peer_map.remove(&addr);
}

#[tokio::main]
async fn main() {
    println!("启动排位匹配服务器……");

    let state = Arc::new(LockFreeCuckooHash::new());
    let arenas = Arc::new(LockFreeCuckooHash::new());

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

    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_connection(
            Arc::clone(&state),
            Arc::clone(&arenas),
            stream,
            addr,
        ));
    }
}
