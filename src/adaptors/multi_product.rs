#![cfg(feature = "use_std")]

use size_hint;
use Itertools;
use std::marker::PhantomData;

#[derive(Clone)]
/// An iterator adaptor that iterates over the cartesian product of
/// multiple iterators of type `I`.
///
/// An iterator element type is `Vec<I::Item>`.
///
/// See [`.multi_cartesian_product()`](../trait.Itertools.html#method.multi_cartesian_product)
/// for more information.
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct MultiProduct<I>
    where I: Iterator + Clone
{
    iters: Vec<MultiProductIter<I>>,
    cur: Option<Vec<I::Item>>,
}


/// An iterator adaptor that iterates over the cartesian product of
/// multiple iterators of type `I`.
///
/// An iterator element type is `[I::Item; N]`, where `N` is the number of
/// sub-iterators.
///
/// Type `A` is a dummy array type, the length of which is used to determine the
/// length of yielded items when iterating. The array item component of `A` is
/// not used.
///
/// See [`iproduct_arr`](../macro.iproduct_arr.html) for more information.
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct MultiProductArray<I, A>(MultiProduct<I>, PhantomData<A>)
    where I: Iterator + Clone;

/// Create a new cartesian product iterator over an arbitrary number
/// of iterators of the same type.
///
/// Iterator element is of type `Vec<H::Item::Item>`.
pub fn multi_cartesian_product<H>(iters: H)
    -> MultiProduct<<H::Item as IntoIterator>::IntoIter>
    where H: Iterator,
          H::Item: IntoIterator,
          <H::Item as IntoIterator>::IntoIter: Clone,
{
    MultiProduct {
        iters: iters.map(|iter| {
            let iter = iter.into_iter();
            MultiProductIter {
                iter: iter.clone(),
                iter_orig: iter
            }
        }).collect(),
        cur: None
    }
}

#[derive(Clone, Debug)]
/// Holds the state of a single iterator within a MultiProduct.
struct MultiProductIter<I>
    where I: Iterator + Clone
{
    iter: I,
    iter_orig: I,
}

impl<I> MultiProduct<I>
    where I: Iterator + Clone
{
    /// Converts this iterator into one which yields arrays instead of `Vec`s.
    /// 
    /// Type `A` is a dummy array type, the length of which is used to determine
    /// the length of yielded items when iterating. If the number of
    /// sub-iterators does not match the length of type `A`, it will `panic`.
    /// 
    /// The array item component of `A` is not used.
    /// 
    /// ```
    /// use itertools::Itertools;
    /// let mut multi_prod_array = (0..4).map(|_| 0..3)
    ///     .multi_cartesian_product()
    ///     .array::<[(); 4]>();
    /// ```
    /// 
    /// In most instances, it is preferable to construct using the
    /// [`iproduct_arr`](../macro.iproduct_arr.html) macro.
    pub fn array<A>(self) -> MultiProductArray<I, A>
        where MultiProductArray<I, A>: AssertIterLength
    {
        let prod = MultiProductArray(self, PhantomData::<A>);
        prod.assert_iter_length();
        prod
    }

    /// Returns first item of each iterator as a `Vec`, or None if any iterator
    /// is empty.
    fn initial_iteration(
        multi_iters: &mut [MultiProductIter<I>]
    ) -> Option<Vec<I::Item>> {
        let iter_count = multi_iters.len();

        let initial: Vec<I::Item> = multi_iters.iter_mut()
            .map(|multi_iter| multi_iter.iter.next())
            .while_some()
            .collect();

        if initial.len() == iter_count {
            Some(initial)
        } else {
            None
        }
    }

    /// Iterates the rightmost iterator, then recursively iterates iterators
    /// to the left if necessary.
    ///
    /// Returns `Ok(())` if the iteration succeeded, else `Err(())`.
    fn iterate_last(
        multi_iters: &mut [MultiProductIter<I>],
        curs: &mut [I::Item]
    ) -> Result<(), ()> {
        // If split fails, reached end of iterator list; all iterators finished.
        let (last, rest) = try!(multi_iters.split_last_mut().ok_or(()));

        // Should be the same length as multi_iters
        let (last_cur, rest_curs) = curs.split_last_mut().unwrap();

        *last_cur = if let Some(next) = last.iter.next() {
            next
        } else {
            last.iter = last.iter_orig.clone();

            // Propagate failures from further multi_iters
            try!(Self::iterate_last(rest, rest_curs));

            // If restarted iter returns None, it is empty, therefore whole
            // product is empty; finish.
            try!(last.iter.next().ok_or(()))
        };

        Ok(())
    }

    fn in_progress(&self) -> bool {
        self.cur.is_some()
    }

    fn advance(&mut self) {
        if self.iters.len() == 0 {
            return;
        }

        let mut finished = false;

        match self.cur {
            None => {
                self.cur = Self::initial_iteration(&mut self.iters);
            },
            Some(ref mut cur) => {
                finished = Self::iterate_last(&mut self.iters, cur) == Err(());
            }
        }

        if finished {
            self.cur = None;
        }
    }

    fn _count(self) -> usize {
        if self.iters.len() == 0 {
            return 0;
        }

        if !self.in_progress() {
            return self.iters.into_iter().fold(1, |acc, multi_iter| {
                acc * multi_iter.iter.count()
            });
        }

        self.iters.into_iter().fold(
            0,
            |acc, MultiProductIter { iter, iter_orig }| {
                let total_count = iter_orig.count();
                let cur_count = iter.count();
                acc * total_count + cur_count
            }
        )
    }

    fn _size_hint(&self) -> (usize, Option<usize>) {
        // Not ExactSizeIterator because size may be larger than usize
        if self.iters.len() == 0 {
            return (0, Some(0));
        }

        if !self.in_progress() {
            return self.iters.iter().fold((1, Some(1)), |acc, multi_iter| {
                size_hint::mul(acc, multi_iter.iter.size_hint())
            });
        }

        self.iters.iter().fold(
            (0, Some(0)),
            |acc, &MultiProductIter { ref iter, ref iter_orig }| {
                let cur_size = iter.size_hint();
                let total_size = iter_orig.size_hint();
                size_hint::add(size_hint::mul(acc, total_size), cur_size)
            }
        )
    }
}

impl<I> Iterator for MultiProduct<I>
    where I: Iterator + Clone,
          I::Item: Clone
{
    type Item = Vec<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        self.advance();

        if let Some(ref cur) = self.cur {
            Some(cur.clone())
        } else {
            None
        }
    }

    fn count(self) -> usize {
        self._count()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self._size_hint()
    }

    fn last(self) -> Option<Self::Item> {
        let iter_count = self.iters.len();

        let lasts: Vec<I::Item> = self.iters.into_iter()
            .map(|multi_iter| multi_iter.iter.last())
            .while_some()
            .collect();

        if lasts.len() == iter_count {
            Some(lasts)
        } else {
            None
        }
    }
}

/// A trait to check that the number of iterators provided to a
/// `MultiProductArray` is correct.
pub trait AssertIterLength {
    /// Asserts that the number of iterators matches the length of the Item
    /// type.
    fn assert_iter_length(&self);
}

macro_rules! multi_product_array_impl {
    ($N:expr, $($M:expr,)*) => {
        multi_product_array_impl!($($M,)*);

        impl<I, _A> Iterator for MultiProductArray<I, [_A; $N]>
            where I: Iterator + Clone,
                  I::Item: Clone
        {
            type Item = [I::Item; $N];

            fn next(&mut self) -> Option<Self::Item> {
                (self.0).advance();

                if let Some(ref cur) = (self.0).cur {
                    let mut _cur_iter = cur.iter();
                    Some([ $({
                        $M; // Dummy macro expansion statement
                        if let Some(c) = _cur_iter.next() {
                            c.clone()
                        } else {
                            return None;
                        }
                    },)* ])
                } else {
                    None
                }
            }

            fn count(self) -> usize {
                (self.0)._count()
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                (self.0)._size_hint()
            }

            fn last(self) -> Option<Self::Item> {
                let mut _lasts = (self.0).iters.into_iter()
                    .map(|multi_iter| multi_iter.iter.last())
                    .while_some();

                Some([ $({
                    $M; // Dummy macro expansion statement
                    if let Some(last) = _lasts.next() {
                        last
                    } else {
                        return None;
                    }
                },)* ])
            }
        }

        impl<I, _A> AssertIterLength for MultiProductArray<I, [_A; $N]>
            where I: Iterator + Clone
        {
            fn assert_iter_length(&self) {
                let len = (self.0).iters.len();
                if len != $N {
                    panic!("MultiProductArray constructed with incorrect \
                    number of iterators; iters={} arraylen={}", len, $N);
                }
            }
        }
    };
    () => {};
}

multi_product_array_impl!{
    32, 31, 30,
    29, 28, 27, 26, 25, 24, 23, 22, 21, 20,
    19, 18, 17, 16, 15, 14, 13, 12, 11, 10,
    9,  8,  7,  6,  5,  4,  3,  2,  1,  0,
}