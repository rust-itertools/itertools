#![feature(old_impl_check)]
#![feature(unboxed_closures)]
#![crate_name="itertools"]
#![crate_type="dylib"]
#![allow(unstable)]

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
};
pub use intersperse::Intersperse;
pub use islice::{ISlice};
pub use rciter::RcIter;
pub use stride::Stride;
pub use stride::StrideMut;
pub use tee::Tee;
pub use times::Times;
pub use times::times;
pub use linspace::{linspace, Linspace};
pub use zip::{ZipLongest, EitherOrBoth};
pub use ziptuple::Zip;
mod adaptors;
mod intersperse;
mod islice;
mod linspace;
pub mod misc;
mod rciter;
mod stride;
mod tee;
mod times;
mod zip;
mod ziptuple;

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
pub macro_rules! iproduct {
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

#[deprecated="izip!() is deprecated, use Zip::new instead"]
#[macro_export]
/// *This macro is deprecated, use* **Zip::new** *instead.*
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
/// for (i, a, b) in izip!(range(0, 100), xs.mut_iter(), ys.iter()) {
///    *a = i ^ *b;
/// }
/// ```
pub macro_rules! izip {
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
/// let mut squares = icompr!(x * x, x, range(1, 100));
/// ```
///
/// **Note:** A Python like syntax of `<expression> for <pattern> in <iterator>` is
/// **not possible** with the stable macro rules since Rust 1.0.0-alpha.
#[macro_export]
pub macro_rules! icompr {
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

    /// Like regular `.map`, but using a simple function pointer instead,
    /// so that the resulting `FnMap` iterator value can be cloned.
    ///
    /// Iterator element type is `B`
    #[deprecated="Use libstd .map() instead"]
    fn fn_map<B>(self, map: fn(Self::Item) -> B) -> FnMap<Self::Item, B, Self> where
        Self: Sized
    {
        FnMap::new(self, map)
    }

    /// Alternate elements from two iterators until both
    /// are run out
    ///
    /// Iterator element type is `Self::Item`
    fn interleave<J: Iterator<Item=Self::Item>>(self, other: J) -> Interleave<Self, J> where
        Self: Sized
    {
        Interleave::new(self, other)
    }

    /// An iterator adaptor to insert a particular value
    /// between each element of the adapted iterator.
    ///
    /// Iterator element type is `Self::Item`
    fn intersperse(self, element: Self::Item) -> Intersperse<Self::Item, Self> where
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
    /// Iterator element type is `EitherOrBoth<Self::Item, B>`.
    #[inline]
    fn zip_longest<U: Iterator>(self, other: U) -> ZipLongest<Self, U> where
        Self: Sized,
    {
        ZipLongest::new(self, other)
    }

    /// Remove duplicates from sections of consecutive identical elements.
    /// If the iterator is sorted, all elements will be unique.
    ///
    /// Iterator element type is `Self::Item`.
    fn dedup(self) -> Dedup<Self::Item, Self> where
        Self: Sized,
    {
        Dedup::new(self)
    }

    /// A “meta iterator adaptor”. Its closure recives a reference to the iterator
    /// and may pick off as many elements as it likes, to produce the next iterator element.
    ///
    /// Iterator element type is `B`.
    ///
    /// ## Example
    ///
    /// ```
    /// # use itertools::Itertools;
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
    fn group_by<K, F: FnMut(&Self::Item) -> K>(self, key: F) -> GroupBy<Self::Item, K, Self, F> where
        Self: Sized,
    {
        GroupBy::new(self, key)
    }

    /// Split into an iterator pair that both yield all elements from
    /// the original iterator.
    ///
    /// Iterator element type is `Self::Item`.
    ///
    /// ## Example
    /// ```
    /// # use itertools::Itertools;
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
    fn tee(self) -> (Tee<Self::Item, Self>, Tee<Self::Item, Self>) where
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
    /// ## Example
    /// ```
    /// use std::iter::repeat;
    /// use itertools::Itertools;
    ///
    /// let mut it = repeat('a').slice(..3);
    /// assert_eq!(it.count(), 3);
    /// ```
    fn slice<R: misc::GenericRange>(self, range: R) -> ISlice<Self> where
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
    /// ## Example
    ///
    /// ```
    /// use itertools::Itertools;
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
    ///
    /// Iterator element type is **Self::Item**.
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
    /// **Panics** if the step is 0.
    ///
    /// ## Example
    /// ```
    /// # #![feature(slicing_syntax)]
    /// # extern crate itertools;
    /// # fn main() {
    /// # use itertools::Itertools;
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

    // non-adaptor methods

    /// Find the position and value of the first element satisfying a predicate.
    fn find_position<P>(&mut self, mut pred: P) -> Option<(usize, Self::Item)> where
        P: FnMut(&Self::Item) -> bool,
    {
        let mut index = 0us;
        for elt in *self {
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

    /// Run the iterator, eagerly, to the end and consume all its elements.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use itertools::Itertools;
    ///
    /// let mut cnt = 0us;
    /// "hi".chars().map(|c| cnt += 1).drain();
    /// ```
    ///
    fn drain(&mut self)
    {
        for _ in *self { /* nothing */ }
    }

    /// Run the closure **f** eagerly on each element of the iterator.
    ///
    /// Consumes the iterator until its end.
    fn apply<F: FnMut(Self::Item)>(&mut self, mut f: F)
    {
        for elt in *self { f(elt) }
    }

    /// **.collec_vec()** is simply a type specialization of **.collect()**,
    /// for convenience.
    fn collect_vec(self) -> Vec<Self::Item> where
        Self: Sized,
    {
        self.collect()
    }

    /// Given two sorted iterators (this and `other`), return an iterator adapter yeilds elements
    /// from both iterators in sorted order.
    fn merge<B>(self, other: B) -> Merge<Self::Item, Self, B> where
        Self: Sized,
        B: Iterator<Item = Self::Item>,
        Self::Item: PartialOrd,
    {
        Merge::new(self, other)
    }
}

impl<T: ?Sized> Itertools for T where T: Iterator { }

/// Assign to each reference in `to` from `from`, stopping
/// at the shortest of the two iterators.
///
/// Return the number of elements written.
#[inline]
pub fn write<'a, A: 'a, I: Iterator<Item=&'a mut A>, J: Iterator<Item=A>>
    (mut to: I, mut from: J) -> usize
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
