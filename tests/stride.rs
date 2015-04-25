extern crate itertools;

use itertools::Stride;
use itertools::StrideMut;

#[test]
fn mut_stride() {
    let mut xs = vec![1, 1, 1, 1, 1, 1];
    for x in StrideMut::from_slice(&mut *xs, 2) {
        *x = 0;
    }
    assert_eq!(xs, vec![0, 1, 0, 1, 0, 1]);
}

#[test]
fn mut_stride_compose() {
    let mut xs = vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1];
    {
        let iter1 = StrideMut::from_slice(&mut *xs, 2);
        let iter2 = StrideMut::from_stride(iter1, 3);
        for x in iter2 {
            *x = 0;
        }
    }
    assert_eq!(xs, vec![0, 1, 1, 1, 1, 1, 0, 1, 1, 1]);
}

#[test]
fn stride_uneven() {
    let xs = &[7, 9, 8];
    let mut it = Stride::from_slice(xs, 2);
    assert!(it.size_hint() == (2, Some(2)));
    assert!(*it.next().unwrap() == 7);
    assert!(*it.next().unwrap() == 8);
    assert!(it.len() == 0);
    assert!(it.next().is_none());

    let xs = &[7, 9, 8, 10];
    let mut it = Stride::from_slice(&xs[1..], 2);
    assert!(it.size_hint() == (2, Some(2)));
    assert!(*it.next().unwrap() == 9);
    assert!(*it.next().unwrap() == 10);
    assert!(it.len() == 0);
    assert!(it.next().is_none());
}

#[test]
fn stride_compose() {
    let xs = &[1, 2, 3, 4, 5, 6, 7, 8, 9];
    let odds = Stride::from_slice(xs, 2);
    let it = Stride::from_stride(odds, 2);
    let ans: Vec<isize> = it.map(|&x| x).collect();
    assert_eq!(ans, vec![1, 5, 9]);

    let xs = &[1, 2, 3, 4, 5, 6, 7, 8, 9];
    let evens = Stride::from_slice(&xs[1..], 2);
    let it = Stride::from_stride(evens, 2);
    let ans: Vec<isize> = it.map(|&x| x).collect();
    assert_eq!(ans, vec![2, 6]);

    let xs = &[1, 2, 3, 4, 5, 6, 7, 8, 9];
    let evens = Stride::from_slice(&xs[1..], 2);
    let it = Stride::from_stride(evens, 1);
    let ans: Vec<isize> = it.map(|&x| x).collect();
    assert_eq!(ans, vec![2, 4, 6, 8]);

    let xs = &[1, 2, 3, 4, 5, 6, 7, 8, 9];
    let mut odds = Stride::from_slice(xs, 2);
    odds.swap_ends();
    let it = Stride::from_stride(odds, 2);
    let ans: Vec<isize> = it.map(|&x| x).collect();
    assert_eq!(ans, vec![9, 5, 1]);

    let xs = &[1, 2, 3];
    let every = Stride::from_slice(xs, 1);
    assert_eq!(every.len(), 3);
    let odds = Stride::from_stride(every, 2);
    assert_eq!(odds.len(), 2);
    let v = odds.cloned().collect::<Vec<isize>>();
    assert_eq!(v, vec![1, 3]);

    let xs = &[1, 2, 3, 4, 5, 6, 7, 8, 9];
    let evens = Stride::from_slice(&xs[1..], 2);
    let it = Stride::from_stride(evens, -2);
    let ans: Vec<isize> = it.map(|&x| x).collect();
    assert_eq!(ans, vec![8, 4]);
}

#[test]
fn from_stride_empty()
{
    let xs = &[1, 2, 3, 4, 5, 6, 7, 8, 9];
    let mut odds = Stride::from_slice(xs, 2);
    odds.by_ref().count();
    assert!(odds.len() == 0);
    assert!(odds.next().is_none());
    let mut it = Stride::from_stride(odds, 2);
    assert!(it.len() == 0);
    assert!(it.next().is_none());
}

#[test]
fn stride() {
    let xs: &[u8]  = &[];
    let mut it = Stride::from_slice(xs, 1);
    assert!(it.size_hint() == (0, Some(0)));
    assert!(it.next().is_none());

    let xs = &[7, 9, 8, 10];
    let mut it = Stride::from_slice(xs, 2);
    assert!(it.size_hint() == (2, Some(2)));
    assert!(*it.next().unwrap() == 7);
    assert!(*it.next().unwrap() == 8);
    assert!(it.next().is_none());

    let mut it = Stride::from_slice(xs, 2).rev();
    assert!(it.size_hint() == (2, Some(2)));
    assert!(*it.next().unwrap() == 8);
    assert!(*it.next().unwrap() == 7);
    assert!(it.next().is_none());

    let xs = &[7, 9, 8, 10];
    let mut it = Stride::from_slice(xs, 1);
    assert!(it.size_hint() == (4, Some(4)));
    assert!(*it.next().unwrap() == 7);
    assert!(*it.next().unwrap() == 9);
    assert!(*it.next().unwrap() == 8);
    assert!(*it.next().unwrap() == 10);
    assert!(it.len() == 0);
    assert!(it.next().is_none());

    let mut it = Stride::from_slice(xs, 1).rev();
    assert!(it.size_hint() == (4, Some(4)));
    assert!(*it.next().unwrap() == 10);
    assert!(*it.next().unwrap() == 8);
    assert!(*it.next().unwrap() == 9);
    assert!(*it.next().unwrap() == 7);
    assert!(it.next().is_none());

    let mut it = Stride::from_slice(xs, 2);
    it.swap_ends();
    assert!(it.size_hint() == (2, Some(2)));
    assert!(*it.next().unwrap() == 8);
    assert!(*it.next().unwrap() == 7);
    assert!(it.next().is_none());

    let mut it = Stride::from_slice(xs, -2);
    assert_eq!(it.size_hint(), (2, Some(2)));
    assert_eq!(*it.next().unwrap(), 10);
    assert_eq!(*it.next().unwrap(), 9);
    assert_eq!(it.next(), None);
}

#[test]
fn stride_index() {
    let xs = &[7, 9, 8, 10];
    let it = Stride::from_slice(xs, 2);
    assert_eq!(it[0], 7);
    assert_eq!(it[1], 8);
}

#[test]
#[should_panic]
fn stride_index_fail() {
    let xs = &[7, 9, 8, 10];
    let it = Stride::from_slice(xs, 2);
    let _ = it[2];
}
