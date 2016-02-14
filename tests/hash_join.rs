extern crate itertools;

use std::collections::HashSet;
use itertools::Itertools;
use itertools::EitherOrBoth::{Both, Left, Right};

#[test]
fn inner_fused() {
    let a = (0..3).zip(0..3);
    let b = (2..5).zip(2..5);
    let mut it = a.hash_join_inner(b);
    assert_eq!(it.next(), Some((2, 2)));
    assert_eq!(it.next(), None);
}
#[test]
fn inner_fused_inv() {
    let a = (2..5).zip(2..5);
    let b = (0..3).zip(0..3);
    let mut it = a.hash_join_inner(b);
    assert_eq!(it.next(), Some((2, 2)));
    assert_eq!(it.next(), None);
}


#[test]
fn left_excl_fused() {
    let a = (0..3).zip(0..3);
    let b = (2..5).zip(2..5);
    let mut it = a.hash_join_left_excl(b);
    assert_eq!(it.next(), Some(0));
    assert_eq!(it.next(), Some(1));
    assert_eq!(it.next(), None);
}
#[test]
fn left_excl_fused_inv() {
    let a = (2..5).zip(2..5);
    let b = (0..3).zip(0..3);
    let mut it = a.hash_join_left_excl(b);
    assert_eq!(it.next(), Some(3));
    assert_eq!(it.next(), Some(4));
    assert_eq!(it.next(), None);
}

#[test]
fn left_outer_fused() {
    let a = (0..3).zip(0..3);
    let b = (2..5).zip(2..5);
    let mut it = a.hash_join_left_outer(b);
    assert_eq!(it.next(), Some(Left(0)));
    assert_eq!(it.next(), Some(Left(1)));
    assert_eq!(it.next(), Some(Both(2, 2)));
    assert_eq!(it.next(), None);
}
#[test]
fn left_outer_fused_inv() {
    let a = (2..5).zip(2..5);
    let b = (0..3).zip(0..3);
    let mut it = a.hash_join_left_outer(b);
    assert_eq!(it.next(), Some(Both(2, 2)));
    assert_eq!(it.next(), Some(Left(3)));
    assert_eq!(it.next(), Some(Left(4)));
    assert_eq!(it.next(), None);
}

#[test]
fn right_excl_fused() {
    let a = (0..3).zip(0..3);
    let b = (2..5).zip(2..5);
    let mut it = a.hash_join_right_excl(b);
    let right_values: HashSet<u64> = it.by_ref().take(2).collect();
    assert!(right_values.contains(&3));
    assert!(right_values.contains(&4));
    assert_eq!(it.next(), None);
}
#[test]
fn right_excl_fused_inv() {
    let a = (2..5).zip(2..5);
    let b = (0..3).zip(0..3);
    let mut it = a.hash_join_right_excl(b);
    let right_values: HashSet<u64> = it.by_ref().take(2).collect();
    assert!(right_values.contains(&0));
    assert!(right_values.contains(&1));
    assert_eq!(it.next(), None);
}

#[test]
fn right_outer_fused() {
    let a = (0..3).zip(0..3);
    let b = (2..5).zip(2..5);
    let mut it = a.hash_join_right_outer(b);
    assert_eq!(it.next(), Some(Both(2, 2)));
    let right_values: HashSet<u64> = it.by_ref()
        .take(2)
        .map(|e| match e {
                    Right(r) => return r,
                    _ => panic!("Expected Right variant"),
             })
        .collect();
    assert!(right_values.contains(&3));
    assert!(right_values.contains(&4));
    assert_eq!(it.next(), None);
}
#[test]
fn right_outer_fused_inv() {
    let a = (2..5).zip(2..5);
    let b = (0..3).zip(0..3);
    let mut it = a.hash_join_right_outer(b);
    assert_eq!(it.next(), Some(Both(2, 2)));
    let right_values: HashSet<u64> = it.by_ref()
        .take(2)
        .map(|e| match e {
                    Right(r) => return r,
                    _ => panic!("Expected Right variant"),
             })
        .collect();
    assert!(right_values.contains(&0));
    assert!(right_values.contains(&1));
    assert_eq!(it.next(), None);
}

#[test]
fn full_outer_fused() {
    let a = (0..3).zip(0..3);
    let b = (2..5).zip(2..5);
    let mut it = a.hash_join_full_outer(b);
    assert_eq!(it.next(), Some(Left(0)));
    assert_eq!(it.next(), Some(Left(1)));
    assert_eq!(it.next(), Some(Both(2, 2)));
    let right_values: HashSet<u64> = it.by_ref()
        .take(2)
        .map(|e| match e {
                    Right(r) => return r,
                    _ => panic!("Expected Right variant"),
             })
        .collect();
    assert!(right_values.contains(&3));
    assert!(right_values.contains(&4));
    assert_eq!(it.next(), None);
}

#[test]
fn full_outer_fused_inv() {
    let a = (2..5).zip(2..5);
    let b = (0..3).zip(0..3);
    let mut it = a.hash_join_full_outer(b);
    assert_eq!(it.next(), Some(Both(2, 2)));
    assert_eq!(it.next(), Some(Left(3)));
    assert_eq!(it.next(), Some(Left(4)));
    let right_values: HashSet<u64> = it.by_ref()
        .take(2)
        .map(|e| match e {
                    Right(r) => return r,
                    _ => panic!("Expected Right variant"),
             })
        .collect();
    assert!(right_values.contains(&0));
    assert!(right_values.contains(&1));
    assert_eq!(it.next(), None);
}
