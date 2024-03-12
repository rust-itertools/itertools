//! **Private** generalizations of containers:
//! - `Map`: `BTreeMap` and `HashMap` (any hasher).

#![cfg(feature = "use_alloc")]

use alloc::collections::BTreeMap;
#[cfg(feature = "use_std")]
use core::hash::{BuildHasher, Hash};
#[cfg(feature = "use_std")]
use std::collections::HashMap;

pub trait Map {
    type Key;
    type Value;
    fn insert(&mut self, key: Self::Key, value: Self::Value) -> Option<Self::Value>;
    fn remove(&mut self, key: &Self::Key) -> Option<Self::Value>;
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
