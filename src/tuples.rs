use std::collections::VecDeque;
use std::marker::PhantomData;

/// An iterator over all contiguous windows that produces tuples of a specific size.
///
/// See [`.tuple_windows()`](../trait.Itertools.html#method.tuple_windows) for more
/// information.
pub struct TupleWindows<I, T>
    where I: Iterator
{
    iter: I,
    buf: VecDeque<I::Item>,
    _marker: PhantomData<T>,
}

/// Create a new tuple windows iterator.
pub fn tuple_windows<I, T>(iter: I) -> TupleWindows<I, T>
    where I: Iterator,
          T: TupleCollect<I>,
{
    TupleWindows {
        iter: iter,
        buf: VecDeque::with_capacity(T::num_items()),
        _marker: PhantomData
    }
}

impl<I, T> Iterator for TupleWindows<I, T>
    where I: Iterator,
          I::Item: Clone,
          T: TupleCollect<I>
{
    type Item = T;

    fn next(&mut self) -> Option<T> {
        T::try_collect_from_iter_windows(self)
    }
}

impl<I, T> TupleWindows<I, T>
    where I: Iterator,
          T: TupleCollect<I>
{
    fn get(&mut self) -> Option<&mut VecDeque<I::Item>> {
        let remmaining = T::num_items() - self.buf.len();
        for (_, item) in (0..remmaining).zip(self.iter.by_ref()) {
            self.buf.push_back(item);
        }
        if self.buf.len() == T::num_items() {
            Some(&mut self.buf)
        } else {
            None
        }
    }

    /// Return a pair with the inner iterator and the buffer with the already produced items.
    ///
    /// ```
    /// use itertools::Itertools;
    /// use std::collections::VecDeque;
    ///
    /// let iter = 0..10;
    /// let mut w = iter.tuple_windows();
    /// for (a, _, _) in &mut w {
    ///     if a == 3 {
    ///         break
    ///     }
    /// }
    ///
    /// let (mut iter, buffer) = w.into_parts();
    /// // Items 4 and 5 was already produced
    /// assert_eq!(VecDeque::from(vec![4, 5]), buffer);
    /// // The next unproduced item is 6
    /// assert_eq!(Some(6), iter.next());
    /// ```
    pub fn into_parts(self) -> (I, VecDeque<I::Item>) {
        (self.iter, self.buf)
    }
}

pub trait TupleCollect<I>: Sized
    where I: Iterator
{
    fn try_collect_from_iter_windows(iter: &mut TupleWindows<I, Self>) -> Option<Self>
        where I::Item: Clone;

    fn collect_from_iter(iter: &mut I) -> Self;

    fn num_items() -> usize;
}

macro_rules! impl_tuple_collect {
    () => ();
    ($A:ident $($X:ident)*) => (
        impl<I, $A> TupleCollect<I> for ($A, $($X),*)
            where I: Iterator<Item = $A>
        {
            #[allow(unused_assignments, non_snake_case, unused_mut, unused_variables)]
            fn try_collect_from_iter_windows(iter: &mut TupleWindows<I, Self>) -> Option<Self>
                where I::Item: Clone,
            {
                iter.get().map(|v| {
                    let first = v.pop_front().unwrap();
                    let mut iter = v.iter().cloned();
                    let r = (
                        first,
                        // X must be used, so use it as var name
                        $({let $X = iter.next().unwrap(); $X }),*
                    );
                    r
                })
            }

            #[allow(non_snake_case)]
            fn collect_from_iter(iter: &mut I) -> Self {
                (
                    iter.next().unwrap(),
                    // X must be used, so use it as var name
                    $({let $X = iter.next().unwrap(); $X }),*
                )
            }

            #[allow(non_snake_case)]
            fn num_items() -> usize {
                // X must be used, so use it as var name
                1 $(+ { let $X = 1; $X})*
            }
        }

        impl_tuple_collect!($($X)*);
    )
}

impl_tuple_collect!(A A A A);
