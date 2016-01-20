//! "Diff"ing iterators for caching elements to sequential collections without requiring the new
//! elements' iterator to be `Clone`.
//!
//! - [**Diff**](./enum.Diff.html) (produced by the [**diff**](./fn.diff.html) and
//! [**diff_by_ref**](./fn.diff_by_ref.html) functions) describes the difference between two
//! non-`Clone` iterators `a` and `b` after breaking ASAP from a comparison with enough data to
//! update `a`'s collection.
//! - [**copy_on_diff**](./fn.copy_on_diff.html) is an application of [**diff**] that compares two
//! iterators `a` and `b`, borrowing the source of `a` if they are the same or creating a new owned
//! collection with `b`'s elements if they are different.

use adaptors::PutBack;
use std::borrow::{Cow, ToOwned};
use std::iter::FromIterator;

/// A type returned by the [`diff`](./fn.diff.html) function.
///
/// `Diff` represents the way in which the elements yielded by the iterator `I` differ to some
/// iterator `J`.
pub enum Diff<I, J>
    where I: Iterator,
          J: Iterator,
{
    /// The index of the first non-matching element along with both iterator's remaining elements
    /// starting with the first mis-match.
    FirstMismatch(usize, PutBack<I>, PutBack<J>),
    /// The total number of elements that were in `J` along with the remaining elements of `I`.
    Shorter(usize, PutBack<I>),
    /// The total number of elements that were in `I` along with the remaining elements of `J`.
    Longer(usize, PutBack<J>),
}

/// Compares every element yielded by both `i` and `j` in lock-step and returns a `Diff` which
/// describes how `j` differs from `i`.
///
/// If the number of elements yielded by `j` is less than the number of elements yielded by `i`,
/// the number of `j` elements yielded will be returned along with `i`'s remaining elements as
/// `Diff::Shorter`.
///
/// If the two elements of a step differ, the index of those elements along with the remaining
/// elements of both `i` and `j` are returned as `Diff::FirstMismatch`.
///
/// If `i` becomes exhausted before `j` becomes exhausted, the number of elements in `i` along with
/// the remaining `j` elements will be returned as `Diff::Longer`.
pub fn diff<I, J>(i: I, j: J) -> Option<Diff<I::IntoIter, J::IntoIter>>
    where I: IntoIterator,
          J: IntoIterator,
          I::Item: PartialEq<J::Item>,
{
    diff_internal(i, j, |ie, je| ie != je)
}

/// Similar to [`diff`](./fn.diff.html), however expects `i` to yield references to its elements.
///
/// This function is useful for caching some iterator `j` in some sequential collection without
/// requiring `j` to be `Clone` in order to compare it to the collection before determining if the
/// collection needs to be updated. The function returns as soon as a difference is found,
/// producing a `Diff` that provides the data necessary to update the collection without ever
/// requiring `J` to be `Clone`. This allows for efficiently caching iterators like `Map` or
/// `Filter` that do not implement `Clone`.
///
/// See [`copy_on_diff`](./fn.copy_on_diff.html) for an application of `diff`.
pub fn diff_by_ref<'a, I, J>(i: I, j: J) -> Option<Diff<I::IntoIter, J::IntoIter>>
    where I: IntoIterator<Item=&'a J::Item>,
          J: IntoIterator,
          J::Item: PartialEq + 'a,
{
    diff_internal(i, j, |ie, je| *ie != je)
}

// Compares every element yielded by both `i` and `j`  with the given `is_diff` function in
// lock-step.
//
// Returns a `Diff` which describes how `j` differs from `i`.
fn diff_internal<I, J, F>(i: I, j: J, is_diff: F) -> Option<Diff<I::IntoIter, J::IntoIter>>
    where I: IntoIterator,
          J: IntoIterator,
          F: Fn(&I::Item, &J::Item) -> bool,
{
    let mut i = i.into_iter();
    let mut j = j.into_iter();
    let mut idx = 0;
    while let Some(i_elem) = i.next() {
        match j.next() {
            None => return Some(Diff::Shorter(idx, PutBack::value(i_elem, i))),
            Some(j_elem) => if is_diff(&i_elem, &j_elem) {
                let remaining_i = PutBack::value(i_elem, i);
                let remaining_j = PutBack::value(j_elem, j);
                return Some(Diff::FirstMismatch(idx, remaining_i, remaining_j));
            },
        }
        idx += 1;
    }
    j.next().map(|j_elem| Diff::Longer(idx, PutBack::value(j_elem, j)))
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
    match diff_by_ref(c_iter.clone(), update.into_iter()) {
        Some(diff) => match diff {
            Diff::FirstMismatch(idx, _, mismatch) =>
                Cow::Owned(c_iter.take(idx).cloned().chain(mismatch).collect()),
            Diff::Longer(_, remaining) =>
                Cow::Owned(c_iter.cloned().chain(remaining).collect()),
            Diff::Shorter(num_update, _) =>
                Cow::Owned(c_iter.cloned().take(num_update).collect()),
        },
        None => Cow::Borrowed(collection),
    }
}
