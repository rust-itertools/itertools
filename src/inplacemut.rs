use std::vec::Vec;

pub enum Combine<T> {
    Keep(usize), // n >= 1
    InsertAndDrop(T, usize), // insert 1 element, remove n elements, n >= 1
}

#[derive(Debug, PartialEq)]
pub enum CombineError {
    ZeroForward, // not increasing index ends up looping forever
    OutOfRange, // tried to Drop/Keep more elements than were available
}

use std::ops::{Deref, DerefMut};
use std::slice::SliceExt;

/// Algorithms that do not require reallocation, since they can only shrink
/// the sequence
pub trait InplaceMappable<T> : Deref<Target = [T]> + DerefMut<Target = [T]> + Sized {

    fn len(&self) -> usize;
    fn swap(&mut self, i: usize, j: usize);
    fn truncate(self, newsize: usize) -> Self;

    /// Allows merging of elements after custom rules.
    /// Its closure recives a slice from the current location to the end
    /// and may skip (keep) any number of elements or yield a new element, dropping
    /// at least the current element or any number of elements.
    ///
    /// Iterator element type is **T**.
    ///
    /// ## Example
    ///
    /// ```
    /// use itertools::Combine::{InsertAndDrop, Keep};
    /// use itertools::InplaceMappable;
    /// use std::cmp::min;
    /// use Thing::*;
    ///
    /// #[derive(PartialEq, Debug)]
    /// enum Thing { A, B, AandB }
    ///
    /// fn main() {
    ///     let v = vec![A, B, A, A, A, B, B, A];
    ///     let v = v.combine(|arr|
    ///         match &arr[..min(2, arr.len())] {
    ///             [A, B] => InsertAndDrop(AandB, 2),
    ///             [..] => Keep(1)
    ///         }
    ///     );
    ///     assert_eq!(
    ///         vec![AandB, A, A, AandB, B, A],
    ///         v.unwrap()
    ///     );
    /// }
    /// ```
    ///
    /// No Element is moved twice.
    fn combine<P>(mut self, predicate : P)
                  -> Result<Self, CombineError>
               where P: Fn(&[T]) -> Combine<T> {
        fn check(i: usize, num: usize, n: usize) -> Result<(), CombineError> {
            if num == 0 { return Err(CombineError::ZeroForward); }
            if i + num > n { return Err(CombineError::OutOfRange); }
            Ok(())
        }
        let mut i = 0;
        for j in 0..self.len() {
            i = i + match predicate(&self[i..]) {
                Combine::InsertAndDrop(elem, n) => {
                    try!(check(i, n, self.len()));
                    self[j] = elem;
                    n
                },
                Combine::Keep(n) => {
                    try!(check(i, n, self.len()));
                    for k in i..(i+n) {
                        // move would be more efficient than swap self[j] = self[k];
                        // but this requires some thought due to unsafe code and
                        // leftover elements that need to be dropped (as well as unsafe truncate)
                        self.swap(j, k);
                    }
                    n
                }
            };
            if i == self.len() {
                return Ok(self.truncate(j + 1));
            }
        }
        Ok(self)
    }
}

impl<U> InplaceMappable<U> for Vec<U> {
    fn len(&self) -> usize {
        Vec::len(self)
    }
    fn swap(&mut self, i: usize, j: usize) {
        let tmp: &mut [U] = self;
        SliceExt::swap(tmp, i, j);
    }
    fn truncate(mut self, newsize: usize) -> Self {
        Vec::truncate(&mut self, newsize);
        self
    }
}

impl<'a, U> InplaceMappable<U> for &'a mut [U] {
    fn len(&self) -> usize {
        SliceExt::len(*self)
    }
    fn swap(&mut self, i: usize, j: usize) {
        SliceExt::swap(*self, i, j);
    }
    fn truncate(self, newsize: usize) -> Self {
        &mut self[0..newsize]
    }
}
