use std::cmp;

#[derive(Clone)]
/// Create an iterator running multiple iterators in lockstep.
///
/// The iterator **Zip\<(I, J, ..., M)\>** is formed from a tuple of iterators and yields elements
/// until any of the subiterators yields **None**.
///
/// Iterator element type is like **(A, B, ..., E)** where **A** to **E** are the respective
/// subiterator types.
///
/// ## Example
///
/// ```
/// use itertools::Zip;
///
/// // Iterate over three sequences side-by-side
/// let mut xs = [0, 0, 0];
/// let ys = [69, 107, 101];
///
/// for (i, a, b) in Zip::new((0i32..100, xs.iter_mut(), ys.iter())) {
///    *a = i ^ *b;
/// }
/// 
/// assert_eq!(xs, [69, 106, 103]);
/// ```
pub struct Zip<T> {
    t: T
}

impl<T> Zip<T> where Zip<T>: Iterator
{
    /// Create a new **Zip** from a tuple of iterators.
    pub fn new(t: T) -> Zip<T>
    {
        Zip{t: t}
    }
}

macro_rules! impl_zip_iter(
    ($($B:ident),*) => (
        #[allow(non_snake_case)]
        impl<$($B),*> Iterator for Zip<($($B,)*)>
            where
            $(
                $B: Iterator,
            )*
        {
            type Item = ($(<$B as Iterator>::Item,)*);

            fn next(&mut self) -> Option<
                    ($(<$B as Iterator>::Item,)*)
                >
            {
                let &mut Zip { t : ($(ref mut $B,)*)} = self;
                // WARNING: partial consume possible
                // Zip worked the same.
                $(
                    let $B = match $B.next() {
                        None => return None,
                        Some(elt) => elt
                    };
                )*
                Some(($($B,)*))
            }

            fn size_hint(&self) -> (usize, Option<usize>)
            {
                let low = ::std::usize::MAX;
                let high = None;
                let &Zip { t : ($(ref $B,)*) } = self;
                $(
                    // update estimate
                    let (l, h) = $B.size_hint();
                    let low = cmp::min(low, l);
                    let high = match (high, h) {
                        (Some(u1), Some(u2)) => Some(cmp::min(u1, u2)),
                        _ => high.or(h)
                    };
                )*
                (low, high)
            }
        }
    );
);

impl_zip_iter!(A);
impl_zip_iter!(A, B);
impl_zip_iter!(A, B, C);
impl_zip_iter!(A, B, C, D);
impl_zip_iter!(A, B, C, D, E);
impl_zip_iter!(A, B, C, D, E, F);
impl_zip_iter!(A, B, C, D, E, F, G);
impl_zip_iter!(A, B, C, D, E, F, G, H);
impl_zip_iter!(A, B, C, D, E, F, G, H, I);
