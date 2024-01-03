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
        todo!()
    }
}
