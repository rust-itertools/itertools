#![cfg(feature = "use_alloc")]

use alloc::vec::Vec;

#[derive(Clone)]
/// An iterator adaptor that iterates over the cartesian product of
/// multiple iterators of type `I`.
///
/// An iterator element type is `Vec<I::Item>`.
///
/// See [`.multi_cartesian_product()`](crate::Itertools::multi_cartesian_product)
/// for more information.
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct MultiProduct<I>(Option<MultiProductInner<I>>)
where
    I: Iterator + Clone,
    I::Item: Clone;

#[derive(Clone)]
struct MultiProductInner<I>
where
    I: Iterator + Clone,
    I::Item: Clone,
{
    iters: Vec<MultiProductIter<I>>,
    cur: Option<Vec<I::Item>>,
}

impl<I> std::fmt::Debug for MultiProduct<I>
where
    I: Iterator + Clone + std::fmt::Debug,
    I::Item: Clone + std::fmt::Debug,
{
    debug_fmt_fields!(MultiProduct, 0);
}

impl<I> std::fmt::Debug for MultiProductInner<I>
where
    I: Iterator + Clone + std::fmt::Debug,
    I::Item: Clone + std::fmt::Debug,
{
    debug_fmt_fields!(MultiProductInner, iters, cur);
}

/// Create a new cartesian product iterator over an arbitrary number
/// of iterators of the same type.
///
/// Iterator element is of type `Vec<H::Item::Item>`.
pub fn multi_cartesian_product<H>(iters: H) -> MultiProduct<<H::Item as IntoIterator>::IntoIter>
where
    H: Iterator,
    H::Item: IntoIterator,
    <H::Item as IntoIterator>::IntoIter: Clone,
    <H::Item as IntoIterator>::Item: Clone,
{
    let inner = MultiProductInner {
        iters: iters
            .map(|i| MultiProductIter::new(i.into_iter()))
            .collect(),
        cur: None,
    };
    MultiProduct(Some(inner))
}

#[derive(Clone, Debug)]
/// Holds the state of a single iterator within a `MultiProduct`.
struct MultiProductIter<I>
where
    I: Iterator + Clone,
    I::Item: Clone,
{
    iter: I,
    iter_orig: I,
}

impl<I> MultiProductIter<I>
where
    I: Iterator + Clone,
    I::Item: Clone,
{
    fn new(iter: I) -> Self {
        Self {
            iter: iter.clone(),
            iter_orig: iter,
        }
    }
}

impl<I> Iterator for MultiProduct<I>
where
    I: Iterator + Clone,
    I::Item: Clone,
{
    type Item = Vec<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        let inner = self.0.as_mut()?;
        match &mut inner.cur {
            Some(values) => {
                for (iter, item) in inner.iters.iter_mut().zip(values.iter_mut()).rev() {
                    if let Some(new) = iter.iter.next() {
                        *item = new;
                        return Some(values.clone());
                    } else {
                        iter.iter = iter.iter_orig.clone();
                        // `cur` is not none so the untouched `iter_orig` can not be empty.
                        *item = iter.iter.next().unwrap();
                    }
                }
                // The iterator ends.
                self.0 = None;
                None
            }
            // Only the first time.
            None => {
                let next: Option<Vec<_>> = inner.iters.iter_mut().map(|i| i.iter.next()).collect();
                if next.is_some() {
                    inner.cur = next.clone();
                } else {
                    self.0 = None;
                }
                next
            }
        }
    }
}

impl<I> std::iter::FusedIterator for MultiProduct<I>
where
    I: Iterator + Clone,
    I::Item: Clone,
{
}
