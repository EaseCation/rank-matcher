// Rank matching algorithm
use dashmap::DashMap;
use std::{borrow::Borrow, hash::Hash, sync::Arc};

// 一个匹配池
#[derive(Clone)]
pub struct Arena<T> {
    players: Arc<DashMap<T, (usize, usize, usize)>>,
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
    pub fn insert(&self, id: T, rank: usize, length: usize) -> Option<(usize, usize, usize)> {
        self.players.insert(id, (rank, rank, length))
    }

    pub fn remove<Q>(&self, id: &Q) -> Option<(usize, usize, usize)>
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
            let (min_rank_i, max_rank_i, _length) = player.value_mut();
            if *min_rank_i > usize::min_value() {
                *min_rank_i -= 1;
            }
            if *max_rank_i < usize::max_value() {
                *max_rank_i += 1;
            }
        }
    }
}

impl<T> Arena<T>
where
    T: Hash + Eq + Clone,
{
    pub fn rank_match(&self) -> Vec<(T, usize)> {
        let mut max_rank = usize::min_value();
        let mut min_rank = usize::max_value();
        for player in self.players.iter() {
            let (min_rank_i, max_rank_i, _length) = *player;
            max_rank = usize::max(max_rank, max_rank_i);
            min_rank = usize::min(min_rank, min_rank_i);
        }
        if max_rank < min_rank {
            return Vec::new();
        }
        let mut cnt = vec![0isize; max_rank - min_rank + 2];
        for player in self.players.iter() {
            let (min_rank_i, max_rank_i, length) = player.value();
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
        for player in self.players.iter() {
            let id = player.key().clone();
            let (min_rank_i, max_rank_i, length) = player.value();
            if *min_rank_i <= target_rank && target_rank <= *max_rank_i {
                ans.push((id, *length))
            }
        }
        ans
    }
}
