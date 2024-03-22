//! **Private** generalizations of containers:
//! - `Map`: `BTreeMap`, `HashMap` (any hasher) and _unordered_ `Vec`.

#![cfg(feature = "use_alloc")]

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
#[cfg(feature = "use_std")]
use core::hash::{BuildHasher, Hash};
#[cfg(feature = "use_std")]
use std::collections::HashMap;

pub trait Map {
    type Key;
    type Value;
    fn insert(&mut self, key: Self::Key, value: Self::Value) -> Option<Self::Value>;
    fn remove(&mut self, key: &Self::Key) -> Option<Self::Value>;
    fn aggregate<T, F>(&mut self, key: Self::Key, t: T, mut operation: F)
    where
        F: FnMut(Option<Self::Value>, &Self::Key, T) -> Option<Self::Value>,
    {
        let opt_value = self.remove(&key);
        if let Some(value) = operation(opt_value, &key, t) {
            self.insert(key, value);
        }
    }
    fn entry_or_default(&mut self, key: Self::Key) -> &mut Self::Value
    where
        Self::Value: Default;
}

impl<K, V> Map for BTreeMap<K, V>
where
    K: Ord,
{
    type Key = K;
    type Value = V;
    fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.insert(key, value)
    }
    fn remove(&mut self, key: &K) -> Option<V> {
        self.remove(key)
    }
    fn entry_or_default(&mut self, key: K) -> &mut V
    where
        V: Default,
    {
        self.entry(key).or_default()
    }
}

#[cfg(feature = "use_std")]
impl<K, V, S> Map for HashMap<K, V, S>
where
    K: Eq + Hash,
    S: BuildHasher,
{
    type Key = K;
    type Value = V;
    fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.insert(key, value)
    }
    fn remove(&mut self, key: &K) -> Option<V> {
        self.remove(key)
    }
    fn entry_or_default(&mut self, key: K) -> &mut V
    where
        V: Default,
    {
        self.entry(key).or_default()
    }
}

impl<K, V> Map for Vec<(K, V)>
where
    K: Eq,
{
    type Key = K;
    type Value = V;
    fn insert(&mut self, key: K, value: V) -> Option<V> {
        match self.iter_mut().find(|(k, _)| k == &key) {
            Some((_, v)) => Some(core::mem::replace(v, value)),
            None => {
                self.push((key, value));
                None
            }
        }
    }
    fn remove(&mut self, key: &K) -> Option<V> {
        let index = self.iter().position(|(k, _)| k == key)?;
        Some(self.swap_remove(index).1)
    }
    fn aggregate<T, F>(&mut self, key: K, t: T, mut operation: F)
    where
        F: FnMut(Option<V>, &K, T) -> Option<V>,
    {
        let opt_value = Map::remove(self, &key);
        if let Some(value) = operation(opt_value, &key, t) {
            // The key was removed so a single push is enough to insert it back.
            self.push((key, value));
        }
    }
    fn entry_or_default(&mut self, key: K) -> &mut V
    where
        V: Default,
    {
        let index = self.iter().position(|(k, _)| k == &key).unwrap_or_else(|| {
            self.push((key, V::default()));
            self.len() - 1
        });
        &mut self[index].1
    }
}
