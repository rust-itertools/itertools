//! Licensed under the Apache License, Version 2.0
//! http://www.apache.org/licenses/LICENSE-2.0 or the MIT license
//! http://opensource.org/licenses/MIT, at your
//! option. This file may not be copied, modified, or distributed
//! except according to those terms.

#![feature(slicing_syntax)]
#![feature(unboxed_closures)]

#[macro_use]
extern crate itertools;

extern crate test;

use std::fmt::Show;
use std::iter::order;
use itertools::Itertools;
use itertools::Interleave;
use itertools::Zip;

use itertools as it;

fn assert_iters_equal<
    A: PartialEq + Show,
    I: Iterator<Item=A>,
    J: Iterator<Item=A>>(mut it: I, mut jt: J)
{
    loop {
        let elti = it.next();
        let eltj = jt.next();
        assert_eq!(elti, eltj);
        if elti.is_none() { break; }
    }
}

#[test]
fn product2() {
    let s = "αβ";

    let mut prod = iproduct!(s.chars(), range(0, 2));
    assert!(prod.next() == Some(('α', 0)));
    assert!(prod.next() == Some(('α', 1)));
    assert!(prod.next() == Some(('β', 0)));
    assert!(prod.next() == Some(('β', 1)));
    assert!(prod.next() == None);
}

#[test]
fn product3() {
    let prod = iproduct!(range(0, 3), range(0, 2), range(0, 2));
    assert_eq!(prod.size_hint(), (12, Some(12)));
    let v = prod.collect_vec();
    for i in range(0,3) {
        for j in range(0, 2) {
            for k in range(0, 2) {
                assert!((i, j, k) == v[(i * 2 * 2 + j * 2 + k) as usize]);
            }
        }
    }
    for (_, _, _, _) in iproduct!(range(0, 3), range(0, 2), range(0, 2), range(0, 3)) {
        /* test compiles */
    }
}

#[test]
fn izip3() {
    let mut zip = Zip::new((range(0, 3), range(0, 2), range(0, 2i8)));
    for i in range(0, 2) {
        assert!((i as usize, i, i as i8) == zip.next().unwrap());
    }
    assert!(zip.next().is_none());

    
    let xs: [isize; 0] = [];
    let mut zip = Zip::new((range(0, 3), range(0, 2), range(0, 2i8), xs.iter()));
    assert!(zip.next().is_none());

    for (_, _, _, _) in Zip::new((range(0, 3), range(0, 2), range(0, 2), range(0, 3))) {
        /* test compiles */
    }
}

#[test]
fn write_to() {
    let xs = [7, 9, 8];
    let mut ys = [0; 5];
    let cnt = it::write(ys.iter_mut(), xs.iter().map(|x| *x));
    assert!(cnt == xs.len());
    assert!(ys == [7, 9, 8, 0, 0]);

    let cnt = it::write(ys.iter_mut(), range(0,10));
    assert!(cnt == ys.len());
    assert!(ys == [0, 1, 2, 3, 4]);
}

#[test]
fn interleave() {
    let xs: [u8; 0]  = [];
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
    let xs = [1i32,2,3];
    let mut sum = 0;
    xs.iter().map(|elt| sum += *elt).drain();
    assert!(sum == 6);
}

#[test]
fn apply() {
    let xs = [1i32, 2, 3];
    let mut sum = 0;
    xs.iter().apply(|elt| sum += *elt);
    assert!(sum == 6);
}

#[test]
fn dropn() {
    let xs = [1, 2, 3];
    let mut it = xs.iter();
    assert!(it.dropn(2) == 2);
    assert!(it.next().is_some());
    assert!(it.next().is_none());
    let mut it = xs.iter();
    assert!(it.dropn(5) == 3);
    assert!(it.next().is_none());
}

#[test]
fn dropping() {
    let xs = [1, 2, 3];
    let mut it = xs.iter().dropping(2);
    assert!(it.next().is_some());
    assert!(it.next().is_none());
    let mut it = xs.iter().dropping(5);
    assert!(it.next().is_none());
}

#[test]
fn intersperse() {
    let xs = ["a", "", "b", "c"];
    let v: Vec<&str> = xs.iter().map(|x| x.clone()).intersperse(", ").collect();
    let text: String = v.concat();
    assert_eq!(text, "a, , b, c".to_string());

    let ys = [0, 1, 2, 3];
    let mut it = ys.slice_to(0).iter().map(|x| *x).intersperse(1);
    assert!(it.next() == None);
}

#[test]
fn linspace() {
    let mut iter = it::linspace::<f32>(0., 2., 3);
    assert_eq!(iter.next(), Some(0.0));
    assert_eq!(iter.next(), Some(1.0));
    assert_eq!(iter.next(), Some(2.0));
    assert_eq!(iter.next(), None);

    let mut iter = it::linspace::<f32>(0., -2., 4);
    assert_eq!(iter.next(), Some(0.));
    assert_eq!(iter.next(), Some(-0.666666666667));
    assert_eq!(iter.next(), Some(-1.333333333333));
    assert_eq!(iter.next(), Some(-2.));
    assert_eq!(iter.next(), None);

    let mut iter = it::linspace::<f32>(0., 1., 1);
    assert_eq!(iter.next(), Some(0.));
    assert_eq!(iter.next(), None);

    let mut iter = it::linspace::<f32>(0., 1., 0);
    assert_eq!(iter.next(), None);
}

#[test]
fn dedup() {
    let xs = [0, 1, 1, 1, 2, 1, 3, 3];
    let ys = [0, 1, 2, 1, 3];
    assert_iters_equal(ys.iter(), xs.iter().dedup());
    let xs = [0, 0, 0, 0, 0];
    let ys = [0];
    assert_iters_equal(ys.iter(), xs.iter().dedup());
}

#[test]
fn batching() {
    let xs = [0, 1, 2, 1, 3];
    let ys = [(0, 1), (2, 1)];

    // An iterator that gathers elements up in pairs
    let pit = xs.iter().cloned().batching(|mut it| {
               match it.next() {
                   None => None,
                   Some(x) => match it.next() {
                       None => None,
                       Some(y) => Some((x, y)),
                   }
               }
           });
    assert_iters_equal(pit, ys.iter().cloned());
}

#[test]
fn group_by() {
    let xs = [0, 1, 1, 1, 2, 1, 3, 3];
    let ans = vec![(0, vec![0]), (1, vec![1, 1, 1]),
                   (2, vec![2]), (1, vec![1]), (3, vec![3, 3])];
    
    let gb = xs.iter().cloned().group_by(|elt| *elt);
    assert_iters_equal(gb, ans.into_iter());
}

#[test]
fn put_back() {
    let xs = [0, 1, 1, 1, 2, 1, 3, 3];
    let mut pb = it::PutBack::new(xs.iter().cloned());
    pb.next();
    pb.put_back(1);
    pb.put_back(0);
    assert_iters_equal(pb, xs.iter().cloned());
}

#[test]
fn tee() {
    let xs  = [0, 1, 2, 3];
    let (mut t1, mut t2) = xs.iter().cloned().tee();
    assert_eq!(t1.next(), Some(0));
    assert_eq!(t2.next(), Some(0));
    assert_eq!(t1.next(), Some(1));
    assert_eq!(t1.next(), Some(2));
    assert_eq!(t1.next(), Some(3));
    assert_eq!(t1.next(), None);
    assert_eq!(t2.next(), Some(1));
    assert_eq!(t2.next(), Some(2));
    assert_eq!(t1.next(), None);
    assert_eq!(t2.next(), Some(3));
    assert_eq!(t2.next(), None);
    assert_eq!(t1.next(), None);
    assert_eq!(t2.next(), None);

    let (t1, t2) = xs.iter().cloned().tee();
    assert_iters_equal(t1, xs.iter().cloned());
    assert_iters_equal(t2, xs.iter().cloned());

    let (t1, t2) = xs.iter().cloned().tee();
    assert_iters_equal(t1.zip(t2), xs.iter().cloned().zip(xs.iter().cloned()));
}


#[test]
fn rciter() {
    let xs = [0, 1, 1, 1, 2, 1, 3, 5, 6];

    let mut r1 = xs.iter().cloned().into_rc();
    let mut r2 = r1.clone();
    assert_eq!(r1.next(), Some(0));
    assert_eq!(r2.next(), Some(1));
    let mut z = r1.zip(r2);
    assert_eq!(z.next(), Some((1, 1)));
    assert_eq!(z.next(), Some((2, 1)));
    assert_eq!(z.next(), Some((3, 5)));
    assert_eq!(z.next(), None);
}

#[test]
fn slice() {

    let it = 0..10;
    assert_iters_equal(it.slice(..3), 0..3);
    assert_iters_equal(it.slice(3..7), 3..7);
    assert_iters_equal(it.slice(3..27), 3..10);
    assert_iters_equal(it.slice(44..), 0..0);
}

#[test]
fn step() {
    let it = 0..10;
    assert_iters_equal(it.step(1), it);
    assert_iters_equal(it.step(2), it.filter(|x| *x % 2 == 0));
    assert_iters_equal(it.step(10), 0..1);
}
