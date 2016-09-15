//! Free functions that create iterator adaptors or call iterator methods.
//!
//! The benefit of free functions is that they accept any `IntoIterator` as
//! argument, so the resulting code may be easier to read.

use std::fmt::Display;
use std::iter::{self, Zip};
use {
    Interleave,
    Itertools,
    KMerge,
    Merge,
    ZipEq,
    RcIter,
};

/// Iterate `iterable` with a running index.
///
/// `IntoIterator` enabled version of `.enumerate()`.
///
/// ```
/// use itertools::enumerate;
///
/// for (i, elt) in enumerate(&[1, 2, 3]) {
///     /* loop body */
/// }
/// ```
pub fn enumerate<I>(iterable: I) -> iter::Enumerate<I::IntoIter>
    where I: IntoIterator
{
    iterable.into_iter().enumerate()
}

/// Iterate `iterable` in reverse.
///
/// `IntoIterator` enabled version of `.rev()`.
///
/// ```
/// use itertools::rev;
///
/// for elt in rev(&[1, 2, 3]) {
///     /* loop body */
/// }
/// ```
pub fn rev<I>(iterable: I) -> iter::Rev<I::IntoIter>
    where I: IntoIterator,
          I::IntoIter: DoubleEndedIterator
{
    iterable.into_iter().rev()
}

/// Iterate `i` and `j` in lock step.
///
/// `IntoIterator` enabled version of `i.zip(j)`.
///
/// ```
/// use itertools::free::zip;
///
/// let data = [1, 2, 3, 4, 5];
/// for (a, b) in zip(&data, &data[1..]) {
///     /* loop body */
/// }
/// ```
pub fn zip<I, J>(i: I, j: J) -> Zip<I::IntoIter, J::IntoIter>
    where I: IntoIterator,
          J: IntoIterator
{
    i.into_iter().zip(j)
}

/// Iterate `i` and `j` in lock step.
///
/// **Panics** if the iterators are not of the same length.
///
/// `IntoIterator` enabled version of `i.zip_eq(j)`.
///
/// ```
/// use itertools::free::zip_eq;
///
/// let data = [1, 2, 3, 4, 5];
/// for (a, b) in zip_eq(&data[..data.len() - 1], &data[1..]) {
///     /* loop body */
/// }
/// ```
pub fn zip_eq<I, J>(i: I, j: J) -> ZipEq<I::IntoIter, J::IntoIter>
    where I: IntoIterator,
          J: IntoIterator
{
    i.into_iter().zip_eq(j)
}

/// Create an iterator that first iterates `i` and then `j`.
///
/// `IntoIterator` enabled version of `i.chain(j)`.
///
/// ```
/// use itertools::free::chain;
///
/// for elt in chain(&[1, 2, 3], &[4]) {
///     /* loop body */
/// }
/// ```
pub fn chain<I, J>(i: I, j: J) -> iter::Chain<<I as IntoIterator>::IntoIter, <J as IntoIterator>::IntoIter>
    where I: IntoIterator,
          J: IntoIterator<Item = I::Item>
{
    i.into_iter().chain(j)
}

/// Create an iterator that clones each element from &T to T
///
/// `IntoIterator` enabled version of `i.cloned()`.
///
/// ```
/// use itertools::free::cloned;
///
/// assert_eq!(cloned(b"abc").next(), Some(b'a'));
/// ```
pub fn cloned<'a, I, T: 'a>(iterable: I) -> iter::Cloned<I::IntoIter>
    where I: IntoIterator<Item=&'a T>,
          T: Clone,
{
    iterable.into_iter().cloned()
}

/// Perform a fold operation over the iterable.
///
/// `IntoIterator` enabled version of `i.fold(init, f)`
///
/// ```
/// use itertools::free::fold;
///
/// assert_eq!(fold(&[1., 2., 3.], 0., |a, &b| f32::max(a, b)), 3.);
/// ```
pub fn fold<I, B, F>(iterable: I, init: B, f: F) -> B
    where I: IntoIterator,
          F: FnMut(B, I::Item) -> B
{
    iterable.into_iter().fold(init, f)
}

/// Test whether the predicate holds for all elements in the iterable.
///
/// `IntoIterator` enabled version of `i.all(f)`
///
/// ```
/// use itertools::free::all;
///
/// assert!(all(&[1, 2, 3], |elt| *elt > 0));
/// ```
pub fn all<I, F>(iterable: I, f: F) -> bool
    where I: IntoIterator,
          F: FnMut(I::Item) -> bool
{
    iterable.into_iter().all(f)
}

/// Test whether the predicate holds for any elements in the iterable.
///
/// `IntoIterator` enabled version of `i.any(f)`
///
/// ```
/// use itertools::free::any;
///
/// assert!(any(&[0, -1, 2], |elt| *elt > 0));
/// ```
pub fn any<I, F>(iterable: I, f: F) -> bool
    where I: IntoIterator,
          F: FnMut(I::Item) -> bool
{
    iterable.into_iter().any(f)
}

/// Return the maximum value of the iterable.
///
/// `IntoIterator` enabled version of `i.max()`.
///
/// ```
/// use itertools::free::max;
///
/// assert_eq!(max(0..10), Some(9));
/// ```
pub fn max<I>(iterable: I) -> Option<I::Item>
    where I: IntoIterator,
          I::Item: Ord
{
    iterable.into_iter().max()
}

/// Return the minimum value of the iterable.
///
/// `IntoIterator` enabled version of `i.min()`.
///
/// ```
/// use itertools::free::min;
///
/// assert_eq!(min(0..10), Some(0));
/// ```
pub fn min<I>(iterable: I) -> Option<I::Item>
    where I: IntoIterator,
          I::Item: Ord
{
    iterable.into_iter().min()
}

/// Create an iterator that interleaves elements in `i` and `j`.
///
/// `IntoIterator` enabled version of `i.interleave(j)`.
///
/// ```
/// use itertools::free::interleave;
///
/// for elt in interleave(&[1, 2, 3], &[2, 3, 4]) {
///     /* loop body */
/// }
/// ```
pub fn interleave<I, J>(i: I, j: J) -> Interleave<<I as IntoIterator>::IntoIter, <J as IntoIterator>::IntoIter>
    where I: IntoIterator,
          J: IntoIterator<Item = I::Item>
{
    i.into_iter().interleave(j)
}

/// Create an iterator that merges elements in `i` and `j`.
///
/// `IntoIterator` enabled version of `i.merge(j)`.
///
/// ```
/// use itertools::free::merge;
///
/// for elt in merge(&[1, 2, 3], &[2, 3, 4]) {
///     /* loop body */
/// }
/// ```
pub fn merge<I, J>(i: I, j: J) -> Merge<<I as IntoIterator>::IntoIter, <J as IntoIterator>::IntoIter>
    where I: IntoIterator,
          J: IntoIterator<Item = I::Item>,
          I::Item: PartialOrd
{
    i.into_iter().merge(j)
}

/// Create an iterator that merges elements of the contained iterators.
///
/// Equivalent to `i.into_iter().kmerge()`.
///
/// ```
/// use itertools::free::kmerge;
///
/// for elt in kmerge(vec![vec![0, 2, 4], vec![1, 3, 5], vec![6, 7]]) {
///     /* loop body */
/// }
/// ```
pub fn kmerge<I>(i: I) -> KMerge<<<I as IntoIterator>::Item as IntoIterator>::IntoIter>
    where I: IntoIterator,
          I::Item: IntoIterator,
          <<I as IntoIterator>::Item as IntoIterator>::Item: Ord,
{
    i.into_iter().kmerge()
}

/// Combine all iterator elements into one String, seperated by `sep`.
///
/// `IntoIterator` enabled version of `iterable.join(sep)`.
///
/// ```
/// use itertools::free::join;
///
/// assert_eq!(join(&[1, 2, 3], ", "), "1, 2, 3");
/// ```
pub fn join<I>(iterable: I, sep: &str) -> String
    where I: IntoIterator,
          I::Item: Display
{
    iterable.into_iter().join(sep)
}

/// Collect all the iterable's elements into a sorted vector in ascending order.
///
/// `IntoIterator` enabled version of `iterable.sorted()`.
///
/// ```
/// use itertools::free::sorted;
/// use itertools::assert_equal;
///
/// assert_equal(sorted("rust".chars()), "rstu".chars());
/// ```
pub fn sorted<I>(iterable: I) -> Vec<I::Item>
    where I: IntoIterator,
          I::Item: Ord
{
    iterable.into_iter().sorted()
}

/// Return an iterator inside a `Rc<RefCell<_>>` wrapper.
///
/// The returned `RcIter` can be cloned, and each clone will refer back to the
/// same original iterator.
///
/// `RcIter` allows doing interesting things like using `.zip()` on an iterator with
/// itself, at the cost of runtime borrow checking.
/// (If it is not obvious: this has a performance penalty.)
///
/// Iterator element type is `Self::Item`.
///
/// ```
/// use itertools::free::rciter;
///
/// let mut rit = rciter(0..9);
/// let mut z = rit.clone().zip(rit.clone());
/// assert_eq!(z.next(), Some((0, 1)));
/// assert_eq!(z.next(), Some((2, 3)));
/// assert_eq!(z.next(), Some((4, 5)));
/// assert_eq!(rit.next(), Some(6));
/// assert_eq!(z.next(), Some((7, 8)));
/// assert_eq!(z.next(), None);
/// ```
///
/// **Panics** in iterator methods if a borrow error is encountered,
/// but it can only happen if the `RcIter` is reentered in for example `.next()`,
/// i.e. if it somehow participates in an “iterator knot” where it is an adaptor of itself.
pub fn rciter<I>(iterable: I) -> RcIter<I::IntoIter>
    where I: IntoIterator
{
    RcIter::new(iterable.into_iter())
}
