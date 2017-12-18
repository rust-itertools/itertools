use super::size_hint;

/// See [`multizip_fallback`](../fn.multizip_fallback.html) for more information.
#[derive(Clone)]
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct ZipAll<T, X> {
    t: T,
    def: X,
}
/// An iterator that allows running multiple iterators in lockstep while extending depleted
/// iterators with fallback values.
///
/// The iterator `ZipAll<(A, C, ..., M), (B, D, ..., N)>` is formed from a tuple of tuples where
/// the inner tuples contains an Iterator (or values that implement `IntoIterator`) and a fallback
/// value. The iterator yields elements until all of the subiterators returns None. If any
/// subiterator returns None before they all do, its fallback value will be used.
///
/// The iterator element type is a tuple like `(B, D, ..., N)` where `B` to `N` are the
/// element types of the subiterator.
///
/// If extending with fallback values isn't needed, use [`multizip()`].
///
/// [`multizip()`]: fn.multizip.html
///
/// ```
/// use itertools::multizip_fallback;
///
/// let a = &[2, 8, 5, 7];
/// let b = 5..7;
/// let c = vec![10, 2, 20];
/// let mut multiples = Vec::new();
///
/// for (first, second, third) in multizip_fallback(((a, &3), (b, 2), (c, 1))) {
///     multiples.push(*first * second * third);
/// }
///
/// assert_eq!(multiples, vec![2*5*10, 8*6*2, 5*2*20, 7*2*1]);
/// ```
pub fn multizip_fallback<T, U, X>(t: U) -> ZipAll<T, X>
where
    ZipAll<T, X>: From<U>,
    ZipAll<T, X>: Iterator,
{
    ZipAll::from(t)
}

macro_rules! impl_zip_all_iter {
    ($(($B:ident, $C:ident)),*) => (
        #[allow(non_snake_case)]
        impl<$($B: IntoIterator<Item = $C>),*, $($C: Clone),*>
        From<($(($B,$C),)*)> for ZipAll<($($B::IntoIter,)*), ($($C,)*)> {
            fn from(t: ($(($B,$C),)*)) -> Self {
                let ($(($B,$C),)*)= t;
                ZipAll {
                    t: ($($B.into_iter(),)*),
                    def: ($($C,)*)
                }
            }
        }

        #[allow(non_snake_case)]
        #[allow(unused_assignments)]
        impl<$($B,$C),*> Iterator for ZipAll<($($B,)*), ($($C,)*)>
            where
            $(
                $B: Iterator<Item = $C>,
                $C: Clone,
            )*
        {
            type Item = ($($B::Item,)*);

            fn next(&mut self) -> Option<Self::Item>
            {
                let ($(ref mut $B,)*) = self.t;
                let ($(ref mut $C,)*) = self.def;
                let mut empty = true;
                $(
                    let $B = match $B.next() {
                        None => $C.clone(),
                        Some(elt) => {
                            empty = false;
                            elt
                        }
                    };
                )*
                if empty {
                    None
                } else {
                    Some(($($B,)*))
                }
            }

            fn size_hint(&self) -> (usize, Option<usize>)
            {
                let sh = (::std::usize::MAX, None);
                let ($(ref $B,)*) = self.t;
                $(
                    let sh = size_hint::min($B.size_hint(), sh);
                )*
                sh
            }
        }
        #[allow(non_snake_case)]
        impl<$($B,$C),*> ExactSizeIterator for ZipAll<($($B,)*), ($($C,)*)>
        where
        $(
            $B: ExactSizeIterator<Item = $C>,
            $C: Clone,
        )*
        { }
    );
}

impl_zip_all_iter!((A, B));
impl_zip_all_iter!((A, B), (C, D));
impl_zip_all_iter!((A, B), (C, D), (E, F));
impl_zip_all_iter!((A, B), (C, D), (E, F), (G, H));
impl_zip_all_iter!((A, B), (C, D), (E, F), (G, H), (I, J));
impl_zip_all_iter!((A, B), (C, D), (E, F), (G, H), (I, J), (K, L));
impl_zip_all_iter!((A, B), (C, D), (E, F), (G, H), (I, J), (K, L), (M, N));
impl_zip_all_iter!((A, B), (C, D), (E, F), (G, H), (I, J), (K, L), (M, N), (O, P));
