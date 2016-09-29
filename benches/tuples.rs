#![feature(test)]

extern crate test;
extern crate itertools;

use test::{black_box, Bencher};
use itertools::Itertools;

#[bench]
fn tuples_2(b: &mut Bencher) {
    b.iter(|| {
        for (a, b) in (0..20_000).tuples() {
            black_box(a + b);
        }
    })
}

#[bench]
fn tuples_3(b: &mut Bencher) {
    b.iter(|| {
        for (a, b, c) in (0..30_000).tuples() {
            black_box(a + b + c);
        }
    })
}

#[bench]
fn tuples_4(b: &mut Bencher) {
    b.iter(|| {
        for (a, b, c, d) in (0..40_000).tuples() {
            black_box(a + b + c + d);
        }
    })
}

#[bench]
fn tuple_windows_2(b: &mut Bencher) {
    b.iter(|| {
        for (a, b) in (0..10_000).tuple_windows() {
            black_box(a + b);
        }
    })
}

#[bench]
fn tuple_windows_3(b: &mut Bencher) {
    b.iter(|| {
        for (a, b, c) in (0..10_000).tuple_windows() {
            black_box(a + b + c);
        }
    })
}

#[bench]
fn tuple_windows_4(b: &mut Bencher) {
    b.iter(|| {
        for (a, b, c, d) in (0..10_000).tuple_windows() {
            black_box(a + b + c + d);
        }
    })
}
