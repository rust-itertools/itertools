//! Licensed under the Apache License, Version 2.0
//! http://www.apache.org/licenses/LICENSE-2.0 or the MIT license
//! http://opensource.org/licenses/MIT, at your
//! option. This file may not be copied, modified, or distributed
//! except according to those terms.

#![feature(phase)]

#[phase(plugin, link)] extern crate itertools;

extern crate test;

use std::iter::order;
use itertools::Itertools;
use itertools::Interleave;
use itertools::BoxIter;

use itertools::ItertoolsClonable;

use itertools as it;

#[test]
fn product2() {
    let s = "αβ";

    let mut prod = iproduct!(s.chars(), range(0, 2i));
    assert!(prod.next() == Some(('α', 0)));
    assert!(prod.next() == Some(('α', 1)));
    assert!(prod.next() == Some(('β', 0)));
    assert!(prod.next() == Some(('β', 1)));
    assert!(prod.next() == None);
}

#[test]
fn product3() {
    let mut prod = iproduct!(range(0, 3i), range(0, 2i), range(0, 2i));
    assert_eq!(prod.size_hint(), (12, Some(12)));
    let v = prod.collect::<Vec<_>>();
    for i in range(0,3i) {
        for j in range(0, 2i) {
            for k in range(0, 2i) {
                assert!((i, j, k) == v[(i * 2 * 2 + j * 2 + k) as uint]);
            }
        }
    }
    for (a, b, c, d) in iproduct!(range(0, 3i), range(0, 2i), range(0, 2i), range(0, 3i)) {
        /* test compiles */
    }
}

#[test]
fn izip3() {
    let mut zip = izip!(range(0, 3u), range(0, 2i), range(0, 2i8));
    for i in range(0, 2i) {
        assert!((i as uint, i, i as i8) == zip.next().unwrap());
    }
    assert!(zip.next().is_none());

    
    let xs: [int, ..0] = [];
    let mut zip = izip!(range(0, 3u), range(0, 2i), range(0, 2i8), xs.iter());
    assert!(zip.next().is_none());

    for (a, b, c, d) in izip!(range(0, 3i), range(0, 2i), range(0, 2i), range(0, 3i)) {
        /* test compiles */
    }
}

#[test]
fn fn_map() {
    let xs = [0, 1, 2i];
    fn mapper(x: &int) -> String { x.to_string() }
    let it = xs.iter().fn_map(mapper);
    let jt = it.clone();
    assert!(it.zip(jt).all(|(x,y)| x == y));
}

#[test]
fn write_to() {
    let xs = [7i, 9, 8];
    let mut ys = [0i, ..5];
    let cnt = it::write(ys.iter_mut(), xs.iter().map(|x| *x));
    assert!(cnt == xs.len());
    assert!(ys == &[7i, 9, 8, 0, 0]);

    let cnt = it::write(ys.iter_mut(), range(0,10i));
    assert!(cnt == ys.len());
    assert!(ys == &[0, 1, 2, 3, 4]);
}

#[test]
fn interleave() {
    let xs: [u8, ..0]  = [];
    let ys = [7u8, 9, 8, 10];
    let zs = [2u8, 77];
    let it = Interleave::new(xs.iter(), ys.iter());
    assert!(order::eq(it, ys.iter()));

    let rs = [7u8, 2, 9, 77, 8, 10];
    let it = Interleave::new(ys.iter(), zs.iter());
    assert!(order::eq(it, rs.iter()));
}

#[test]
fn times() {
    assert!(it::times(0).count() == 0);
    assert!(it::times(5).count() == 5);
}

#[test]
fn drain() {
    let xs = [1i,2,3];
    let mut sum = 0;
    xs.iter().map(|elt| sum += *elt).drain();
    assert!(sum == 6);
}

#[test]
fn apply() {
    let xs = [1i, 2, 3];
    let mut sum = 0;
    xs.iter().apply(|elt| sum += *elt);
    assert!(sum == 6);
}

#[test]
fn dropn() {
    let xs = [1i, 2, 3];
    let mut it = xs.iter();
    assert!(it.dropn(2) == 2);
    assert!(it.next().is_some());
    assert!(it.next().is_none());
    let mut it = xs.iter();
    assert!(it.dropn(5) == 3);
    assert!(it.next().is_none());
}

#[test]
fn intersperse() {
    let xs = ["a", "", "b", "c"];
    let v: Vec<&str> = xs.iter().map(|x| x.clone()).intersperse(", ").collect();
    let text = v.concat();
    assert_eq!(text, "a, , b, c".to_string());

    let ys = [0, 1, 2, 3i];
    let mut it = ys.slice_to(0).iter().map(|x| *x).intersperse(1i);
    assert!(it.next() == None);
}

#[test]
fn clones() {
    let xs = ["a", "", "b", "c"];
    let v: Vec<&str> = xs.iter().clones().collect();
    let text = v.concat();
    assert_eq!(text, "abc".to_string());
}
