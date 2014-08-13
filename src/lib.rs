#![feature(macro_rules)]
#![crate_name="itertools"]
#![crate_type="dylib"]

//! Itertools — extra iterator adaptors, functions and macros
//!
//! To use the macros in this crate, use the `phase(plugin)` attribute:
//!
//! ```
//! #![feature(phase)]
//! #[phase(plugin, link)] extern crate itertools;
//! ```
//!
//! I recommend shortening the crate name with something like:
//!
//! ```
//! use it = itertools;
//! ```
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

pub use adaptors::Clones;
pub use adaptors::Interleave;
pub use adaptors::Product;
pub use adaptors::PutBack;
pub use adaptors::FnMap;
pub use boxiter::BoxIter;
pub use intersperse::Intersperse;
pub use stride::Stride;
pub use stride::StrideMut;
pub use times::Times;
pub use times::times;
mod adaptors;
mod boxiter;
mod intersperse;
mod stride;
mod times;

/// A helper trait for (x,y,z) ++ w => (x,y,z,w),
/// used for implementing `iproduct!` and `izip!`
trait AppendTuple<X, Y> {
    fn append(self, x: X) -> Y;
}

macro_rules! impl_append_tuple(
    () => (
        impl<T> AppendTuple<T, (T, )> for () {
            fn append(self, x: T) -> (T, ) {
                (x, )
            }
        }
    );

    ($A:ident $(,$B:ident)*) => (
        impl_append_tuple!($($B),*)
        #[allow(uppercase_variables)]
        impl<$A, $($B,)* T> AppendTuple<T, ($A, $($B,)* T)> for ($A, $($B),*) {
            fn append(self, x: T) -> ($A, $($B,)* T) {
                let ($A, $($B),*) = self;
                ($A, $($B,)* x)
            }
        }
    );
)

impl_append_tuple!(A, B, C, D, E, F, G, H, I, J, K, L)

#[deriving(Clone)]
pub struct FlatTuples<I> {
    pub iter: I,
}

impl<X, Y, T: AppendTuple<X, Y>, I: Iterator<(T, X)>>
Iterator<Y> for FlatTuples<I>
{
    #[inline]
    fn next(&mut self) -> Option<Y>
    {
        self.iter.next().map(|(t, x)| t.append(x))
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        self.iter.size_hint()
    }
}

impl<X, Y, T: AppendTuple<X, Y>, I: DoubleEndedIterator<(T, X)>>
DoubleEndedIterator<Y> for FlatTuples<I>
{
    #[inline]
    fn next_back(&mut self) -> Option<Y>
    {
        self.iter.next_back().map(|(t, x)| t.append(x))
    }
}

#[macro_export]
/// Create an iterator over the “cartesian product” of iterators.
///
/// Iterator element type is like `(A, B, ..., E)` if formed
/// from iterators `(I, J, ..., M)` implementing `I: Iterator<A>`,
/// `J: Iterator<B>`, ..., `M: Iterator<E>`
///
/// ## Example
///
/// ```rust
/// // Iterate over the coordinates of a 4 x 4 grid
/// // from (0, 0), (0, 1), .. etc until (3, 3)
/// for (i, j) in iproduct!(range(0, 4i), range(0, 4i)) {
///    // ..
/// }
/// ```
pub macro_rules! iproduct(
    ($I:expr) => (
        ($I)
    );
    ($I:expr, $J:expr $(, $K:expr)*) => (
        {
            let it = ::itertools::Product::new($I, $J);
            $(
                let it = ::itertools::FlatTuples{iter: ::itertools::Product::new(it, $K)};
            )*
            it
        }
    );
)

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
/// ```rust
/// // Iterate over three sequences side-by-side
/// let mut xs = [0u, 0, 0];
/// let ys = [72u, 73, 74];
/// for (i, a, b) in izip!(range(0, 100u), xs.mut_iter(), ys.iter()) {
///    *a = i ^ *b;
/// }
/// ```
pub macro_rules! izip(
    ($I:expr) => (
        ($I)
    );
    ($I:expr, $J:expr $(, $K:expr)*) => (
        {
            let it = $I.zip($J);
            $(
                let it = ::itertools::FlatTuples{iter: it.zip($K)};
            )*
            it
        }
    );
)

// Note: Instead of using struct Product, we could implement iproduct!()
// using .flat_map as well; however it can't implement size_hint.
// ($I).flat_map(|x| Repeat::new(x).zip($J))


/// `icompr` as in “iterator comprehension” allows creating a
/// mapped iterator with simple syntax, similar to set builder notation,
/// and directly inspired by Python. Supports an optional filter clause.
/// 
/// Syntax:
/// 
///  `icompr!(<expression> for <pattern> in <iterator>)`
///
/// or
///
///  `icompr!(<expression> for <pattern> in <iterator> if <expression>)`
///
/// Each element from the `<iterator>` expression is pattern matched
/// with the `<pattern>`, and the bound names are used to express the
/// mapped-to value.
///
/// Iterator element type is the type of `<expression>`
///
/// ## Example
///
/// ```rust
/// let mut squares = icompr!(x * x for x in range(1i, 100));
/// ```
#[macro_export]
pub macro_rules! icompr(
    ($r:expr for $x:pat in $J:expr if $pred:expr) => (
        ($J).filter_map(|$x| if $pred { Some($r) } else { None })
    );
    ($r:expr for $x:pat in $J:expr) => (
        ($J).filter_map(|$x| Some($r))
    );
)

/// Extra iterator methods for arbitrary iterators
pub trait Itertools<A> : Iterator<A> {
    // adaptors

    /// Like regular `.map`, but using a simple function pointer instead,
    /// so that the resulting `FnMap` iterator value can be cloned.
    ///
    /// Iterator element type is `B`
    fn fn_map<B>(self, map: fn(A) -> B) -> FnMap<A, B, Self> {
        FnMap::new(self, map)
    }

    /// Alternate elements from two iterators until both
    /// are run out
    ///
    /// Iterator element type is `A`
    fn interleave<J: Iterator<A>>(self, other: J) -> Interleave<Self, J> {
        Interleave::new(self, other)
    }

    /// An iterator adaptor to insert a particular value
    /// between each element of the adapted iterator.
    ///
    /// Iterator element type is `A`
    fn intersperse(self, element: A) -> Intersperse<A, Self> {
        Intersperse::new(self, element)
    }

    // non-adaptor methods

    /// Consume `n` elements of the iterator eagerly
    ///
    /// Return actual number of elements consumed,
    /// until done or reaching the end.
    fn dropn(&mut self, mut n: uint) -> uint {
        let start = n;
        while n > 0 {
            match self.next() {
                Some(..) => n -= 1,
                None => break
            }
        }
        start - n
    }

    /// Run the iterator, eagerly, to the end and consume all its elements.
    ///
    /// ## Example
    ///
    /// ```rust
    /// let mut cnt = 0;
    /// "hi".chars().map(|c| cnt += 1).drain();
    /// ```
    ///
    fn drain(&mut self) {
        for _ in *self { /* nothing */ }
    }

    /// Run the closure `f` eagerly on each element of the iterator.
    ///
    /// Consumes the iterator until its end.
    fn apply(&mut self, f: |A|) {
        for elt in *self { f(elt) }
    }


    /// Assign to each reference in `iter` from this iterator, stopping
    /// at the shortest of the two iterators.
    ///
    /// Return the number of elements written.
    #[inline]
    fn write_to<'a, I: Iterator<&'a mut A>>(&mut self, iter: I) -> uint
    {
        let mut count = 0u;
        let mut iter = iter;
        for elt in *self {
            match iter.next() {
                None => break,
                Some(ptr) => *ptr = elt
            }
            count += 1;
        }
        count
    }
}

impl<A, T: Iterator<A>> Itertools<A> for T { }

pub trait ItertoolsClonable<A> {
    /// An iterator like `.map(|elt| elt.clone())`
    fn clones(self) -> Clones<Self> {
        Clones::new(self)
    }
}

impl<'a, A: Clone, I: Iterator<&'a A>> ItertoolsClonable<&'a A> for I { }
