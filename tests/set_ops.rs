use std::collections::BTreeSet;

use itertools::free::{difference, intersection, symmetric_difference, union_ref};

fn from_std<F: FnOnce(BTreeSet<u32>, &BTreeSet<u32>) -> Vec<u32>>(
    left: &[u32],
    right: &[u32],
    meth: F,
) -> Vec<u32> {
    meth(
        left.to_vec().into_iter().collect::<BTreeSet<_>>(),
        &right.to_vec().into_iter().collect::<BTreeSet<_>>(),
    )
}

#[test]
fn right_only_union() {
    let left: Vec<u32> = vec![];
    let right: Vec<u32> = vec![1, 2, 3];
    let expected_result = from_std(&left, &right, |l, r| l.union(r).copied().collect());
    let actual_result = union_ref(left.iter(), right.iter())
        .copied()
        .collect::<Vec<_>>();
    assert_eq!(expected_result, actual_result);
}

#[test]
fn right_only_intersection() {
    let left: Vec<u32> = vec![];
    let right: Vec<u32> = vec![1, 2, 3];
    let expected_result = from_std(&left, &right, |l, r| l.intersection(r).copied().collect());
    let actual_result = intersection(left.iter(), right.iter())
        .copied()
        .collect::<Vec<_>>();
    assert_eq!(expected_result, actual_result);
}

#[test]
fn one_intersection() {
    let left: Vec<u32> = vec![2];
    let right: Vec<u32> = vec![1, 2, 3];
    let expected_result = from_std(&left, &right, |l, r| l.intersection(r).copied().collect());
    let actual_result = intersection(left.iter(), right.iter())
        .copied()
        .collect::<Vec<_>>();
    assert_eq!(expected_result, actual_result);
}

#[test]
fn right_only_difference() {
    let left: Vec<u32> = vec![];
    let right: Vec<u32> = vec![1, 2, 3];
    let expected_result = from_std(&left, &right, |l, r| l.difference(r).copied().collect());
    let actual_result = difference(left.iter(), right.iter())
        .copied()
        .collect::<Vec<_>>();
    assert_eq!(expected_result, actual_result);
}

#[test]
fn one_difference() {
    let left: Vec<u32> = vec![2];
    let right: Vec<u32> = vec![1, 2, 3];
    let expected_result = from_std(&left, &right, |l, r| l.difference(r).copied().collect());
    let actual_result = difference(left.iter(), right.iter())
        .copied()
        .collect::<Vec<_>>();
    assert_eq!(expected_result, actual_result);
}

#[test]
fn right_only_symmetric_difference() {
    let left: Vec<u32> = vec![];
    let right: Vec<u32> = vec![1, 2, 3];
    let expected_result = from_std(&left, &right, |l, r| {
        l.symmetric_difference(r).copied().collect()
    });
    let actual_result = symmetric_difference(left.iter(), right.iter())
        .copied()
        .collect::<Vec<_>>();
    assert_eq!(expected_result, actual_result);
}

#[test]
fn one_symmetric_difference() {
    let left: Vec<u32> = vec![2];
    let right: Vec<u32> = vec![1, 2, 3];
    let expected_result = from_std(&left, &right, |l, r| {
        l.symmetric_difference(r).copied().collect()
    });
    let actual_result = symmetric_difference(left.iter(), right.iter())
        .copied()
        .collect::<Vec<_>>();
    assert_eq!(expected_result, actual_result);
}
