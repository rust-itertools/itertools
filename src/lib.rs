#![warn(missing_docs)]
#![cfg_attr(feature = "unstable", feature(core, zero_one))]
#![crate_name="itertools"]

//! Itertools — extra iterator adaptors, functions and macros
//!
//! To use the iterator methods in this crate, import the [**Itertools** trait](./trait.Itertools.html):
//!
//! ```ignore
//! use itertools::Itertools;
//! ```
//!
//! Some adaptors are just used directly like regular structs,
//! for example [**PutBack**](./struct.PutBack.html), [**Zip**](./struct.Zip.html), [**Stride**](./struct.Stride.html), [**StrideMut**](./struct.StrideMut.html).
//!
//! To use the macros in this crate, use the `#[macro_use]` attribute:
//!
//! ```ignore
//! #[macro_use]
//! extern crate itertools;
//! ```
//!
//! ## License
//! Dual-licensed to be compatible with the Rust project.
//!
//! Licensed under the Apache License, Version 2.0
//! http://www.apache.org/licenses/LICENSE-2.0 or the MIT license
//! http://opensource.org/licenses/MIT, at your
//! option. This file may not be copied, modified, or distributed
//! except according to those terms.
//!
//!

use std::fmt::Write;
use std::cmp::Ordering;

pub use adaptors::{
    Interleave,
    Product,
    PutBack,
    FnMap,
    Dedup,
    Batching,
    GroupBy,
    Step,
    Merge,
    MultiPeek,
};
#[cfg(feature = "unstable")]
pub use adaptors::EnumerateFrom;
pub use intersperse::Intersperse;
pub use islice::{ISlice};
pub use repeatn::RepeatN;
pub use rciter::RcIter;
pub use stride::Stride;
pub use stride::StrideMut;
pub use tee::Tee;
pub use times::Times;
pub use times::times;
pub use linspace::{linspace, Linspace};
pub use zip::{ZipLongest, EitherOrBoth};
pub use ziptuple::{Zip};
#[cfg(feature = "unstable")]
pub use ziptrusted::{ZipTrusted, TrustedIterator};
mod adaptors;
mod intersperse;
mod islice;
mod linspace;
pub mod misc;
mod rciter;
mod repeatn;
mod stride;
mod tee;
mod times;
mod zip;
mod ziptuple;
#[cfg(feature = "unstable")]
mod ziptrusted;

#[macro_export]
/// Create an iterator over the “cartesian product” of iterators.
///
/// Iterator element type is like **(A, B, ..., E)** if formed
/// from iterators **(I, J, ..., M)** with element types **I::Item = A**, **J::Item = B**, etc.
///
/// ## Example
///
/// ```ignore
/// #[macro_use]
/// extern crate itertools;
/// # fn main() {
/// // Iterate over the coordinates of a 4 x 4 x 4 grid
/// // from (0, 0, 0), (0, 0, 1), .., (0, 1, 0), (0, 1, 1), .. etc until (3, 3, 3)
/// for (i, j, k) in iproduct!(0..4, 0..4, 0..4) {
///    // ..
/// }
/// # }
/// ```
macro_rules! iproduct {
    ($I:expr) => (
        ($I)
    );
    ($I:expr, $J:expr) => (
        {
            let it = $crate::Product::new($I, $J);
            it
        }
    );
    ($I:expr, $J:expr, $($K:expr),+) => (
        {
            let it = $crate::Product::new($I, $J);
            $(
                let it = $crate::misc::FlatTuples::new($crate::Product::new(it, $K));
            )*
            it
        }
    );
}

#[macro_export]
/// **Deprecated: use *Zip::new* instead.**
///
/// Create an iterator running multiple iterators in lockstep.
///
/// The izip! iterator yields elements until any subiterator
/// returns `None`.
///
/// Iterator element type is like `(A, B, ..., E)` if formed
/// from iterators `(I, J, ..., M)` implementing `I: Iterator<A>`,
/// `J: Iterator<B>`, ..., `M: Iterator<E>`
///
/// ## Example
///
/// ```ignore
/// // Iterate over three sequences side-by-side
/// let mut xs = [0, 0, 0];
/// let ys = [72, 73, 74];
/// for (i, a, b) in izip!(0..100, xs.mut_iter(), ys.iter()) {
///    *a = i ^ *b;
/// }
/// ```
macro_rules! izip {
    ($I:expr) => (
        ($I)
    );
    (($I:expr),*) => (
        {
            $crate::Zip::new(($I),*)
        }
    );
}

/// `icompr` as in “iterator comprehension” allows creating a
/// mapped iterator with simple syntax, similar to set builder notation,
/// and directly inspired by Python. Supports an optional filter clause.
///
/// Syntax:
///
///  `icompr!(<expression>, <pattern>, <iterator>)`
///
/// or
///
///  `icompr!(<expression>, <pattern>, <iterator>, <expression>)`
///
/// Each element from the `<iterator>` expression is pattern matched
/// with the `<pattern>`, and the bound names are used to express the
/// mapped-to value.
///
/// Iterator element type is the type of `<expression>`
///
/// ## Example
///
/// ```ignore
/// let mut squares = icompr!(x * x, x, 1..100);
/// ```
///
/// **Note:** A Python like syntax of `<expression> for <pattern> in <iterator>` is
/// **not possible** with the stable macro rules since Rust 1.0.0-alpha.
#[macro_export]
macro_rules! icompr {
    ($r:expr, $x:pat, $J:expr, $pred:expr) => (
        ($J).filter_map(|$x| if $pred { Some($r) } else { None })
    );
    ($r:expr, $x:pat, $J:expr) => (
        ($J).filter_map(|$x| Some($r))
    );
}

/// Extra iterator methods for arbitrary iterators
pub trait Itertools : Iterator {
    // adaptors

    /// Like regular *.map()*, but using a simple function pointer instead,
    /// so that the resulting **FnMap** iterator value can be cloned.
    ///
    /// Iterator element type is **B**.
    fn fn_map<B>(self, map: fn(Self::Item) -> B) -> FnMap<B, Self> where
        Self: Sized
    {
        FnMap::new(self, map)
    }

    /// Alternate elements from two iterators until both
    /// are run out
    ///
    /// Iterator element type is **Self::Item**.
    fn interleave<J>(self, other: J) -> Interleave<Self, J> where
        J: Iterator<Item=Self::Item>,
        Self: Sized
    {
        Interleave::new(self, other)
    }

    /// An iterator adaptor to insert a particular value
    /// between each element of the adapted iterator.
    ///
    /// Iterator element type is **Self::Item**.
    fn intersperse(self, element: Self::Item) -> Intersperse<Self> where
        Self: Sized,
        Self::Item: Clone
    {
        Intersperse::new(self, element)
    }

    /// Creates an iterator which iterates over both this and the specified
    /// iterators simultaneously, yielding pairs of two optional elements.
    /// When both iterators return None, all further invocations of next() will
    /// return None.
    ///
    /// # Example
    ///
    /// ```rust
    /// use itertools::EitherOrBoth::{Both, Right};
    /// use itertools::Itertools;
    /// let mut it = (0..1).zip_longest(1..3);
    /// assert_eq!(it.next(), Some(Both(0, 1)));
    /// assert_eq!(it.next(), Some(Right(2)));
    /// assert_eq!(it.next(), None);
    /// ```
    ///
    /// Iterator element type is **EitherOrBoth\<Self::Item, B\>**.
    #[inline]
    fn zip_longest<U>(self, other: U) -> ZipLongest<Self, U> where
        U: Iterator,
        Self: Sized,
    {
        ZipLongest::new(self, other)
    }

    /// Remove duplicates from sections of consecutive identical elements.
    /// If the iterator is sorted, all elements will be unique.
    ///
    /// Iterator element type is **Self::Item**.
    fn dedup(self) -> Dedup<Self> where
        Self: Sized,
    {
        Dedup::new(self)
    }

    /// A “meta iterator adaptor”. Its closure recives a reference to the iterator
    /// and may pick off as many elements as it likes, to produce the next iterator element.
    ///
    /// Iterator element type is **B**.
    ///
    /// ## Example
    ///
    /// ```
    /// use itertools::Itertools;
    /// // An adaptor that gathers elements up in pairs
    /// let mut pit = (0..4).batching(|mut it| {
    ///            match it.next() {
    ///                None => None,
    ///                Some(x) => match it.next() {
    ///                    None => None,
    ///                    Some(y) => Some((x, y)),
    ///                }
    ///            }
    ///        });
    /// assert_eq!(pit.next(), Some((0, 1)));
    /// assert_eq!(pit.next(), Some((2, 3)));
    /// assert_eq!(pit.next(), None);
    /// ```
    ///
    fn batching<B, F: FnMut(&mut Self) -> Option<B>>(self, f: F) -> Batching<Self, F> where
        Self: Sized,
    {
        Batching::new(self, f)
    }

    /// Group iterator elements. Consecutive elements that map to the same key (“runs”),
    /// are returned as the iterator elements of **GroupBy**.
    ///
    /// Iterator element type is **(K, Vec\<Self::Item\>)**
    fn group_by<K, F: FnMut(&Self::Item) -> K>(self, key: F) -> GroupBy<K, Self, F> where
        Self: Sized,
    {
        GroupBy::new(self, key)
    }

    /// Split into an iterator pair that both yield all elements from
    /// the original iterator.
    ///
    /// Iterator element type is **Self::Item**.
    ///
    /// ## Example
    /// ```
    /// use itertools::Itertools;
    /// let xs = vec![0, 1, 2, 3];
    ///
    /// let (mut t1, mut t2) = xs.into_iter().tee();
    /// assert_eq!(t1.next(), Some(0));
    /// assert_eq!(t1.next(), Some(1));
    /// assert_eq!(t2.next(), Some(0));
    /// assert_eq!(t1.next(), Some(2));
    /// assert_eq!(t1.next(), Some(3));
    /// assert_eq!(t1.next(), None);
    /// assert_eq!(t2.next(), Some(1));
    /// ```
    fn tee(self) -> (Tee<Self>, Tee<Self>) where
        Self: Sized,
        Self::Item: Clone
    {
        tee::new(self)
    }

    /// Return a sliced iterator.
    ///
    /// **Note:** slicing an iterator is not constant time, and much less efficient than
    /// slicing for example a vector.
    ///
    /// Iterator element type is **Self::Item**.
    ///
    /// ## Example
    /// ```
    /// use std::iter::repeat;
    /// use itertools::Itertools;
    ///
    /// let mut it = repeat('a').slice(..3);
    /// assert_eq!(it.count(), 3);
    /// ```
    fn slice<R>(self, range: R) -> ISlice<Self> where
        R: misc::GenericRange,
        Self: Sized,
    {
        ISlice::new(self, range)
    }

    /// Return an iterator inside a **Rc\<RefCell\<_\>\>** wrapper.
    ///
    /// The returned **RcIter** can be cloned, and each clone will refer back to the
    /// same original iterator.
    ///
    /// **RcIter** allows doing interesting things like using **.zip()** on an iterator with
    /// itself, at the cost of runtime borrow checking.
    /// (If it is not obvious: this has a performance penalty.)
    ///
    /// Iterator element type is **Self::Item**.
    ///
    /// ## Example
    ///
    /// ```
    /// use itertools::Itertools;
    ///
    /// let mut rit = (0..9).into_rc();
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
    /// but it can only happen if the RcIter is reentered in for example **.next()**,
    /// i.e. if it somehow participates in an "iterator knot" where it is an adaptor of itself.
    fn into_rc(self) -> RcIter<Self> where
        Self: Sized,
    {
        RcIter::new(self)
    }

    /// Return an iterator adaptor that steps **n** elements in the base iterator
    /// for each iteration.
    ///
    /// The iterator steps by yielding the next element from the base iterator,
    /// then skipping forward **n - 1** elements.
    ///
    /// Iterator element type is **Self::Item**.
    ///
    /// **Panics** if the step is 0.
    ///
    /// ## Example
    /// ```
    /// # extern crate itertools;
    /// # fn main() {
    /// use itertools::Itertools;
    ///
    /// let mut it = (0..8).step(3);
    /// assert_eq!(it.next(), Some(0));
    /// assert_eq!(it.next(), Some(3));
    /// assert_eq!(it.next(), Some(6));
    /// assert_eq!(it.next(), None);
    /// # }
    /// ```
    fn step(self, n: usize) -> Step<Self> where
        Self: Sized,
    {
        Step::new(self, n)
    }

    /// Return an iterator adaptor that merges the two base iterators in ascending order.
    /// If both base iterators are sorted (ascending), the result is sorted.
    ///
    /// Iterator element type is **Self::Item**.
    ///
    /// ## Example
    /// ```
    /// use itertools::Itertools;
    ///
    /// let a = (0..10).step(2);
    /// let b = (1..10).step(3);
    /// let mut it = a.merge(b);
    /// assert_eq!(it.next(), Some(0));
    /// assert_eq!(it.next(), Some(1));
    /// assert_eq!(it.next(), Some(2));
    /// assert_eq!(it.next(), Some(4));
    /// assert_eq!(it.next(), Some(4));
    /// assert_eq!(it.next(), Some(6));
    /// ```
    fn merge<J>(self, other: J)
        -> Merge<Self, J, fn(&Self::Item, &Self::Item) -> Ordering> where
        Self: Sized,
        Self::Item: PartialOrd,
        J: Iterator<Item=Self::Item>,
    {
        fn wrapper<A: PartialOrd>(a: &A, b: &A) -> Ordering { 
            a.partial_cmp(b).unwrap_or(Ordering::Less) 
        };
        self.merge_by(other, wrapper)
    }

    /// Return an iterator adaptor that merges the two base iterators in an order.
    /// This is much like merge_by but allows for descending orders or sorting tuples.
    /// This can be especially useful walking a BTreeMap structure.
    ///
    /// Iterator element type is **Self::Item**.
    ///
    /// ## Example
    /// ```
    /// use itertools::Itertools;
    ///
    /// let a = (0..).zip("bc".chars());
    /// let b = (0..).zip("ad".chars());
    /// let mut it = a.merge_by(b, |x, y| x.1.cmp(&y.1));
    /// assert_eq!(it.next(), Some((0, 'a')));
    /// assert_eq!(it.next(), Some((0, 'b')));
    /// assert_eq!(it.next(), Some((1, 'c')));
    /// assert_eq!(it.next(), Some((1, 'd')));
    /// assert_eq!(it.next(), None);
    /// ```

    fn merge_by<J, F>(self, other: J, cmp: F) -> Merge<Self, J, F> where
        Self: Sized,
        J: Iterator<Item=Self::Item>,
        F: FnMut(&Self::Item, &Self::Item) -> Ordering
    {
        Merge::new(self, other, cmp)
    }

    /// Return an iterator adaptor that iterates over the cartesian product of
    /// the element sets of two iterators **self** and **J**.
    ///
    /// Iterator element type is **(Self::Item, J::Item)**.
    ///
    /// ```
    /// use itertools::Itertools;
    ///
    /// let mut it = (0..2).cartesian_product("αβ".chars());
    /// assert_eq!(it.next(), Some((0, 'α')));
    /// assert_eq!(it.next(), Some((0, 'β')));
    /// assert_eq!(it.next(), Some((1, 'α')));
    /// assert_eq!(it.next(), Some((1, 'β')));
    /// assert_eq!(it.next(), None);
    /// ```
    fn cartesian_product<J>(self, other: J) -> Product<Self, J> where
        Self: Sized,
        Self::Item: Clone,
        J: Clone + Iterator,
    {
        Product::new(self, other)
    }

    /// Return an iterator adaptor that enumerates the iterator elements,
    /// starting from **start** and incrementing by one.
    ///
    /// Iterator element type is **(K, Self::Item)**.
    ///
    /// ```
    /// use itertools::Itertools;
    ///
    /// assert_eq!(
    ///     "αβγ".chars().enumerate_from(-10i8).collect_vec(),
    ///     [(-10, 'α'), (-9, 'β'), (-8, 'γ')]
    /// );
    /// ```
    #[cfg(feature = "unstable")]
    fn enumerate_from<K>(self, start: K) -> EnumerateFrom<Self, K> where
        Self: Sized,
    {
        EnumerateFrom::new(self, start)
    }

    /// Returns an iterator adapter that allows peeking multiple values.
    ///
    /// After a call to *.next()* the peeking cursor gets resetted.
    ///
    /// ## Example
    ///
    /// ```
    /// use itertools::Itertools;
    ///
    /// let nums = vec![1u8,2,3,4,5];
    /// let mut peekable = nums.into_iter().multipeek();
    /// assert_eq!(peekable.peek(), Some(&1));
    /// assert_eq!(peekable.peek(), Some(&2));
    /// assert_eq!(peekable.peek(), Some(&3));
    /// assert_eq!(peekable.next(), Some(1));
    /// assert_eq!(peekable.peek(), Some(&2));
    /// ```
    fn multipeek(self) -> MultiPeek<Self> where
        Self: Sized
    {
        MultiPeek::new(self)
    }

    // non-adaptor methods

    /// Find the position and value of the first element satisfying a predicate.
    fn find_position<P>(&mut self, mut pred: P) -> Option<(usize, Self::Item)> where
        P: FnMut(&Self::Item) -> bool,
    {
        let mut index = 0usize;
        for elt in self {
            if pred(&elt) {
                return Some((index, elt))
            }
            index += 1;
        }
        None
    }

    /// Consume the first **n** elements of the iterator eagerly.
    ///
    /// Return actual number of elements consumed,
    /// until done or reaching the end.
    fn dropn(&mut self, mut n: usize) -> usize {
        let start = n;
        while n > 0 {
            match self.next() {
                Some(..) => n -= 1,
                None => break
            }
        }
        start - n
    }

    /// Consume the first **n** elements from the iterator eagerly,
    /// and return the same iterator again.
    ///
    /// It works similarly to **.skip(n)** except it is eager and
    /// preserves the iterator type.
    fn dropping(mut self, n: usize) -> Self where
        Self: Sized,
    {
        self.dropn(n);
        self
    }

    /// **Deprecated: because of a name clash, use .count() or .foreach() instead as appropriate.**
    ///
    /// Run the iterator, eagerly, to the end and consume all its elements.
    ///
    /// ## Example
    ///
    /// ```
    /// use itertools::Itertools;
    ///
    /// let mut cnt = 0;
    /// "hi".chars().map(|c| cnt += 1).drain();
    /// ```
    ///
    fn drain(&mut self)
    {
        for _ in self { /* nothing */ }
    }

    /// **Deprecated: Use *.foreach()* instead.**
    ///
    /// Run the closure **f** eagerly on each element of the iterator.
    ///
    /// Consumes the iterator until its end.
    fn apply<F>(&mut self, f: F) where
        F: FnMut(Self::Item),
    {
        self.foreach(f)
    }

    /// Run the closure **f** eagerly on each element of the iterator.
    ///
    /// Consumes the iterator until its end.
    fn foreach<F>(&mut self, mut f: F) where
        F: FnMut(Self::Item),
    {
        for elt in self { f(elt) }
    }

    /// **.collect_vec()** is simply a type specialization of **.collect()**,
    /// for convenience.
    fn collect_vec(self) -> Vec<Self::Item> where
        Self: Sized,
    {
        self.collect()
    }

    /// Assign to each reference in **self** from the **from** iterator,
    /// stopping at the shortest of the two iterators.
    ///
    /// The **from** iterator is queried for its next element before the **self**
    /// iterator, and if either is exhausted the method is done.
    ///
    /// Return the number of elements written.
    ///
    /// ## Example
    /// ```
    /// use itertools::Itertools;
    ///
    /// let mut xs = [0; 4];
    /// xs.iter_mut().set_from(1..);
    /// assert_eq!(xs, [1, 2, 3, 4]);
    /// ```
    #[inline]
    fn set_from<'a, A: 'a, J>(&mut self, from: J) -> usize where
        Self: Iterator<Item=&'a mut A>,
        J: Iterator<Item=A>,
    {
        let mut count = 0;
        for elt in from {
            match self.next() {
                None => break,
                Some(ptr) => *ptr = elt
            }
            count += 1;
        }
        count
    }

    /// **Deprecated: Use *.join()* instead, it's more efficient.**.
    ///
    /// Convert each element to String before joining them all together.
    ///
    /// Like *.join()*, but converts each element to **String** explicitly first.
    ///
    /// ## Example
    ///
    /// ```
    /// use itertools::Itertools;
    ///
    /// assert_eq!([1, 2, 3].iter().to_string_join(", "), "1, 2, 3");
    /// ```
    fn to_string_join(&mut self, sep: &str) -> String where
        Self::Item: ToString,
    {
        self.map(|elt| elt.to_string()).join(sep)
    }

    /// Combine all iterator elements into one String, seperated by **sep**.
    ///
    /// Use the **Display** implementation of each element.
    ///
    /// ## Example
    ///
    /// ```
    /// use itertools::Itertools;
    ///
    /// assert_eq!(["a", "b", "c"].iter().join(", "), "a, b, c");
    /// assert_eq!([1, 2, 3].iter().join(", "), "1, 2, 3");
    /// ```
    fn join(&mut self, sep: &str) -> String where
        Self::Item: std::fmt::Display,
    {
        match self.next() {
            None => String::new(),
            Some(first_elt) => {
                // estimate lower bound of capacity needed
                let (lower, _) = self.size_hint();
                let mut result = String::with_capacity(sep.len() * lower);
                write!(&mut result, "{}", first_elt).unwrap();
                for elt in self {
                    result.push_str(sep);
                    write!(&mut result, "{}", elt).unwrap();
                }
                result
            }
        }
    }

    /// Fold **Result** values from an iterator.
    ///
    /// Only **Ok** values are folded. If no error is encountered, the folded
    /// value is returned inside **Ok**. Otherwise and the operation terminates
    /// and returns the first error it encounters. No iterator elements are 
    /// consumed after the first error.
    ///
    /// ## Example
    ///
    /// ```
    /// use std::ops::Add;
    /// use itertools::Itertools;
    ///
    /// let values = [1, 2, -2, -1, 2, 1];
    /// assert_eq!(
    ///     values.iter()
    ///         .map(Ok::<_, ()>)
    ///         .fold_results(0, Add::add),
    ///     Ok(3)
    /// );
    /// assert!(
    ///     values.iter()
    ///         .map(|&x| if x >= 0 { Ok(x) } else { Err("Negative number") })
    ///         .fold_results(0, Add::add)
    ///         .is_err()
    /// );
    /// ```
    fn fold_results<A, E, B, F>(&mut self, mut start: B, mut f: F) -> Result<B, E> where
        Self: Iterator<Item=Result<A, E>>,
        F: FnMut(B, A) -> B,
    {
        for elt in self {
            match elt {
                Ok(v) => start = f(start, v),
                Err(u) => return Err(u),
            }
        }
        Ok(start)
    }

}

impl<T: ?Sized> Itertools for T where T: Iterator { }

/// **Deprecated: Use *.set_from()* instead**.
///
/// Assign to each reference in `to` from `from`, stopping
/// at the shortest of the two iterators.
///
/// Return the number of elements written.
#[inline]
pub fn write<'a, A: 'a, I, J>(mut to: I, from: J) -> usize where
    I: Iterator<Item=&'a mut A>,
    J: Iterator<Item=A>
{
    let mut count = 0;
    for elt in from {
        match to.next() {
            None => break,
            Some(ptr) => *ptr = elt
        }
        count += 1;
    }
    count
}
