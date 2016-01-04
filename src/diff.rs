//! "Diff"ing iterators for caching elements to sequential collections without requiring the new
//! elements' iterator to be `Clone`.
//!
//! - [**Diff**](./enum.Diff.html) (produced by the [**diff**](./fn.diff.html) function) describes
//! the difference between two non-`Clone` iterators `a` and `b` after breaking ASAP from a
//! comparison with enough data to update `a`'s collection.
//! - [**copy_on_diff**](./fn.copy_on_diff.html) is an application of [**diff**] that compares two
//! iterators `a` and `b`, borrowing the source of `a` if they are the same or creating a new owned
//! collection with `b`'s elements if they are different.

use adaptors::PutBack;
use std::borrow::{Cow, ToOwned};
use std::iter::FromIterator;

/// A type returned by the [`diff`](./fn.diff.html) function.
///
/// `Diff` represents the way in which the elements (of type `E`) yielded by the iterator `I`
/// differ to some other iterator yielding borrowed elements of the same type.
///
/// `I` is some `Iterator` yielding elements of type `E`.
pub enum Diff<I>
    where I: Iterator,
{
    /// The index of the first non-matching element along with the iterator's remaining elements
    /// starting with the first mis-matched element.
    FirstMismatch(usize, PutBack<I>),
    /// The remaining elements of the iterator.
    Longer(PutBack<I>),
    /// The total number of elements that were in the iterator.
    Shorter(usize),
}

/// Compares every element yielded by both elems and new_elems in lock-step and returns a `Diff`
/// which describes how `j` differs from `i`.
///
/// This function is useful for caching some iterator `j` in some sequential collection without
/// requiring `j` to be `Clone` in order to compare it to the collection before determining if the
/// collection needs to be updated. The returned function returns as soon as a difference is found,
/// producing a `Diff` that provides the data necessary to update the collection without ever
/// requiring `J` to be `Clone`. This allows for efficiently caching iterators like `Map` or
/// `Filter` that do not implement `Clone`.
///
/// If the number of elements yielded by `j` is less than the number of elements yielded by `i`,
/// the number of `j` elements yielded will be returned as `Diff::Shorter`.
///
/// If the two elements of a step differ, the index of those elements along with the remaining
/// elements of `j` are returned as `Diff::FirstMismatch`.
///
/// If `i` becomes exhausted before `j` becomes exhausted, the remaining `j` elements will be
/// returned as `Diff::Longer`.
///
/// See [`copy_on_diff`](./fn.copy_on_diff.html) for an application of `diff`.
pub fn diff<'a, I, J>(i: I, j: J) -> Option<Diff<J::IntoIter>>
    where I: IntoIterator<Item=&'a J::Item>,
          J: IntoIterator,
          J::Item: PartialEq + 'a,
{
    let mut j = j.into_iter();
    for (idx, i_elem) in i.into_iter().enumerate() {
        match j.next() {
            None => return Some(Diff::Shorter(idx)),
            Some(j_elem) => if *i_elem != j_elem {
                return Some(Diff::FirstMismatch(idx, PutBack::value(j_elem, j)));
            },
        }
    }
    j.next().map(|elem| Diff::Longer(PutBack::value(elem, j)))
}

/// Returns `Cow::Borrowed` `collection` if `collection` contains the same elements as yielded by
/// `update`'s iterator.
///
/// Collects into a new `C::Owned` and returns `Cow::Owned` if either the number of elements or the
/// elements themselves differ.
///
/// # Examples
///
/// ```
/// use itertools::copy_on_diff;
/// use std::borrow::Cow;
///
/// let a = vec![0, 1, 2];
/// let b = vec![0.0, 1.0, 2.0];
/// let b_map = b.into_iter().map(|f| f as i32);
/// let cow = copy_on_diff(&a, b_map);
///
/// assert!(match cow {
///     Cow::Borrowed(slice) => slice == &a,
///     _ => false,
/// });
/// ```
///
/// ```
/// use itertools::copy_on_diff;
/// use std::borrow::Cow;
///
/// let a = vec![0, 1, 2, 3];
/// let b = vec![0.0, 1.0, 2.0];
/// let b_map = b.into_iter().map(|f| f as i32);
/// let cow = copy_on_diff(&a, b_map);
///
/// assert!(match cow {
///     Cow::Owned(vec) => vec == vec![0, 1, 2],
///     _ => false,
/// });
/// ```
pub fn copy_on_diff<'a, C, U, T: 'a>(collection: &'a C, update: U) -> Cow<'a, C>
    where &'a C: IntoIterator<Item=&'a T>,
          <&'a C as IntoIterator>::IntoIter: Clone,
          C: ToOwned,
          <C as ToOwned>::Owned: FromIterator<T>,
          U: IntoIterator<Item=T>,
          T: Clone + PartialEq,
{
    let c_iter = collection.into_iter();
    match diff(c_iter.clone(), update.into_iter()) {
        Some(diff) => match diff {
            Diff::FirstMismatch(idx, mismatch) =>
                Cow::Owned(c_iter.take(idx).cloned().chain(mismatch).collect()),
            Diff::Longer(remaining) =>
                Cow::Owned(c_iter.cloned().chain(remaining).collect()),
            Diff::Shorter(num_update) =>
                Cow::Owned(c_iter.cloned().take(num_update).collect()),
        },
        None => Cow::Borrowed(collection),
    }
}
