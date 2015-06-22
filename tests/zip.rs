extern crate itertools;

use itertools::Itertools;
use itertools::EitherOrBoth::{Both, Left, Right};
#[cfg(feature = "unstable")]
use itertools::ZipTrusted;

#[test]
fn zip_longest_fused()
{
    let a = [Some(1), None, Some(3), Some(4)];
    let b = [1, 2, 3];

    let unfused = a.iter().batching(|mut it| *it.next().unwrap())
        .zip_longest(b.iter().cloned());
    itertools::assert_equal(unfused,
                       vec![Both(1, 1), Right(2), Right(3)]);
}

#[cfg(feature = "unstable")]
#[test]
fn test_zip_longest_size_hint() {
    let c = (1..10).cycle();
    let v: &[_] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
    let v2 = &[10, 11, 12];

    assert_eq!(c.zip_longest(v.iter()).size_hint(), (std::usize::MAX, None));

    assert_eq!(v.iter().zip_longest(v2.iter()).size_hint(), (10, Some(10)));
}

#[test]
fn test_double_ended_zip_longest() {
    let xs = [1, 2, 3, 4, 5, 6];
    let ys = [1, 2, 3, 7];
    let a = xs.iter().map(|&x| x);
    let b = ys.iter().map(|&x| x);
    let mut it = a.zip_longest(b);
    assert_eq!(it.next(), Some(Both(1, 1)));
    assert_eq!(it.next(), Some(Both(2, 2)));
    assert_eq!(it.next_back(), Some(Left(6)));
    assert_eq!(it.next_back(), Some(Left(5)));
    assert_eq!(it.next_back(), Some(Both(4, 7)));
    assert_eq!(it.next(), Some(Both(3, 3)));
    assert_eq!(it.next(), None);
}


#[cfg(feature = "unstable")]
#[test]
fn ziptrusted_1() {
    let mut xs = [0; 6];
    let mut ys = [0; 8];
    let mut zs = [0; 7];

    xs.iter_mut().set_from(0..);
    ys.iter_mut().set_from(0..);
    zs.iter_mut().set_from(0..);

    let it = ZipTrusted::new((xs.iter(), ys.iter()));
    assert_eq!(it.size_hint(), (6, Some(6)));
    itertools::assert_equal(it, xs.iter().zip(ys.iter()));

    let it = ZipTrusted::new((xs.iter(), ys.iter(), zs.iter()));
    assert_eq!(it.size_hint(), (6, Some(6)));
    itertools::assert_equal(it, xs.iter()
                             .zip(ys.iter())
                             .zip(zs.iter())
                             .map(|((a, b), c)| (a, b, c)));
}
