extern crate itertools;

use itertools::Itertools;
use itertools::EitherOrBoth::{Both, Left, Right};

#[test]
fn inner_fused() {
    let a = 0..3;
    let b = 2..5;
    let mut it = a.merge_join_inner_by(b, |x, y| Ord::cmp(&x, &y));
    assert_eq!(it.next(), Some((2, 2)));
    assert_eq!(it.next(), None);
}
#[test]
fn inner_fused_inv() {
    let a = 2..5;
    let b = 0..3;
    let mut it = a.merge_join_inner_by(b, |x, y| Ord::cmp(&x, &y));
    assert_eq!(it.next(), Some((2, 2)));
    assert_eq!(it.next(), None);
}


#[test]
fn left_excl_fused() {
    let a = 0..3;
    let b = 2..5;
    let mut it = a.merge_join_left_excl_by(b, |x, y| Ord::cmp(&x, &y));
    assert_eq!(it.next(), Some(0));
    assert_eq!(it.next(), Some(1));
    assert_eq!(it.next(), None);
}
#[test]
fn left_excl_fused_inv() {
    let a = 2..5;
    let b = 0..3;
    let mut it = a.merge_join_left_excl_by(b, |x, y| Ord::cmp(&x, &y));
    assert_eq!(it.next(), Some(3));
    assert_eq!(it.next(), Some(4));
    assert_eq!(it.next(), None);
}

#[test]
fn left_outer_fused() {
    let a = 0..3;
    let b = 2..5;
    let mut it = a.merge_join_left_outer_by(b, |x, y| Ord::cmp(&x, &y));
    assert_eq!(it.next(), Some(Left(0)));
    assert_eq!(it.next(), Some(Left(1)));
    assert_eq!(it.next(), Some(Both(2, 2)));
    assert_eq!(it.next(), None);
}
#[test]
fn left_outer_fused_inv() {
    let a = 2..5;
    let b = 0..3;
    let mut it = a.merge_join_left_outer_by(b, |x, y| Ord::cmp(&x, &y));
    assert_eq!(it.next(), Some(Both(2, 2)));
    assert_eq!(it.next(), Some(Left(3)));
    assert_eq!(it.next(), Some(Left(4)));
    assert_eq!(it.next(), None);
}

#[test]
fn full_outer_fused() {
    let a = 0..3;
    let b = 2..5;
    let mut it = a.merge_join_full_outer_by(b, |x, y| Ord::cmp(&x, &y));
    assert_eq!(it.next(), Some(Left(0)));
    assert_eq!(it.next(), Some(Left(1)));
    assert_eq!(it.next(), Some(Both(2, 2)));
    assert_eq!(it.next(), Some(Right(3)));
    assert_eq!(it.next(), Some(Right(4)));
    assert_eq!(it.next(), None);
}
#[test]
fn full_outer_fused_inv() {
    let a = 2..5;
    let b = 0..3;
    let mut it = a.merge_join_full_outer_by(b, |x, y| Ord::cmp(&x, &y));
    assert_eq!(it.next(), Some(Right(0)));
    assert_eq!(it.next(), Some(Right(1)));
    assert_eq!(it.next(), Some(Both(2, 2)));
    assert_eq!(it.next(), Some(Left(3)));
    assert_eq!(it.next(), Some(Left(4)));
    assert_eq!(it.next(), None);
}
