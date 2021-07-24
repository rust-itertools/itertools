/// Unzips an iterator over tuples into a tuple of containers.
///
/// ```
/// use itertools::multiunzip;
///
/// let inputs = vec![(1, 2, 3), (4, 5, 6), (7, 8, 9)];
///
/// let (a, b, c): (Vec<_>, Vec<_>, Vec<_>) = multiunzip(inputs);
///
/// assert_eq!((a, b, c), (vec![1, 4, 7], vec![2, 5, 8], vec![3, 6, 9]));
/// ```
pub fn multiunzip<FromI, I>(i: I) -> FromI
where
    I: IntoIterator,
    I::IntoIter: MultiUnzip<FromI>,
{
    i.into_iter().multiunzip()
}

/// An iterator that can be unzipped into multiple collections.
///
/// See [`.multiunzip()`](crate::Itertools::multiunzip) for more information.
pub trait MultiUnzip<FromI>: Iterator {
    /// Unzip this iterator into multiple collections.
    fn multiunzip(self) -> FromI;
}

macro_rules! impl_unzip_iter {
    ($($T:ident => $FromT:ident),*) => (
        impl_unzip_iter!(@rec $($T => $FromT,)*);
    );
    (@rec) => ();
    (@rec $__:ident => $___:ident, $($T:ident => $FromT:ident,)*) => (
        #[allow(non_snake_case)]
        impl<IT: Iterator<Item = ($($T,)*)>, $($T, $FromT: Default + Extend<$T>),* > MultiUnzip<($($FromT,)*)> for IT {
            fn multiunzip(self) -> ($($FromT,)*) {
                let mut res = ($($FromT::default(),)*);
                let ($($FromT,)*) = &mut res;

                // Still unstable #72631
                // let (lower_bound, _) = self.size_hint();
                // if lower_bound > 0 {
                //     $($FromT.extend_reserve(lower_bound);)*
                // }

                self.fold((), |(), ($($T,)*)| {
                    // Still unstable #72631
                    // $( $FromT.extend_one($T); )*
                    $( $FromT.extend(std::iter::once($T)); )*
                });
                res
            }
        }
        impl_unzip_iter!(@rec $($T => $FromT,)*);
    );
}

impl_unzip_iter!(L => FromL, K => FromK, J => FromJ, I => FromI, H => FromH, G => FromG, F => FromF, E => FromE, D => FromD, C => FromC, B => FromB, A => FromA);
