#![feature(
    core,
    )]

extern crate itertools;

use std::iter::count;
use std::iter::RandomAccessIterator;
use itertools::Itertools;
use itertools::EitherOrBoth::{Both, Left};
use itertools::Zip;

#[test]
fn test_zip_longest_size_hint() {
    let c = count(0, 1);
    let v: &[_] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
    let v2 = &[10, 11, 12];
    let vi = v.iter();

    assert_eq!(c.zip_longest(vi).size_hint(), (std::usize::MAX, None));

    assert_eq!(vi.zip_longest(v2.iter()).size_hint(), (10, Some(10)));
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


// This function copied from std::iter in rust-lang/rust
#[cfg(test)]
fn check_randacc_iter<A: PartialEq, T: Clone + Iterator<Item=A> + RandomAccessIterator>(a: T, len: usize)
{
    let mut b = a.clone();
    assert_eq!(len, b.indexable());
    let mut n = 0;
    for (i, elt) in a.enumerate() {
        assert!(Some(elt) == b.idx(i));
        n += 1;
    }
    assert_eq!(n, len);
    assert!(None == b.idx(n));
    // call recursively to check after picking off an element
    if len > 0 {
        b.next();
        check_randacc_iter(b, len-1);
    }
}
#[test]
fn test_random_access_zip_longest() {
    let xs = [1, 2, 3, 4, 5];
    let ys = [7, 9, 11];
    check_randacc_iter(xs.iter().zip_longest(ys.iter()), std::cmp::max(xs.len(), ys.len()));
}

#[test]
fn zip_tuple() {
    let xs = [1, 2, 3];
    let ys = b"ab";
    let mut it = Zip::new((xs.iter().cloned(), ));
    assert!(it.next() != None);
    let mut jt = Zip::new((xs.iter().cloned(), ys.iter().cloned()));
    assert_eq!(jt.next(), Some((1, b'a')));
    assert_eq!(jt.next(), Some((2, b'b')));
    assert_eq!(jt.next(), None);

    let mut jt = Zip::new((xs.iter().cloned(), xs.iter().cloned(), xs.iter().cloned()));
    assert_eq!(jt.next(), Some((1, 1, 1)));
}
