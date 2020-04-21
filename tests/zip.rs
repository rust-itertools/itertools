use itertools::Itertools;
use itertools::EitherOrBoth::{Both, Left, Right};
use itertools::free::zip_eq;

#[test]
fn zip_longest_fused() {
    let a = [Some(1), None, Some(3), Some(4)];
    let b = [1, 2, 3];

    let unfused = a.iter().batching(|it| *it.next().unwrap())
        .zip_longest(b.iter().cloned());
    itertools::assert_equal(unfused,
                       vec![Both(1, 1), Right(2), Right(3)]);
}

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


#[should_panic]
#[test]
fn zip_eq_panic1()
{
    let a = [1, 2];
    let b = [1, 2, 3];

    zip_eq(&a, &b).count();
}

#[should_panic]
#[test]
fn zip_eq_panic2()
{
    let a: [i32; 0] = [];
    let b = [1, 2, 3];

    zip_eq(&a, &b).count();
}

#[test]
fn zip_with_maxes() {
    let a = [1, 2, 3, 4, 5, -1, 100];
    let b = [7, -3, 4, 10, -2, 10, 90];
    let maxes = a.iter().zip_with(b.iter(), std::cmp::max);
    itertools::assert_equal(maxes, &[7, 2, 4, 10, 5, 10, 100]);
}

#[test]
fn zip_with_short1() {
    let a = [4, 5];
    let b = [3, 7, 9];
    let sums = a.iter().zip_with(b.iter(), |x,y| x + y);
    itertools::assert_equal(sums, vec![7, 12]);
}

#[test]
fn zip_with_short2() {
    let a = [4, 5, 7, 11];
    let b = [1];
    let sums = a.iter().zip_with(b.iter(), |x,y| x - y);
    itertools::assert_equal(sums, vec![3]);
}

#[test]
fn zip_with_size_hint1() {
    let a = [1, 2, 3];
    let b = [1, 2, 3, 5, 7, 9];
    assert_eq!(a.iter().zip_with(b.iter(), std::cmp::min).size_hint(), (3, Some(3)));
}

#[test]
fn zip_with_size_hint2() {
    let a = [-2, 5, 6, 7, 8, 9];
    let b = [-1];
    assert_eq!(a.iter().zip_with(b.iter(), std::cmp::min).size_hint(), (1, Some(1)));
}