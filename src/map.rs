
/// Clonable iterator adaptor to map elementwise
/// from `Iterator<A>` to `Iterator<B>`
///
/// Created with `.map_unboxed(..)` on an iterator
///
/// Iterator element type is `B`
#[deprecated]
pub struct MapMut<F, I> {
    map: F,
    iter: I,
}

impl<A, B, F: FnMut(A) -> B, I> MapMut<F, I>
{
    pub fn new(iter: I, map: F) -> MapMut<F, I> {
        MapMut{iter: iter, map: map}
    }
}

impl<A, B, F, I> Iterator for MapMut<F, I>
    where
        I: Iterator<Item=A>,
        F: FnMut(A) -> B,
{
    type Item = B;
    #[inline]
    fn next(&mut self) -> Option<B> {
        self.iter.next().map(|a| (self.map)(a))
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        self.iter.size_hint()
    }
}

impl<A, B, F, I> DoubleEndedIterator for MapMut<F, I>
    where
        I: DoubleEndedIterator<Item=A>,
        F: FnMut(A) -> B,
{
    #[inline]
    fn next_back(&mut self) -> Option<B> {
        self.iter.next_back().map(|a| (self.map)(a))
    }
}

impl<A, B, F: Clone + FnMut(A) -> B, I: Clone> Clone for MapMut<F, I>
{
    #[inline]
    fn clone(&self) -> MapMut<F, I> {
        MapMut::new(self.iter.clone(), self.map.clone())
    }
}

