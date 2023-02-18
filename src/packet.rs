use std::{collections::VecDeque, str::FromStr};

use dashmap::DashMap;

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
        // 通常是1。用于按队伍为单位匹配时，以队长的名义和分数匹配，此时length为队伍成员的数量
        length: u64,
    },
    RemovePlayer {
        arena: String,
        player: String,
    },
    GetOrSubscribeState {
        // 0 => 立即返回，并且以后不再发送, 非0 => 每隔多少秒返回一次
        period: u64,
    },
    ConnectionState {
        // 玩家名称 => (匹配池名称, 已经匹配的人数)
        player_info: DashMap<String, (String, u64)>,
    },
    MatchSuccess {
        arena: String,
        stage_request_id: u64, // 请求创建房间的requestId，然后交给各个nk去轮询检查房间是否创建成功
        // String是玩家的名称，u64是队伍内玩家的个数。u64通常是1
        // 若不为1表示String为队长的名字
        players: Vec<(String, u64)>,
    },
    MatchFailure {
        arena: String,
        error_id: u64,
        error_msg: String,
        players: Vec<(String, u64)>,
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
                length,
            } => {
                self.inner.push_back(',');
                self.inner.push_back('3');
                self.write_string(&arena);
                self.write_string(&player);
                self.write_number(*rank);
                self.write_number(*length);
            }
            Packet::RemovePlayer { arena, player } => {
                self.inner.push_back(',');
                self.inner.push_back('4');
                self.write_string(&arena);
                self.write_string(&player);
            }
            Packet::GetOrSubscribeState { period } => {
                self.inner.push_back(',');
                self.inner.push_back('5');
                self.write_number(*period);
            }
            Packet::ConnectionState { player_info } => {
                self.inner.push_back(',');
                self.inner.push_back('6');
                self.write_number(player_info.len() as u64);
                for info in player_info {
                    let player = info.key();
                    let (arena, num_matched) = info.value();
                    self.write_string(player);
                    self.write_string(arena);
                    self.write_number(*num_matched);
                }
            }
            Packet::MatchSuccess {
                arena,
                stage_request_id,
                players,
            } => {
                self.inner.push_back(',');
                self.inner.push_back('7');
                self.write_string(&arena);
                self.write_number(*stage_request_id);
                self.write_number(players.len() as u64);
                for (player, length) in players {
                    self.write_string(&player);
                    self.write_number(*length)
                }
            }
            Packet::MatchFailure {
                arena,
                error_id,
                error_msg,
                players,
            } => {
                self.inner.push_back(',');
                self.inner.push_back('8');
                self.write_string(&arena);
                self.write_number(*error_id);
                self.write_string(&error_msg);
                self.write_number(players.len() as u64);
                for (player, length) in players {
                    self.write_string(&player);
                    self.write_number(*length)
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
            (Some('5'), Some(',')) => self.read_v1_get_or_subscribe_state(),
            (Some('6'), Some(',')) => self.read_v1_connection_state(),
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
        let length = self.read_number();
        Ok(Packet::AddPlayer {
            arena,
            player,
            rank,
            length,
        })
    }
    #[inline]
    fn read_v1_remove_player(&mut self) -> Result<Packet, PacketFormat> {
        let arena = self.read_string();
        let player = self.read_string();
        Ok(Packet::RemovePlayer { arena, player })
    }
    #[inline]
    fn read_v1_get_or_subscribe_state(&mut self) -> Result<Packet, PacketFormat> {
        let period = self.read_number();
        Ok(Packet::GetOrSubscribeState { period })
    }
    #[inline]
    fn read_v1_connection_state(&mut self) -> Result<Packet, PacketFormat> {
        let number = self.read_number();
        let player_info = DashMap::with_capacity(number as usize);
        for _ in 0..number {
            let player = self.read_string();
            let arena = self.read_string();
            let num_matched = self.read_number();
            player_info.insert(player, (arena, num_matched));
        }
        Ok(Packet::ConnectionState { player_info })
    }
    #[inline]
    fn read_v1_match_success(&mut self) -> Result<Packet, PacketFormat> {
        let arena = self.read_string();
        let stage_request_id = self.read_number();
        let number = self.read_number();
        let mut players = Vec::with_capacity(number as usize);
        for _ in 0..number {
            let player = self.read_string();
            let length = self.read_number();
            players.push((player, length));
        }
        Ok(Packet::MatchSuccess {
            arena,
            stage_request_id,
            players,
        })
    }
    #[inline]
    fn read_v1_match_failure(&mut self) -> Result<Packet, PacketFormat> {
        let arena = self.read_string();
        let error_id = self.read_number();
        let error_msg = self.read_string();
        let number = self.read_number();
        let mut players = Vec::with_capacity(number as usize);
        for _ in 0..number {
            let player = self.read_string();
            let length = self.read_number();
            players.push((player, length));
        }
        Ok(Packet::MatchFailure {
            arena,
            error_id,
            error_msg,
            players,
        })
    }
    #[inline]
    fn read_v1_format_error(&mut self) -> Result<Packet, PacketFormat> {
        let error = self.read_string();
        Ok(Packet::FormatError { error })
    }
}
