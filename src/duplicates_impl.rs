use core::hash::BuildHasher;
use std::hash::{Hash, RandomState};

mod private {
    use core::hash::BuildHasher;
    use std::collections::HashMap;
    use std::fmt;
    use std::hash::{Hash, RandomState};

    #[derive(Clone)]
    #[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
    pub struct DuplicatesBy<I: Iterator, Key, F, S = RandomState>
    where
        S: BuildHasher,
    {
        pub(crate) iter: I,
        pub(crate) meta: Meta<Key, F, S>,
    }

    impl<I, V, F, S> fmt::Debug for DuplicatesBy<I, V, F, S>
    where
        I: Iterator + fmt::Debug,
        V: fmt::Debug + Hash + Eq,
        S: BuildHasher,
    {
        debug_fmt_fields!(DuplicatesBy, iter, meta.used);
    }

    impl<I: Iterator, Key: Eq + Hash, F, S: BuildHasher> DuplicatesBy<I, Key, F, S> {
        pub(crate) fn new(iter: I, key_method: F, hash_builder: S) -> Self {
            Self {
                iter,
                meta: Meta {
                    used: HashMap::with_hasher(hash_builder),
                    pending: 0,
                    key_method,
                },
            }
        }
    }

    #[derive(Clone)]
    pub struct Meta<Key, F, S> {
        used: HashMap<Key, bool, S>,
        pending: usize,
        key_method: F,
    }

    impl<Key, F, S> Meta<Key, F, S>
    where
        Key: Eq + Hash,
        S: BuildHasher,
    {
        /// Takes an item and returns it back to the caller if it's the second time we see it.
        /// Otherwise the item is consumed and None is returned
        #[inline(always)]
        fn filter<I>(&mut self, item: I) -> Option<I>
        where
            F: KeyMethod<Key, I>,
        {
            let kv = self.key_method.make(item);
            match self.used.get_mut(kv.key_ref()) {
                None => {
                    self.used.insert(kv.key(), false);
                    self.pending += 1;
                    None
                }
                Some(true) => None,
                Some(produced) => {
                    *produced = true;
                    self.pending -= 1;
                    Some(kv.value())
                }
            }
        }
    }

    impl<I, Key, F, S> Iterator for DuplicatesBy<I, Key, F, S>
    where
        I: Iterator,
        Key: Eq + Hash,
        F: KeyMethod<Key, I::Item>,
        S: BuildHasher,
    {
        type Item = I::Item;

        fn next(&mut self) -> Option<Self::Item> {
            let Self { iter, meta } = self;
            iter.find_map(|v| meta.filter(v))
        }

        #[inline]
        fn size_hint(&self) -> (usize, Option<usize>) {
            let (_, hi) = self.iter.size_hint();
            let hi = hi.map(|hi| {
                if hi <= self.meta.pending {
                    // fewer or equally many iter-remaining elements than pending elements
                    // => at most, each iter-remaining element is matched
                    hi
                } else {
                    // fewer pending elements than iter-remaining elements
                    // => at most:
                    //    * each pending element is matched
                    //    * the other iter-remaining elements come in pairs
                    self.meta.pending + (hi - self.meta.pending) / 2
                }
            });
            // The lower bound is always 0 since we might only get unique items from now on
            (0, hi)
        }
    }

    impl<I, Key, F, S> DoubleEndedIterator for DuplicatesBy<I, Key, F, S>
    where
        I: DoubleEndedIterator,
        Key: Eq + Hash,
        F: KeyMethod<Key, I::Item>,
        S: BuildHasher,
    {
        fn next_back(&mut self) -> Option<Self::Item> {
            let Self { iter, meta } = self;
            iter.rev().find_map(|v| meta.filter(v))
        }
    }

    /// A keying method for use with `DuplicatesBy`
    pub trait KeyMethod<K, V> {
        type Container: KeyXorValue<K, V>;

        fn make(&mut self, value: V) -> Self::Container;
    }

    /// Apply the identity function to elements before checking them for equality.
    #[derive(Debug, Clone)]
    pub struct ById;
    impl<V> KeyMethod<V, V> for ById {
        type Container = JustValue<V>;

        fn make(&mut self, v: V) -> Self::Container {
            JustValue(v)
        }
    }

    /// Apply a user-supplied function to elements before checking them for equality.
    #[derive(Clone)]
    pub struct ByFn<F>(pub(crate) F);
    impl<F> fmt::Debug for ByFn<F> {
        debug_fmt_fields!(ByFn,);
    }
    impl<K, V, F> KeyMethod<K, V> for ByFn<F>
    where
        F: FnMut(&V) -> K,
    {
        type Container = KeyValue<K, V>;

        fn make(&mut self, v: V) -> Self::Container {
            KeyValue((self.0)(&v), v)
        }
    }

    // Implementors of this trait can hold onto a key and a value but only give access to one of them
    // at a time. This allows the key and the value to be the same value internally
    pub trait KeyXorValue<K, V> {
        fn key_ref(&self) -> &K;
        fn key(self) -> K;
        fn value(self) -> V;
    }

    #[derive(Debug)]
    pub struct KeyValue<K, V>(K, V);
    impl<K, V> KeyXorValue<K, V> for KeyValue<K, V> {
        fn key_ref(&self) -> &K {
            &self.0
        }
        fn key(self) -> K {
            self.0
        }
        fn value(self) -> V {
            self.1
        }
    }

    #[derive(Debug)]
    pub struct JustValue<V>(V);
    impl<V> KeyXorValue<V, V> for JustValue<V> {
        fn key_ref(&self) -> &V {
            &self.0
        }
        fn key(self) -> V {
            self.0
        }
        fn value(self) -> V {
            self.0
        }
    }
}

/// An iterator adapter to filter for duplicate elements.
///
/// See [`.duplicates_by()`](crate::Itertools::duplicates_by) for more information.
pub type DuplicatesBy<I, V, F, S = RandomState> = private::DuplicatesBy<I, V, private::ByFn<F>, S>;

/// Create a new `DuplicatesBy` iterator with a specified hash builder.
pub fn duplicates_by_with_hasher<I, Key, F, S>(
    iter: I,
    f: F,
    hash_builder: S,
) -> DuplicatesBy<I, Key, F, S>
where
    Key: Eq + Hash,
    F: FnMut(&I::Item) -> Key,
    I: Iterator,
    S: BuildHasher,
{
    DuplicatesBy::new(iter, private::ByFn(f), hash_builder)
}

/// An iterator adapter to filter out duplicate elements.
///
/// See [`.duplicates()`](crate::Itertools::duplicates) for more information.
pub type Duplicates<I, S = RandomState> =
    private::DuplicatesBy<I, <I as Iterator>::Item, private::ById, S>;

/// Create a new `Duplicates` iterator with a specified hash builder.
pub fn duplicates_with_hasher<I, S>(iter: I, hash_builder: S) -> Duplicates<I, S>
where
    I: Iterator,
    I::Item: Eq + Hash,
    S: BuildHasher,
{
    Duplicates::new(iter, private::ById, hash_builder)
}
