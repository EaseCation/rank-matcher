use std::{collections::VecDeque, str::FromStr};

// 包，可以是收的也可以是发的
#[derive(Debug)]
pub enum Packet {
    AddArena {
        arena: String,
        num_players: u64,
    },
    RemoveArena(String),
    AddPlayer {
        arena: String,
        player: String,
        rank: u64,
    },
    RemovePlayer {
        arena: String,
        player: String,
    },
    GetState,
    SubscribeState,
    MatchSuccess {
        arena: String,
        players: Vec<String>,
    },
    MatchFailure {
        arena: String,
        players: Vec<String>,
    },
    FormatError {
        error: String,
    },
}

// 包格式错误
#[derive(Debug)]
pub struct PacketFormat(pub &'static str);

impl FromStr for Packet {
    type Err = PacketFormat;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut reader = CharReader {
            inner: s.chars().collect(),
        };
        reader.read_packet()
    }
}

impl ToString for Packet {
    fn to_string(&self) -> String {
        let mut writer = CharWriter {
            inner: VecDeque::new(),
        };
        writer.write_packet(self);
        let mut ans = String::new();
        ans.extend(writer.inner);
        ans
    }
}

struct CharWriter {
    inner: VecDeque<char>,
}

impl CharWriter {
    #[inline]
    fn write_packet(&mut self, packet: &Packet) {
        self.inner.push_back('1'); // version
        match packet {
            Packet::AddArena { arena, num_players } => {
                self.inner.push_back(',');
                self.inner.push_back('1');
                self.write_string(&arena);
                self.write_number(*num_players);
            }
            Packet::RemoveArena(arena) => {
                self.inner.push_back(',');
                self.inner.push_back('2');
                self.write_string(&arena);
            }
            Packet::AddPlayer {
                arena,
                player,
                rank,
            } => {
                self.inner.push_back(',');
                self.inner.push_back('3');
                self.write_string(&arena);
                self.write_string(&player);
                self.write_number(*rank);
            }
            Packet::RemovePlayer { arena, player } => {
                self.inner.push_back(',');
                self.inner.push_back('4');
                self.write_string(&arena);
                self.write_string(&player);
            }
            Packet::GetState => {
                self.inner.push_back(',');
                self.inner.push_back('5');
            }
            Packet::SubscribeState => {
                self.inner.push_back(',');
                self.inner.push_back('6');
            }
            Packet::MatchSuccess { arena, players } => {
                self.inner.push_back(',');
                self.inner.push_back('7');
                self.write_string(&arena);
                self.write_number(players.len() as u64);
                for player in players {
                    self.write_string(&player);
                }
            }
            Packet::MatchFailure { arena, players } => {
                self.inner.push_back(',');
                self.inner.push_back('8');
                self.write_string(&arena);
                self.write_number(players.len() as u64);
                for player in players {
                    self.write_string(&player);
                }
            }
            Packet::FormatError { error } => {
                self.inner.push_back(',');
                self.inner.push_back('9');
                self.write_string(&error);
            }
        }
    }
    #[inline]
    fn write_number(&mut self, number: u64) {
        // 以后再优化吧
        let string = format!(",{}", number);
        self.inner
            .extend(string.chars().collect::<VecDeque<char>>());
    }
    #[inline]
    fn write_string(&mut self, string: &str) {
        self.write_number(string.len() as u64);
        self.inner.push_back(',');
        self.inner
            .extend(string.chars().collect::<VecDeque<char>>());
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
            _ => Err(PacketFormat("不支持除了1之外的版本号。")),
        }
    }
    #[inline]
    fn read_v1(&mut self) -> Result<Packet, PacketFormat> {
        match (self.inner.pop_front(), self.inner.pop_front()) {
            (Some('1'), Some(',')) => self.read_v1_add_arena(),
            (Some('2'), Some(',')) => self.read_v1_remove_arena(),
            (Some('3'), Some(',')) => self.read_v1_add_player(),
            (Some('4'), Some(',')) => self.read_v1_remove_player(),
            (Some('5'), _) => self.read_v1_get_state(),
            (Some('6'), _) => self.read_v1_subscribe_state(),
            (Some('7'), Some(',')) => self.read_v1_match_success(),
            (Some('8'), Some(',')) => self.read_v1_match_failure(),
            (Some('9'), Some(',')) => self.read_v1_format_error(),
            _ => Err(PacketFormat("不支持除了1-9之外的包类别。")),
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
        }
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
        let num_players = self.read_number();
        Ok(Packet::AddArena { arena, num_players })
    }
    #[inline]
    fn read_v1_remove_arena(&mut self) -> Result<Packet, PacketFormat> {
        let arena = self.read_string();
        Ok(Packet::RemoveArena(arena))
    }
    #[inline]
    fn read_v1_add_player(&mut self) -> Result<Packet, PacketFormat> {
        let arena = self.read_string();
        let player = self.read_string();
        let rank = self.read_number();
        Ok(Packet::AddPlayer {
            arena,
            player,
            rank,
        })
    }
    #[inline]
    fn read_v1_remove_player(&mut self) -> Result<Packet, PacketFormat> {
        let arena = self.read_string();
        let player = self.read_string();
        Ok(Packet::RemovePlayer { arena, player })
    }
    #[inline]
    fn read_v1_get_state(&mut self) -> Result<Packet, PacketFormat> {
        Ok(Packet::GetState)
    }
    #[inline]
    fn read_v1_subscribe_state(&mut self) -> Result<Packet, PacketFormat> {
        Ok(Packet::SubscribeState)
    }
    #[inline]
    fn read_v1_match_success(&mut self) -> Result<Packet, PacketFormat> {
        let arena = self.read_string();
        let number = self.read_number();
        let mut players = Vec::with_capacity(number as usize);
        for _ in 0..number {
            players.push(self.read_string());
        }
        Ok(Packet::MatchSuccess { arena, players })
    }
    #[inline]
    fn read_v1_match_failure(&mut self) -> Result<Packet, PacketFormat> {
        let arena = self.read_string();
        let number = self.read_number();
        let mut players = Vec::with_capacity(number as usize);
        for _ in 0..number {
            players.push(self.read_string());
        }
        Ok(Packet::MatchFailure { arena, players })
    }
    #[inline]
    fn read_v1_format_error(&mut self) -> Result<Packet, PacketFormat> {
        let error = self.read_string();
        Ok(Packet::FormatError { error })
    }
}
