#![warn(missing_docs)]
#![cfg_attr(feature = "unstable", feature(core, zero_one))]
#![crate_name="itertools"]

//! Itertools — extra iterator adaptors, functions and macros.
//!
//! To use the iterator methods in this crate, import the [**Itertools** trait](./trait.Itertools.html):
//!
//! ```ignore
//! use itertools::Itertools;
//! ```
//!
//! Some iterators or adaptors are used directly like regular structs, for example
//! [**PutBack**](./struct.PutBack.html), [**Zip**](./struct.Zip.html),
//! [**Stride**](./struct.Stride.html), [**StrideMut**](./struct.StrideMut.html).
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

use std::iter::{self, IntoIterator};
use std::fmt::Write;
use std::cmp::Ordering;

pub use adaptors::{
    Interleave,
    Product,
    PutBack,
    PutBackN,
    FnMap,
    Batching,
    GroupBy,
    Step,
    Merge,
    MultiPeek,
    TakeWhileRef,
    Coalesce,
    CoalesceFn,
    Combinations,
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
pub use sources::{
    RepeatCall,
};
pub use zip_longest::{ZipLongest, EitherOrBoth};
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
mod sources;
pub mod size_hint;
mod stride;
mod tee;
mod times;
mod zip_longest;
mod ziptuple;
#[cfg(feature = "unstable")]
mod ziptrusted;

/// The function pointer map iterator created with *.map_fn()*.
pub type MapFn<I, B> where I: Iterator = iter::Map<I, fn(I::Item) -> B>;

/// An ascending order merge iterator created with *.merge()*.
pub type MergeAscend<I, J> where I: Iterator = Merge<I, J, fn(&I::Item, &I::Item) -> Ordering>;

#[macro_export]
/// Create an iterator over the “cartesian product” of iterators.
///
/// Iterator element type is like **(A, B, ..., E)** if formed
/// from iterators **(I, J, ..., M)** with element types **I::Item = A**, **J::Item = B**, etc.
///
/// ## Example
///
/// ```
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
        (::std::iter::IntoIterator::into_iter($I))
    );
    ($I:expr, $J:expr) => (
        $crate::Product::new(iproduct!($I), iproduct!($J))
    );
    ($I:expr, $J:expr, $($K:expr),+) => (
        {
            let it = iproduct!($I, $J);
            $(
                let it = $crate::misc::FlatTuples::new(iproduct!(it, $K));
            )*
            it
        }
    );
}

#[macro_export]
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
/// ```
/// #[macro_use]
/// extern crate itertools;
/// # fn main() {
///
/// // Iterate over three sequences side-by-side
/// let mut xs = [0, 0, 0];
/// let ys = [69, 107, 101];
///
/// for (i, a, b) in izip!(0..100, &mut xs, &ys) {
///    *a = i ^ *b;
/// }
///
/// assert_eq!(xs, [69, 106, 103]);
/// # }
/// ```
macro_rules! izip {
    ($I:expr) => (
        (::std::iter::IntoIterator::into_iter($I))
    );
    ($($I:expr),*) => (
        {
            $crate::Zip::new(($(izip!($I)),*))
        }
    );
}

/// **Deprecated:** Will hopefully be replaced by a dedicated
/// syntax extension that can offer real convenient python-like syntax.
///
/// **Note:** A Python like syntax of `<expression> for <pattern> in <iterator>` is
/// **not possible** with the stable macro rules since Rust 1.0.0-alpha.
///
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
#[macro_export]
macro_rules! icompr {
    ($r:expr, $x:pat, $J:expr, $pred:expr) => (
        ($J).filter_map(|$x| if $pred { Some($r) } else { None })
    );
    ($r:expr, $x:pat, $J:expr) => (
        ($J).filter_map(|$x| Some($r))
    );
}

/// The trait **Itertools**: extra iterator adaptors and methods for iterators.
///
/// This trait defines a number of methods. They are divided into two groups:
///
/// * *Adaptors* take an interator and parameter as input, and return
/// a new iterator value. These are listed first in the trait. An example
/// of an adaptor is [*.interleave()*](#method.interleave)
///
/// * *Regular methods* are those that don't return iterators and instead
/// return a regular value of some other kind. [*.find_position()*](#method.find_position)
/// is an example and the first regular method in the list.
pub trait Itertools : Iterator {
    // adaptors

    /// Alternate elements from two iterators until both
    /// run out.
    ///
    /// Iterator element type is **Self::Item**.
    ///
    /// This iterator is *fused*.
    ///
    /// ## Example
    ///
    /// ```
    /// use itertools::Itertools;
    ///
    /// let it = (0..3).interleave(vec![7, 8]);
    /// assert!(itertools::equal(it, vec![0, 7, 1, 8, 2]));
    /// ```
    fn interleave<J>(self, other: J) -> Interleave<Self, J::IntoIter> where
        J: IntoIterator<Item=Self::Item>,
        Self: Sized
    {
        Interleave::new(self, other.into_iter())
    }

    /// An iterator adaptor to insert a particular value
    /// between each element of the adapted iterator.
    ///
    /// Iterator element type is **Self::Item**.
    ///
    /// This iterator is *fused*.
    ///
    /// ## Example
    ///
    /// ```
    /// use itertools::Itertools;
    ///
    /// assert!(itertools::equal((0..3).intersperse(8), vec![0, 8, 1, 8, 2]));
    /// ```
    fn intersperse(self, element: Self::Item) -> Intersperse<Self> where
        Self: Sized,
        Self::Item: Clone
    {
        Intersperse::new(self, element)
    }

    /// Create an iterator which iterates over both this and the specified
    /// iterator simultaneously, yielding pairs of two optional elements.
    ///
    /// This iterator is *fused*.
    ///
    /// When both iterators return **None**, all further invocations of *.next()* 
    /// will return **None**.
    ///
    /// # Example
    ///
    /// ```rust
    /// use itertools::EitherOrBoth::{Both, Right};
    /// use itertools::Itertools;
    /// let it = (0..1).zip_longest(1..3);
    /// assert!(itertools::equal(it, vec![Both(0, 1), Right(2)]));
    /// ```
    ///
    /// Iterator element type is **EitherOrBoth\<Self::Item, J::Item\>**.
    #[inline]
    fn zip_longest<J>(self, other: J) -> ZipLongest<Self, J::IntoIter> where
        J: IntoIterator,
        Self: Sized,
    {
        ZipLongest::new(self, other.into_iter())
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
    ///
    /// // An adaptor that gathers elements up in pairs
    /// let pit = (0..4).batching(|mut it| {
    ///            match it.next() {
    ///                None => None,
    ///                Some(x) => match it.next() {
    ///                    None => None,
    ///                    Some(y) => Some((x, y)),
    ///                }
    ///            }
    ///        });
    ///
    /// assert!(itertools::equal(pit, vec![(0, 1), (2, 3)]));
    /// ```
    ///
    fn batching<B, F>(self, f: F) -> Batching<Self, F> where
        F: FnMut(&mut Self) -> Option<B>,
        Self: Sized,
    {
        Batching::new(self, f)
    }

    /// Group iterator elements. Consecutive elements that map to the same key (“runs”),
    /// are returned as the iterator elements of **GroupBy**.
    ///
    /// Iterator element type is **(K, Vec\<Self::Item\>)**
    ///
    /// ## Example
    ///
    /// ```
    /// use itertools::Itertools;
    ///
    /// // group data into runs of larger than zero or not.
    /// let data = vec![1, 2, -3, 4, 5];
    ///
    /// let mut iter = data.into_iter().group_by(|elt| *elt >= 0);
    /// assert_eq!(iter.next(), Some((true, vec![1, 2])));
    /// assert_eq!(iter.next(), Some((false, vec![-3])));
    /// ```
    fn group_by<K, F: FnMut(&Self::Item) -> K>(self, key: F) -> GroupBy<K, Self, F> where
        Self: Sized,
    {
        GroupBy::new(self, key)
    }

    /// Split into an iterator pair that both yield all elements from
    /// the original iterator.
    ///
    /// **Note:** If the iterator is clonable, prefer using that instead
    /// of using this method. It is likely to be more efficient.
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
    /// let it = repeat('a').slice(..3);
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
    /// i.e. if it somehow participates in an “iterator knot” where it is an adaptor of itself.
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
    /// use itertools::Itertools;
    ///
    /// let it = (0..8).step(3);
    /// assert!(itertools::equal(it, vec![0, 3, 6]));
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
    /// let a = (0..11).step(3);
    /// let b = (0..11).step(5);
    /// let it = a.merge(b);
    /// assert!(itertools::equal(it, vec![0, 0, 3, 5, 6, 9, 10]));
    /// ```
    fn merge<J>(self, other: J) -> MergeAscend<Self, J::IntoIter> where
        Self: Sized,
        Self::Item: PartialOrd,
        J: IntoIterator<Item=Self::Item>,
    {
        fn wrapper<A: PartialOrd>(a: &A, b: &A) -> Ordering {
            a.partial_cmp(b).unwrap_or(Ordering::Less)
        };
        self.merge_by(other, wrapper)
    }

    /// Return an iterator adaptor that merges the two base iterators in order.
    /// This is much like *.merge()* but allows for a custom ordering.
    ///
    /// This can be especially useful for sequences of tuples.
    ///
    /// Iterator element type is **Self::Item**.
    ///
    /// ## Example
    /// ```
    /// use itertools::Itertools;
    ///
    /// let a = (0..).zip("bc".chars());
    /// let b = (0..).zip("ad".chars());
    /// let it = a.merge_by(b, |x, y| x.1.cmp(&y.1));
    /// assert!(itertools::equal(it, vec![(0, 'a'), (0, 'b'), (1, 'c'), (1, 'd')]));
    /// ```

    fn merge_by<J, F>(self, other: J, cmp: F) -> Merge<Self, J::IntoIter, F> where
        Self: Sized,
        J: IntoIterator<Item=Self::Item>,
        F: FnMut(&Self::Item, &Self::Item) -> Ordering
    {
        Merge::new(self, other.into_iter(), cmp)
    }

    /// Return an iterator adaptor that iterates over the cartesian product of
    /// the element sets of two iterators **self** and **J**.
    ///
    /// Iterator element type is **(Self::Item, J::Item)**.
    ///
    /// ```
    /// use itertools::Itertools;
    ///
    /// let it = (0..2).cartesian_product("αβ".chars());
    /// assert!(itertools::equal(it, vec![(0, 'α'), (0, 'β'), (1, 'α'), (1, 'β')]));
    /// ```
    fn cartesian_product<J>(self, other: J) -> Product<Self, J::IntoIter> where
        Self: Sized,
        Self::Item: Clone,
        J: IntoIterator,
        J::IntoIter: Clone,
    {
        Product::new(self, other.into_iter())
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

    /// Return an iterator adapter that allows peeking multiple values.
    ///
    /// After a call to *.next()* the peeking cursor is reset.
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

    /// Return an iterator adaptor that uses the passed-in closure to
    /// optionally merge together consecutive elements. For each pair the closure
    /// is passed the latest two elements, `x`, `y` and may return either `Ok(z)`
    /// to merge the two values or `Err((x, y))` to indicate they can't be merged.
    ///
    /// *.dedup()* and *.mend_slices()* are specializations of the coalesce
    /// adaptor.
    ///
    /// Iterator element type is **Self::Item**.
    ///
    /// This iterator is *fused*.
    ///
    /// ## Example
    ///
    /// ```
    /// use itertools::Itertools;
    ///
    /// // sum same-sign runs together
    /// let data = vec![-1., -2., -3., 3., 1., 0., -1.];
    /// assert!(itertools::equal(data.into_iter().coalesce(|x, y|
    ///         if (x >= 0.) == (y >= 0.) {
    ///             Ok(x + y)
    ///         } else {
    ///             Err((x, y))
    ///         }),
    ///         vec![-6., 4., -1.]));
    /// ```
    fn coalesce<F>(self, f: F) -> Coalesce<Self, F> where
        Self: Sized,
        F: FnMut(Self::Item, Self::Item) -> Result<Self::Item, (Self::Item, Self::Item)>
    {
        Coalesce::new(self, f)
    }

    /// Remove duplicates from sections of consecutive identical elements.
    /// If the iterator is sorted, all elements will be unique.
    ///
    /// Iterator element type is **Self::Item**.
    ///
    /// This iterator is *fused*.
    ///
    /// ## Example
    ///
    /// ```
    /// use itertools::Itertools;
    ///
    /// let data = vec![1., 1., 2., 3., 3., 2., 2.];
    /// assert!(itertools::equal(data.into_iter().dedup(),
    ///                          vec![1., 2., 3., 2.]));
    /// ```
    fn dedup(self) -> CoalesceFn<Self> where
        Self: Sized,
        Self::Item: PartialEq,
    {
        fn eq<T: PartialEq>(x: T, y: T) -> Result<T, (T, T)>
        {
            if x == y { Ok(x) } else { Err((x, y)) }
        }
        Coalesce::new(self, eq)
    }


    /// Return an iterator adaptor that joins together adjacent slices if possible.
    ///
    /// Only implemented for iterators with slice or string slice elements.
    /// Only slices that are contiguous together can be joined.
    ///
    /// ## Example
    ///
    /// ```
    /// use itertools::Itertools;
    ///
    /// let text = String::from("let there be text");
    /// let excerpts = vec![&text[0..4], &text[4..9], &text[10..12], &text[12..]];
    ///
    /// assert!(itertools::equal(excerpts.into_iter().mend_slices(),
    ///                          vec!["let there", "be text"]));
    /// ```
    fn mend_slices(self) -> CoalesceFn<Self> where
        Self: Sized,
        Self::Item: misc::MendSlice
    {
        fn mend<T: misc::MendSlice>(x: T, y: T) -> Result<T, (T, T)>
        {
            match misc::MendSlice::mend(x, y) {
                Some(z) => Ok(z),
                None => Err((x, y)),
            }
        }
        Coalesce::new(self, mend)
    }

    /// Return an iterator adaptor that borrows from a **Clone**-able iterator
    /// to only pick off elements while the predicate **f** returns **true**.
    ///
    /// It uses the **Clone** trait to restore the original iterator so that the last
    /// and rejected element is still available when **TakeWhileRef** is done.
    ///
    /// ## Example
    ///
    /// ```
    /// use itertools::Itertools;
    ///
    /// let mut alphanumerics = "abcdef012345".chars();
    ///
    /// let alphas = alphanumerics.take_while_ref(|c| c.is_alphabetic())
    ///                           .collect::<String>();
    /// assert_eq!(alphas, "abcdef");
    /// assert_eq!(alphanumerics.next(), Some('0'));
    ///
    /// ```
    fn take_while_ref<'a, F>(&'a mut self, f: F) -> TakeWhileRef<'a, Self, F> where
        Self: Clone,
        F: FnMut(&Self::Item) -> bool,
    {
        TakeWhileRef::new(self, f)
    }

    /// Return an iterator adaptor that iterates over the combinations of
    /// the elements from an iterator.
    ///
    /// Iterator element type is **(Self::Item, Self::Item)**.
    ///
    /// ```
    /// use itertools::Itertools;
    ///
    /// let it = (1..5).combinations();
    /// assert!(itertools::equal(it, vec![(1, 2), (1, 3), (1, 4), (2, 3), (2, 4), (3, 4)]));
    /// ```
    fn combinations(self) -> Combinations<Self> where
        Self: Sized + Clone, Self::Item: Clone
    {
        Combinations::new(self)
    }

    /// Like regular *.map()*, specialized to using a simple function pointer instead,
    /// so that the resulting **Map** iterator value can be cloned.
    ///
    /// Iterator element type is **B**.
    ///
    /// ## Example
    ///
    /// ```
    /// use itertools::Itertools;
    ///
    /// let data = vec![Ok(1), Ok(0), Err("No result")];
    ///
    /// let iter = data.iter().cloned().map_fn(Result::ok);
    /// let iter_copy = iter.clone();
    ///
    /// assert!(itertools::equal(iter, vec![Some(1), Some(0), None]));
    /// assert!(itertools::equal(iter_copy, vec![Some(1), Some(0), None]));
    /// ```
    fn map_fn<B>(self, f: fn(Self::Item) -> B) -> MapFn<Self, B> where
        Self: Sized
    {
        self.map(f)
    }

    /// **Deprecated:** Use *.map_fn()* instead.
    fn fn_map<B>(self, map: fn(Self::Item) -> B) -> FnMap<B, Self> where
        Self: Sized
    {
        FnMap::new(self, map)
    }


    // non-adaptor methods

    /// Find the position and value of the first element satisfying a predicate.
    ///
    /// The iterator is not advanced past the first element found.
    ///
    /// ## Example
    ///
    /// ```
    /// use itertools::Itertools;
    ///
    /// let text = "α-0";
    /// assert_eq!(text.chars().find_position(|ch| ch.is_numeric()), Some((2, '0')));
    /// assert_eq!(text.chars().find_position(|ch| ch.is_uppercase()), None);
    /// ```
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
    /// Return actual number of elements consumed, until done or reaching the end.
    ///
    /// ## Example
    ///
    /// ```
    /// use itertools::Itertools;
    ///
    /// let mut iter = "αβγ".chars();
    /// iter.dropn(2);
    /// assert!(itertools::equal(iter, "γ".chars()));
    ///
    /// assert_eq!((0..10).dropn(50), 10);
    /// ```
    fn dropn(&mut self, mut n: usize) -> usize
    {
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
    ///
    /// ## Example
    ///
    /// ```
    /// use itertools::Itertools;
    ///
    /// let mut iter = "αβγ".chars().dropping(2);
    /// assert!(itertools::equal(iter, "γ".chars()));
    /// ```
    fn dropping(mut self, n: usize) -> Self where
        Self: Sized,
    {
        self.dropn(n);
        self
    }

    /// Consume the last **n** elements from the iterator eagerly,
    /// and return the same iterator again.
    ///
    /// This is only possible on double ended iterators. **n** may be
    /// larger than the number of elements.
    ///
    /// Note: This method is eager, dropping the back elements immediately and
    /// preserves the iterator type.
    ///
    /// ## Example
    ///
    /// ```
    /// use itertools::Itertools;
    ///
    /// let init = vec![0, 3, 6, 9].into_iter().dropping_back(1);
    /// assert!(itertools::equal(init, vec![0, 3, 6]));
    /// ```
    fn dropping_back(mut self, n: usize) -> Self where
        Self: Sized,
        Self: DoubleEndedIterator,
    {
        self.by_ref().rev().dropn(n);
        self
    }

    /// Run the closure **f** eagerly on each element of the iterator.
    ///
    /// Consumes the iterator until its end.
    ///
    /// ## Example
    ///
    /// ```
    /// use std::sync::mpsc::channel;
    /// use itertools::Itertools;
    ///
    /// let (tx, rx) = channel();
    ///
    /// // use .foreach() to apply a function to each value -- sending it
    /// (0..5).map(|x| x * 2 + 1).foreach(|x| { tx.send(x).unwrap(); } );
    ///
    /// drop(tx);
    ///
    /// assert!(itertools::equal(rx.iter(), vec![1, 3, 5, 7, 9]));
    /// ```
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
        J: IntoIterator<Item=A>,
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
    /// value is returned inside **Ok**. Otherwise, the operation terminates
    /// and returns the first **Err** value it encounters. No iterator elements are
    /// consumed after the first error.
    ///
    /// The first accumulator value is the **start** parameter.
    /// Each iteration passes the accumulator value and the next value inside **Ok**
    /// to the fold function **f** and its return value becomes the new accumulator value.
    ///
    /// For example the sequence *Ok(1), Ok(2), Ok(3)* will result in a
    /// computation like this:
    ///
    /// ```ignore
    /// let mut accum = start;
    /// accum = f(accum, 1);
    /// accum = f(accum, 2);
    /// accum = f(accum, 3);
    /// ```
    ///
    /// With a **start** value of 0 and an addition as folding function,
    /// this effetively results in *((0 + 1) + 2) + 3*
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

    /// Accumulator of the elements in the iterator.
    ///
    /// Like *.fold()*, without a base case. If the iterator is
    /// empty, return **None**. With just one element, return it.
    /// Otherwise elements are accumulated in sequence using the closure **f**.
    ///
    /// ## Example
    ///
    /// ```
    /// use itertools::Itertools;
    ///
    /// assert_eq!((0..10).fold1(|x, y| x + y).unwrap_or(0), 45);
    /// assert_eq!((0..0).fold1(|x, y| x * y), None);
    /// ```
    fn fold1<F>(&mut self, mut f: F) -> Option<Self::Item> where
        F: FnMut(Self::Item, Self::Item) -> Self::Item,
    {
        match self.next() {
            None => None,
            Some(mut x) => {
                for y in self {
                    x = f(x, y);
                }
                Some(x)
            }
        }
    }

    /// Tell if the iterator is empty or not according to its size hint.
    /// Return **None** if the size hint does not tell, or return a **Some**
    /// value with the emptiness if it's possible to tell.
    ///
    /// ## Example
    ///
    /// ```
    /// use itertools::Itertools;
    ///
    /// assert_eq!((1..1).is_empty_hint(), Some(true));
    /// assert_eq!([1, 2, 3].iter().is_empty_hint(), Some(false));
    /// assert_eq!((0..10).filter(|&x| x > 0).is_empty_hint(), None);
    /// ```
    fn is_empty_hint(&self) -> Option<bool>
    {
        let (low, opt_hi) = self.size_hint();
        // check for erronous hint
        if let Some(hi) = opt_hi {
            if hi < low { return None }
        }

        if opt_hi == Some(0) {
            Some(true)
        } else if low > 0 {
            Some(false)
        } else {
            None
        }
    }
}

impl<T: ?Sized> Itertools for T where T: Iterator { }

/// Return **true** if both iterators produce equal sequences
/// (elements pairwise equal and sequences of the same length),
/// **false** otherwise.
///
/// ## Example
///
/// ```
/// assert!(itertools::equal(vec![1, 2, 3], 1..4));
/// assert!(!itertools::equal(&[0, 0], &[0, 0, 0]));
/// ```
pub fn equal<I, J>(a: I, b: J) -> bool where
    I: IntoIterator,
    J: IntoIterator,
    I::Item: PartialEq<J::Item>,
{
    let mut ia = a.into_iter();
    let mut ib = b.into_iter();
    loop {
        match (ia.next(), ib.next()) {
            (Some(ref x), Some(ref y)) if x == y => { }
            (None, None) => return true,
            _ => return false,
        }
    }
}

/// Partition a sequence using predicate **pred** so that elements
/// that map to **true** are placed before elements which map to **false**.
///
/// The order within the partitions is arbitrary.
///
/// Return the index of the split point.
///
/// ## Example
///
/// ```
/// use itertools::partition;
///
/// # // use repeated numbers to not promise any ordering
/// let mut data = [7, 1, 1, 7, 1, 1, 7];
/// let split_index = partition(&mut data, |elt| *elt >= 3);
///
/// assert_eq!(data, [7, 7, 7, 1, 1, 1, 1]);
/// assert_eq!(split_index, 3);
/// ```
pub fn partition<'a, A: 'a, I, F>(iter: I, mut pred: F) -> usize where
    I: IntoIterator<Item=&'a mut A>,
    I::IntoIter: DoubleEndedIterator,
    F: FnMut(&A) -> bool,
{
    let mut split_index = 0;
    let mut iter = iter.into_iter();
    'main: while let Some(front) = iter.next() {
        if !pred(front) {
            loop {
                match iter.next_back() {
                    Some(back) => if pred(back) {
                        std::mem::swap(front, back);
                        break;
                    },
                    None => break 'main,
                }
            }
        }
        split_index += 1;
    }
    split_index
}
