use super::size_hint;

/// An iterator which zips two iterators using a function to produce an arbitrary result
/// type. The resulting iterator will be as large as the smallest of the two iterators given.
/// 
/// # Params
/// * left The left (arg 1) iterator to zip.
/// * right The right (arg 2) iterator to zip.
/// * zipper The function(left, right) to use in zipping the iterators.
/// 
/// ```
/// use itertools::Itertools;
/// 
/// let mut zipped = [1, 2, 9].iter().zip_with([4, 5, 6].iter(), |x,y| x+y);
/// for x in vec![5, 7, 15] { assert_eq!(x, zipped.next().expect("unexpected zip_with result")) };
/// ```
pub fn zip_with<T, U, R, F>(left: T, right: U, zipper: F) -> ZipWith<T, U, F> where
    T: Iterator,
    U: Iterator, 
    F: Fn(T::Item, U::Item) -> R
{
    ZipWith { left: left.into_iter(), right: right.into_iter(), zipper: zipper }
}

pub trait IntoZipWith: IntoIterator + Sized 
{
    fn zip_with<R, F, S>(self, other: R, zipper: F) -> ZipWith<Self::IntoIter, R::IntoIter, F> where 
        R: Sized + IntoIterator,
        F: Fn(Self::Item, R::Item) -> S
    {
        zip_with(self.into_iter(), other.into_iter(), zipper)
    }
}

impl<T: Iterator> IntoZipWith for T {}

/// A ZipWith implementation, zips two iterators into a function.
///
/// See [`.zip_with()`](../trait.Itertools.html#method.zip_with) for more information.
#[derive(Clone, Debug)]
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct ZipWith<T, U, F> 
{
    left: T,
    right: U,
    zipper: F
}

impl<T, U, R, F> Iterator for ZipWith<T, U, F> where
    T: Iterator,
    U: Iterator,
    F: Fn(T::Item, U::Item) -> R
{
    type Item = R;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> 
    {
        match self.left.next()
        {
            Some(l) => match self.right.next()
            {
                Some(r) => Some((self.zipper) (l, r)),
                None => None
            },
            None => None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) 
    {
        size_hint::min(self.left.size_hint(), self.right.size_hint())
    }
}
