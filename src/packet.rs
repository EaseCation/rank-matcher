use std::{str::FromStr, collections::VecDeque};

// 包，可以是收的也可以是发的
#[derive(Debug)]
pub enum Packet {
    AddArena(String),
    RemoveArena(String),
    AddPlayer {
        player: String,
        arena: String,
    },
    RemovePlayer {
        player: String,
        arena: String,
    },
    GetState,
    SubscribeState,
    MatchSuccess {
        arena: String,
        player: Vec<String>,        
    },
    MatchFailure {
        arena: String,
        player: Vec<String>,  
    },
    CommandFailure,
    PacketFormatError {
        error: String,
    },
}

// 包格式错误
#[derive(Debug)]
pub struct PacketFormat(&'static str);

impl FromStr for Packet {
    type Err = PacketFormat;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut reader = CharReader { inner: s.chars().collect() };
        reader.read_packet()
    }
}

struct CharReader {
    inner: VecDeque<char>,
}

impl CharReader {
    #[inline]
    fn read_packet(&mut self) -> Result<Packet, PacketFormat> {        
        match (self.inner.pop_front(), self.inner.pop_front()) {
            (Some('1'), Some(',')) => self.read_v1(),
            _ => Err(PacketFormat("不支持除了1之外的版本号。"))
        }
    }
    #[inline]
    fn read_v1(&mut self) -> Result<Packet, PacketFormat> {
        match (self.inner.pop_front(), self.inner.pop_front()) {
            (Some('1'), Some(',')) => self.read_v1_add_arena(),
            // (Some('2'), Some(',')) => self.handle_v1_text(text),
            // (Some('3'), Some(',')) => self.handle_v1_chan(text),
            // (Some('4'), Some(',')) => self.handle_v1_disconnect(text),
            _ => Err(PacketFormat("不支持除了1-9之外的包类别。"))
        }
    }
    #[inline]
    fn read_number(&mut self) -> u64 {
        let mut cur = self.inner.pop_front();
        while let Some(c) = cur {
            if c.to_digit(10).is_some() {
                break;
            } 
            cur = self.inner.pop_front();
        }
        let mut ans = 0;
        while let Some(c) = cur {
            if let Some(digit) = c.to_digit(10) {
                ans *= 10;
                ans += digit as u64;
                cur = self.inner.pop_front();
            } else {
                return ans;
            }
        };
        return ans;
    }
    #[inline]
    fn read_string(&mut self) -> String {
        let cap = self.read_number();
        let mut ans = String::with_capacity(cap as usize);
        for _i in 0..cap {
            if let Some(ch) = self.inner.pop_front() {
                ans.push(ch)
            }
        }
        ans
    }
    #[inline]
    fn read_v1_add_arena(&mut self) -> Result<Packet, PacketFormat> {
        let arena = self.read_string();
        Ok(Packet::AddArena(arena))
    }
}
