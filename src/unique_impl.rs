use core::hash::BuildHasher;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, RandomState};
use std::iter::FusedIterator;

/// An iterator adapter to filter out duplicate elements.
///
/// See [`.unique_by()`](crate::Itertools::unique) for more information.
#[derive(Clone)]
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct UniqueBy<I: Iterator, V, F, S = RandomState>
where
    S: BuildHasher,
{
    iter: I,
    // Use a Hashmap for the Entry API in order to prevent hashing twice.
    // This can maybe be replaced with a HashSet once `get_or_insert_with`
    // or a proper Entry API for Hashset is stable and meets this msrv
    used: HashMap<V, (), S>,
    f: F,
}

impl<I, V, F, S> fmt::Debug for UniqueBy<I, V, F, S>
where
    I: Iterator + fmt::Debug,
    V: fmt::Debug + Hash + Eq,
    S: BuildHasher,
{
    debug_fmt_fields!(UniqueBy, iter, used);
}

/// Create a new `UniqueBy` iterator.
pub fn unique_by_with_hasher<I, V, F, S>(iter: I, f: F, hash_builder: S) -> UniqueBy<I, V, F, S>
where
    V: Eq + Hash,
    F: FnMut(&I::Item) -> V,
    I: Iterator,
    S: BuildHasher,
{
    UniqueBy {
        iter,
        used: HashMap::with_hasher(hash_builder),
        f,
    }
}

// count the number of new unique keys in iterable (`used` is the set already seen)
fn count_new_keys<I, K, S>(mut used: HashMap<K, (), S>, iterable: I) -> usize
where
    I: IntoIterator<Item = K>,
    K: Hash + Eq,
    S: BuildHasher,
{
    let iter = iterable.into_iter();
    let current_used = used.len();
    used.extend(iter.map(|key| (key, ())));
    used.len() - current_used
}

impl<I, V, F, S> Iterator for UniqueBy<I, V, F, S>
where
    I: Iterator,
    V: Eq + Hash,
    F: FnMut(&I::Item) -> V,
    S: BuildHasher,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let Self { iter, used, f } = self;
        iter.find(|v| used.insert(f(v), ()).is_none())
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (low, hi) = self.iter.size_hint();
        ((low > 0 && self.used.is_empty()) as usize, hi)
    }

    fn count(self) -> usize {
        let mut key_f = self.f;
        count_new_keys(self.used, self.iter.map(move |elt| key_f(&elt)))
    }
}

impl<I, V, F, S> DoubleEndedIterator for UniqueBy<I, V, F, S>
where
    I: DoubleEndedIterator,
    V: Eq + Hash,
    F: FnMut(&I::Item) -> V,
    S: BuildHasher,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { iter, used, f } = self;
        iter.rfind(|v| used.insert(f(v), ()).is_none())
    }
}

impl<I, V, F, S> FusedIterator for UniqueBy<I, V, F, S>
where
    I: FusedIterator,
    V: Eq + Hash,
    F: FnMut(&I::Item) -> V,
    S: BuildHasher,
{
}

impl<I, S> Iterator for Unique<I, S>
where
    I: Iterator,
    I::Item: Eq + Hash + Clone,
    S: BuildHasher,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let UniqueBy { iter, used, .. } = &mut self.iter;
        iter.find_map(|v| {
            if let Entry::Vacant(entry) = used.entry(v) {
                let elt = entry.key().clone();
                entry.insert(());
                return Some(elt);
            }
            None
        })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (low, hi) = self.iter.iter.size_hint();
        ((low > 0 && self.iter.used.is_empty()) as usize, hi)
    }

    fn count(self) -> usize {
        count_new_keys(self.iter.used, self.iter.iter)
    }
}

impl<I, S> DoubleEndedIterator for Unique<I, S>
where
    I: DoubleEndedIterator,
    I::Item: Eq + Hash + Clone,
    S: BuildHasher,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let UniqueBy { iter, used, .. } = &mut self.iter;
        iter.rev().find_map(|v| {
            if let Entry::Vacant(entry) = used.entry(v) {
                let elt = entry.key().clone();
                entry.insert(());
                return Some(elt);
            }
            None
        })
    }
}

impl<I, S> FusedIterator for Unique<I, S>
where
    I: FusedIterator,
    I::Item: Eq + Hash + Clone,
    S: BuildHasher,
{
}

/// An iterator adapter to filter out duplicate elements.
///
/// See [`.unique()`](crate::Itertools::unique) for more information.
#[derive(Clone)]
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct Unique<I, S = RandomState>
where
    I: Iterator,
    I::Item: Eq + Hash + Clone,
    S: BuildHasher,
{
    iter: UniqueBy<I, I::Item, (), S>,
}

impl<I, S> fmt::Debug for Unique<I, S>
where
    I: Iterator + fmt::Debug,
    I::Item: Hash + Eq + fmt::Debug + Clone,
    S: BuildHasher,
{
    debug_fmt_fields!(Unique, iter);
}

pub fn unique_with_hasher<I, S>(iter: I, hash_builder: S) -> Unique<I, S>
where
    I: Iterator,
    I::Item: Eq + Hash + Clone,
    S: BuildHasher,
{
    Unique {
        iter: UniqueBy {
            iter,
            used: HashMap::with_hasher(hash_builder),
            f: (),
        },
    }
}
