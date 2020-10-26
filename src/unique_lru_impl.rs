
use std::hash::Hash;
use std::fmt;
use lru::LruCache;

/// An iterator adapter to filter out duplicate elements.
///
/// See [`.unique_by_lru()`](../trait.Itertools.html#method.unique_lru) for more information.
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct UniqueByLru<I: Iterator, V, F> {
    iter: I,
    used: LruCache<V, ()>,
    f: F,
}

impl<I, V, F> fmt::Debug for UniqueByLru<I, V, F>
    where I: Iterator + fmt::Debug,
          V: fmt::Debug + Hash + Eq,
{
    debug_fmt_fields!(UniqueByLru, iter, used);
}

/// Create a new `UniqueByLru` iterator.
pub fn unique_by_lru<I, V, F>(iter: I, capacity: usize, f: F) -> UniqueByLru<I, V, F>
    where V: Eq + Hash,
          F: FnMut(&I::Item) -> V,
          I: Iterator,
{
    UniqueByLru {
        iter,
        used: LruCache::new(capacity),
        f,
    }
}

impl<I, V, F> Iterator for UniqueByLru<I, V, F>
    where I: Iterator,
          V: Eq + Hash,
          F: FnMut(&I::Item) -> V
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(v) = self.iter.next() {
            let key = (self.f)(&v);
            if self.used.put(key, ()).is_none() {
                return Some(v);
            }
        }
        None
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<I, V, F> DoubleEndedIterator for UniqueByLru<I, V, F>
    where I: DoubleEndedIterator,
          V: Eq + Hash,
          F: FnMut(&I::Item) -> V
{
    fn next_back(&mut self) -> Option<Self::Item> {
        while let Some(v) = self.iter.next_back() {
            let key = (self.f)(&v);
            if self.used.put(key, ()).is_none() {
                return Some(v);
            }
        }
        None
    }
}

impl<I> Iterator for UniqueLru<I>
    where I: Iterator,
          I::Item: Eq + Hash + Clone
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(v) = self.iter.iter.next() {
            if self.iter.used.put(v.clone(), ()).is_none() {
                return Some(v);
            }
        }
        None
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.iter.size_hint()
    }
}

impl<I> DoubleEndedIterator for UniqueLru<I>
    where I: DoubleEndedIterator,
          I::Item: Eq + Hash + Clone
{
    fn next_back(&mut self) -> Option<Self::Item> {
        while let Some(v) = self.iter.iter.next_back() {
            if self.iter.used.put(v.clone(), ()).is_none() {
                return Some(v);
            }
        }
        None
    }
}

/// An iterator adapter to filter out duplicate elements.
///
/// See [`.unique_lru()`](../trait.Itertools.html#method.unique_lru) for more information.
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct UniqueLru<I: Iterator> {
    iter: UniqueByLru<I, I::Item, ()>,
}

impl<I> fmt::Debug for UniqueLru<I>
    where I: Iterator + fmt::Debug,
          I::Item: Hash + Eq + fmt::Debug,
{
    debug_fmt_fields!(Unique, iter);
}

pub fn unique_lru<I>(iter: I, capacity: usize) -> UniqueLru<I>
    where I: Iterator,
          I::Item: Eq + Hash,
{
    UniqueLru {
        iter: UniqueByLru {
            iter,
            used: LruCache::new(capacity),
            f: (),
        }
    }
}
