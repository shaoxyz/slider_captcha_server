use std::{
    collections::VecDeque,
    sync::Arc,
    time::{Duration, Instant},
};

use dashmap::DashMap;

#[derive(Clone, Debug)]
pub struct CachedEntry<T> {
    pub value: Arc<T>,
    inserted_at: Instant,
}

#[derive(Clone, Debug)]
pub struct ExpiringCache<K, V>
where
    K: Eq + std::hash::Hash,
{
    ttl: Duration,
    max_len: usize,
    buckets: Arc<DashMap<K, VecDeque<CachedEntry<V>>>>,
}

impl<K, V> ExpiringCache<K, V>
where
    K: Eq + std::hash::Hash + Clone,
{
    pub fn new(ttl: Duration, max_len: usize) -> Self {
        Self {
            ttl,
            max_len,
            buckets: Arc::new(DashMap::new()),
        }
    }

    pub fn insert(&self, key: K, value: V) {
        let entry = CachedEntry {
            value: Arc::new(value),
            inserted_at: Instant::now(),
        };

        let mut bucket = self.buckets.entry(key).or_insert_with(VecDeque::new);

        bucket.push_back(entry);
        while bucket.len() > self.max_len {
            bucket.pop_front();
        }
    }

    pub fn pop(&self, key: &K) -> Option<Arc<V>> {
        if let Some(mut bucket) = self.buckets.get_mut(key) {
            while let Some(entry) = bucket.pop_back() {
                if entry.inserted_at.elapsed() < self.ttl {
                    return Some(entry.value);
                }
            }
        }
        None
    }

    pub fn len_for(&self, key: &K) -> usize {
        self.buckets
            .get(key)
            .map(|bucket| {
                bucket
                    .iter()
                    .filter(|entry| entry.inserted_at.elapsed() < self.ttl)
                    .count()
            })
            .unwrap_or(0)
    }

    pub fn total_len(&self) -> usize {
        self.buckets
            .iter()
            .map(|bucket| {
                bucket
                    .iter()
                    .filter(|entry| entry.inserted_at.elapsed() < self.ttl)
                    .count()
            })
            .sum()
    }

    pub fn clean_expired(&self) -> (usize, usize) {
        let mut removed = 0;
        let mut remaining = 0;

        for mut bucket in self.buckets.iter_mut() {
            let vec = bucket.value_mut();
            let before = vec.len();
            vec.retain(|entry| entry.inserted_at.elapsed() < self.ttl);
            removed += before - vec.len();
            remaining += vec.len();
        }

        (removed, remaining)
    }
}
