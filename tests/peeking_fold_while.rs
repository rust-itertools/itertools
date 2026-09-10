extern crate itertools;

use itertools::Itertools;
use itertools::{put_back, put_back_n};

#[test]
fn peeking_fold_while_peekable_consumes_all() {
    let a = [10, 20, 30];
    let mut it = a.iter().peekable();
    let sum = it.peeking_fold_while(0i8, |acc, &&x| acc.checked_add(x).ok_or(acc));
    assert_eq!(sum, Ok(60));
    assert_eq!(it.next(), None);
}

#[test]
fn peeking_fold_while_peekable_consumes_some() {
    let a = [10, 20, 30, 100, 40, 50];
    let mut it = a.iter().peekable();
    let sum = it.peeking_fold_while(0i8, |acc, &&x| acc.checked_add(x).ok_or(acc));
    assert_eq!(sum, Err(60));
    assert_eq!(it.next(), Some(&100));
}

#[test]
fn peeking_fold_while_put_back_consumes_all() {
    let a = [10, 20, 30];
    let mut it = put_back(a.iter());
    let sum = it.peeking_fold_while(0i8, |acc, &&x| acc.checked_add(x).ok_or(acc));
    assert_eq!(sum, Ok(60));
    assert_eq!(it.next(), None);
}

#[test]
fn peeking_fold_while_put_back_consumes_some() {
    let a = [10, 20, 30, 100, 40, 50];
    let mut it = put_back(a.iter());
    let sum = it.peeking_fold_while(0i8, |acc, &&x| acc.checked_add(x).ok_or(acc));
    assert_eq!(sum, Err(60));
    assert_eq!(it.next(), Some(&100));
}

#[test]
fn peeking_fold_while_put_back_n_consumes_all() {
    let a = [10, 20, 30];
    let mut it = put_back_n(a.iter());
    let sum = it.peeking_fold_while(0i8, |acc, &&x| acc.checked_add(x).ok_or(acc));
    assert_eq!(sum, Ok(60));
    assert_eq!(it.next(), None);
}

#[test]
fn peeking_fold_while_put_back_n_consumes_some() {
    let a = [10, 20, 30, 100, 40, 50];
    let mut it = put_back(a.iter());
    let sum = it.peeking_fold_while(0i8, |acc, &&x| acc.checked_add(x).ok_or(acc));
    assert_eq!(sum, Err(60));
    assert_eq!(it.next(), Some(&100));
}
