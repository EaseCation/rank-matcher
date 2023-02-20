// Rank matching algorithm
use dashmap::DashMap;
use std::{borrow::Borrow, collections::HashSet, hash::Hash, sync::Arc};

// 一个匹配池
#[derive(Clone)]
pub struct Arena<T> {
    players: Arc<DashMap<T, (usize, usize, usize, usize)>>,
}

impl<T> Arena<T>
where
    T: Hash + Eq,
{
    pub fn new() -> Self {
        Arena {
            players: Arc::new(DashMap::new()),
        }
    }
}

impl<T> Arena<T>
where
    T: Hash + Eq,
{
    pub fn insert(
        &self,
        id: T,
        length: usize,
        rank_min: usize,
        rank_max: usize,
        speed: usize,
    ) -> Option<(usize, usize, usize, usize)> {
        self.players.insert(id, (rank_min, rank_max, length, speed))
    }

    pub fn remove<Q>(&self, id: &Q) -> Option<(usize, usize, usize, usize)>
        where
            T: Borrow<Q>,
            Q: Hash + Eq,
    {
        self.players.remove(id).map(|(_k, v)| v)
    }

    // pub fn get<Q>(&self, key: &Q) -> Option<&(usize, usize, usize)>
    // where
    //     T: Borrow<Q>,
    //     Q: Hash + Eq + ?Sized,
    // {
    //     self.players.get(key).clone()
    // }
}

impl<T> Arena<T>
where
    T: Hash + Eq,
{
    pub fn rank_update(&self) {
        for mut player in self.players.iter_mut() {
            let (min_rank_i, max_rank_i, _length, speed) = player.value_mut();
            *min_rank_i = min_rank_i.saturating_sub(*speed);
            *max_rank_i = max_rank_i.saturating_add(*speed);
        }
    }
}

impl<T> Arena<T>
where
    T: Hash + Eq + Clone + core::fmt::Debug,
{
    pub fn rank_match(&self) -> Vec<(T, usize)> {
        let players = {
            let mut players = DashMap::new();
            players.clone_from(&self.players);
            players
        };
        let mut max_rank = usize::min_value();
        let mut min_rank = usize::max_value();
        for player in players.iter() {
            let (min_rank_i, max_rank_i, _length, _speed) = *player;
            max_rank = usize::max(max_rank, max_rank_i);
            min_rank = usize::min(min_rank, min_rank_i);
        }
        if max_rank < min_rank {
            return Vec::new();
        }
        let mut cnt = vec![0isize; max_rank - min_rank + 2];
        for player in players.iter() {
            let (min_rank_i, max_rank_i, length, _speed) = player.value();
            assert!(*min_rank_i >= min_rank && *min_rank_i <= max_rank);
            assert!(*max_rank_i >= min_rank && *max_rank_i <= max_rank);
            let index_l = min_rank_i - min_rank;
            let index_r = max_rank_i - min_rank + 1;
            cnt[index_l] += *length as isize;
            cnt[index_r] -= *length as isize;
        }
        let mut max_cnt = isize::min_value();
        let mut max_cnt_i = 0;
        for i in 1..cnt.len() {
            cnt[i] += cnt[i - 1];
            if cnt[i] > max_cnt {
                max_cnt_i = i;
                max_cnt = cnt[i];
            }
        }
        let target_rank = max_cnt_i + min_rank;
        let mut ans = Vec::new();
        for player in players.iter() {
            let id = player.key().clone();
            let (min_rank_i, max_rank_i, length, _speed) = player.value();
            if *min_rank_i <= target_rank && target_rank <= *max_rank_i {
                ans.push((id, *length))
            }
        }
        ans
    }

    pub fn get_player_states(&self) -> DashMap<T, u64> {
        let ans = DashMap::new();
        let players = {
            let mut players = DashMap::new();
            players.clone_from(&self.players);
            players
        };

        let mut max_rank = usize::min_value();
        let mut min_rank = usize::max_value();
        for player in players.iter() {
            let (min_rank_i, max_rank_i, _length, _speed) = *player;
            max_rank = usize::max(max_rank, max_rank_i);
            min_rank = usize::min(min_rank, min_rank_i);
        }

        if max_rank < min_rank {
            return ans;
        }

        let mut cnt = vec![0isize; max_rank - min_rank + 2];
        let mut player_idx_l = vec![HashSet::new(); max_rank - min_rank + 2];
        let mut player_idx_r = vec![HashSet::new(); max_rank - min_rank + 2];
        for player in players.iter() {
            let (min_rank_i, max_rank_i, length, _speed) = *player.value();
            assert!(min_rank_i >= min_rank && min_rank_i <= max_rank);
            assert!(max_rank_i >= min_rank && max_rank_i <= max_rank);
            let index_l = min_rank_i - min_rank;
            let index_r = max_rank_i - min_rank + 1;
            assert!(/*index_l >= 0 && */index_l < max_rank - min_rank + 2);
            assert!(/*index_r >= 0 && */index_r < max_rank - min_rank + 2);
            assert!(cnt[index_r] >= length as isize);
            cnt[index_l] += length as isize;
            cnt[index_r] -= length as isize;
            player_idx_l[index_l].insert((player.key().clone(), length));
            player_idx_r[index_r].insert((player.key().clone(), length));
        }

        for i in 1..cnt.len() {
            cnt[i] += cnt[i - 1];
        }

        let mut cur_players = HashSet::new();
        for (idx, &cnt_i) in cnt.iter().enumerate() {
            // println!("idx = {idx}, cnt_i = {cnt_i}, players = {:?}", cur_players);
            cur_players.extend(player_idx_l[idx].iter());
            for player in cur_players.iter() {
                ans.entry(player.0.clone())
                    .and_modify(|e| *e = u64::max(*e, cnt_i as u64))
                    .or_insert(0);
            }
            cur_players.retain(|e| player_idx_r[idx].get(e).is_none());
        }

        ans
    }
}
