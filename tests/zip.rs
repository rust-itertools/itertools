use itertools::multizip;
use itertools::EitherOrBoth::{Both, Left, Right};
use itertools::Itertools;

#[test]
fn zip_longest_fused() {
    let a = [Some(1), None, Some(3), Some(4)];
    let b = [1, 2, 3];

    let unfused = a
        .iter()
        .batching(|it| *it.next().unwrap())
        .zip_longest(b.iter().cloned());
    itertools::assert_equal(unfused, vec![Both(1, 1), Right(2), Right(3)]);
}

#[test]
fn test_zip_longest_size_hint() {
    let c = (1..10).cycle();
    let v: &[_] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
    let v2 = &[10, 11, 12];

    assert_eq!(c.zip_longest(v.iter()).size_hint(), (usize::MAX, None));

    assert_eq!(v.iter().zip_longest(v2.iter()).size_hint(), (10, Some(10)));
}

#[test]
fn test_double_ended_zip_longest() {
    let xs = [1, 2, 3, 4, 5, 6];
    let ys = [1, 2, 3, 7];
    let a = xs.iter().copied();
    let b = ys.iter().copied();
    let mut it = a.zip_longest(b);
    assert_eq!(it.next(), Some(Both(1, 1)));
    assert_eq!(it.next(), Some(Both(2, 2)));
    assert_eq!(it.next_back(), Some(Left(6)));
    assert_eq!(it.next_back(), Some(Left(5)));
    assert_eq!(it.next_back(), Some(Both(4, 7)));
    assert_eq!(it.next(), Some(Both(3, 3)));
    assert_eq!(it.next(), None);
}

#[test]
fn test_double_ended_zip() {
    let xs = [1, 2, 3, 4, 5, 6];
    let ys = [1, 2, 3, 7];
    let a = xs.iter().copied();
    let b = ys.iter().copied();
    let mut it = multizip((a, b));
    assert_eq!(it.next_back(), Some((4, 7)));
    assert_eq!(it.next_back(), Some((3, 3)));
    assert_eq!(it.next_back(), Some((2, 2)));
    assert_eq!(it.next_back(), Some((1, 1)));
    assert_eq!(it.next_back(), None);
}

#[test]
/// The purpose of this test is not really to test the iterator mechanics itself, rather that it
/// compiles and accepts temporary values as inputs, as those would be valid when not used with the
/// izip! macro and zipped manually via .zip calls
fn test_izip_with_temporaries() {
    struct Owned {
        data: Vec<i32>,
    }

    impl Owned {
        fn new(val: i32) -> Self {
            Self {
                data: vec![val; 10],
            }
        }

        fn as_view(&self) -> View<'_> {
            View {
                data: self.data.as_slice(),
            }
        }
    }

    struct View<'a> {
        data: &'a [i32],
    }

    impl View<'_> {
        fn iter(&self) -> impl Iterator<Item = &i32> {
            self.data.iter()
        }
    }

    let a = Owned::new(0);
    let b = Owned::new(1);
    let c = Owned::new(2);

    let mut sum = 0;

    for (x, y, z) in itertools::izip!(a.as_view().iter(), b.as_view().iter(), c.as_view().iter()) {
        sum += x + y + z;
    }

    assert_eq!(sum, 30);
}
