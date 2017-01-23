#![feature(test)]

#[macro_use]
extern crate itertools;
extern crate test;

use std::cmp::max;

#[inline]
pub fn _max3<T: Ord>(a: T, b: T, c: T) -> T {
    max(max(a, b), c)
}

#[inline]
pub fn _max4<T: Ord>(a: T, b: T, c: T, d: T) -> T {
    max(_max3(a, b, c), d)
}

#[inline]
pub fn _max5<T: Ord>(a: T, b: T, c: T, d: T, e: T) -> T {
    max(_max4(a, b, c, d), e)
}

fn gen_vec() -> Vec<usize> {
    (0..1024).collect()
}

#[bench]
fn max2(b: &mut test::Bencher) {
    let x = gen_vec();
    b.iter(|| {
        let mut s = 0usize;
        for i in 0..x.len() - 2 {
            s = s.wrapping_add(max(x[i], x[i + 1]));
        }
        s
    })
}

#[bench]
fn max3(b: &mut test::Bencher) {
    let x = gen_vec();
    b.iter(|| {
        let mut s = 0usize;
        for i in 0..x.len() - 3 {
            s = s.wrapping_add(_max3(x[i], x[i + 1], x[i + 2]));
        }
        s
    })
}

#[bench]
fn max4(b: &mut test::Bencher) {
    let x = gen_vec();
    b.iter(|| {
        let mut s = 0usize;
        for i in 0..x.len() - 4 {
            s = s.wrapping_add(_max4(x[i], x[i + 1], x[i + 2], x[i + 3]));
        }
        s
    })
}

#[bench]
fn max5(b: &mut test::Bencher) {
    let x = gen_vec();
    b.iter(|| {
        let mut s = 0usize;
        for i in 0..x.len() - 5 {
            s = s.wrapping_add(_max5(x[i], x[i + 1], x[i + 2], x[i + 3], x[i + 4]));
        }
        s
    })
}

#[bench]
fn max2_iter(b: &mut test::Bencher) {
    let x = gen_vec();
    b.iter(|| {
        let mut s = 0usize;
        for i in 0..x.len() - 2 {
            s = s.wrapping_add(iter!(x[i], x[i + 1]).max().unwrap());
        }
        s
    })
}

#[bench]
fn max3_iter(b: &mut test::Bencher) {
    let x = gen_vec();
    b.iter(|| {
        let mut s = 0usize;
        for i in 0..x.len() - 3 {
            s = s.wrapping_add(iter!(x[i], x[i + 1], x[i + 2]).max().unwrap());
        }
        s
    })
}

#[bench]
fn max4_iter(b: &mut test::Bencher) {
    let x = gen_vec();
    b.iter(|| {
        let mut s = 0usize;
        for i in 0..x.len() - 4 {
            s = s.wrapping_add(iter!(x[i], x[i + 1], x[i + 2], x[i + 3]).max().unwrap());
        }
        s
    })
}

#[bench]
fn max5_iter(b: &mut test::Bencher) {
    let x = gen_vec();
    b.iter(|| {
        let mut s = 0usize;
        for i in 0..x.len() - 5 {
            s = s.wrapping_add(iter!(x[i], x[i + 1], x[i + 2], x[i + 3], x[i + 4])
                .max()
                .unwrap());
        }
        s
    })
}
