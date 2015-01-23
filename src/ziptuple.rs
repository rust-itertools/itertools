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

// Macro to turn `t.$idx`, where $idx:tt, into an expression.
macro_rules! e(
    ($e:expr) => { $e }
);

macro_rules! impl_zip_iter(
    ($($idx:tt is $B:ident),*) => (
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
                $(
                    let $B = match e!(self.t. $idx).next() {
                        None => return None,
                        Some(elt) => elt
                    };
                )*
                Some(($($B,)*))
            }

            fn size_hint(&self) -> (usize, Option<usize>)
            {
                let mut low = ::std::usize::MAX;
                let mut high = None;
                $(
                    // update estimate
                    let (l, h) = e!(self.t. $idx).size_hint();
                    low = cmp::min(low, l);
                    high = match (high, h) {
                        (Some(u1), Some(u2)) => Some(cmp::min(u1, u2)),
                        _ => high.or(h)
                    };
                )*
                (low, high)
            }
        }
    );
);

impl_zip_iter!(0 is A);
impl_zip_iter!(0 is A, 1 is B);
impl_zip_iter!(0 is A, 1 is B, 2 is C);
impl_zip_iter!(0 is A, 1 is B, 2 is C, 3 is D);
impl_zip_iter!(0 is A, 1 is B, 2 is C, 3 is D, 4 is E);
impl_zip_iter!(0 is A, 1 is B, 2 is C, 3 is D, 4 is E, 5 is F);
impl_zip_iter!(0 is A, 1 is B, 2 is C, 3 is D, 4 is E, 5 is F, 6 is G);
impl_zip_iter!(0 is A, 1 is B, 2 is C, 3 is D, 4 is E, 5 is F, 6 is G, 7 is H);
impl_zip_iter!(0 is A, 1 is B, 2 is C, 3 is D, 4 is E, 5 is F, 6 is G, 7 is H, 8 is I);
