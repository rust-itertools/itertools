#![cfg(feature = "lending_iters")]

use std::collections::{HashSet, VecDeque};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use itertools::Itertools;
use itertools::LendingIterator;

// approximate 100_000 iterations for each combination
const N1: usize = 100_000;
const N2: usize = 448;
const N3: usize = 86;
const N4: usize = 41;
const N14: usize = 21;

fn comb_lending_c1(c: &mut Criterion) {
    c.bench_function("comb lending c1", move |b| {
        b.iter(|| {
            (0..N1).combinations_lending(1).for_each(|combo| {
                black_box({
                    let mut out = Vec::with_capacity(1);
                    out.extend(combo);
                    out
                });
            });
        })
    });
}

fn comb_lending_c2(c: &mut Criterion) {
    c.bench_function("comb lending c2", move |b| {
        b.iter(|| {
            (0..N2).combinations_lending(2).for_each(|combo| {
                black_box({
                    let mut out = Vec::with_capacity(2);
                    out.extend(combo);
                    out
                });
            });
        })
    });
}

fn comb_lending_c3(c: &mut Criterion) {
    c.bench_function("comb lending c3", move |b| {
        b.iter(|| {
            (0..N3).combinations_lending(3).for_each(|combo| {
                black_box({
                    let mut out = Vec::with_capacity(3);
                    out.extend(combo);
                    out
                });
            });
        })
    });
}

fn comb_lending_c4(c: &mut Criterion) {
    c.bench_function("comb lending c4", move |b| {
        b.iter(|| {
            (0..N4).combinations_lending(4).for_each(|combo| {
                black_box({
                    let mut out = Vec::with_capacity(4);
                    out.extend(combo);
                    out
                });
            });
        })
    });
}

fn comb_lending_c14(c: &mut Criterion) {
    c.bench_function("comb lending c14", move |b| {
        b.iter(|| {
            (0..N14).combinations_lending(14).for_each(|combo| {
                black_box({
                    let mut out = Vec::with_capacity(14);
                    out.extend(combo);
                    out
                });
            });
        })
    });
}

fn comb_lending_single_use(c: &mut Criterion) {
    c.bench_function("comb lending single use", move |b| {
        b.iter(|| {
            let mut combination_bitmask = 0usize;
            (0..N14).combinations_lending(14).for_each(|combo| {
                let compared_bitmask = 0b101010101010101011110000usize;
                combo.for_each(|bit_pos| {
                    combination_bitmask |= 1 << bit_pos;
                });
                black_box((combination_bitmask & compared_bitmask).count_ones());
            });
        })
    });
}

fn comb_lending_into_hash_set_from_collect(c: &mut Criterion) {
    c.bench_function("comb lending into hash set from collect", move |b| {
        b.iter(|| {
            (0..N14).combinations_lending(14).for_each(|combo| {
                black_box(combo.collect::<HashSet<_>>());
            });
        })
    });
}

fn comb_lending_into_hash_set_from_extend(c: &mut Criterion) {
    c.bench_function("comb lending into hash set from extend", move |b| {
        b.iter(|| {
            (0..N14).combinations_lending(14).for_each(|combo| {
                black_box({
                    let mut out = HashSet::with_capacity(14);
                    out.extend(combo);
                    out
                });
            });
        })
    });
}

fn comb_lending_into_vec_deque_from_collect(c: &mut Criterion) {
    c.bench_function("comb lending into vec deque from collect", move |b| {
        b.iter(|| {
            (0..N14).combinations_lending(14).for_each(|combo| {
                black_box(combo.collect::<VecDeque<_>>());
            });
        })
    });
}

fn comb_lending_into_vec_deque_from_extend(c: &mut Criterion) {
    c.bench_function("comb lending into vec deque from extend", move |b| {
        b.iter(|| {
            (0..N14).combinations_lending(14).for_each(|combo| {
                black_box({
                    let mut out = VecDeque::with_capacity(14);
                    out.extend(combo);
                    out
                });
            });
        })
    });
}

fn comb_lending_into_slice(c: &mut Criterion) {
    c.bench_function("comb lending into slice", move |b| {
        b.iter(|| {
            (0..N14).combinations_lending(14).for_each(|mut combo| {
                black_box({
                    let mut out = [0; 14];
                    out.fill_with(|| combo.next().unwrap_or_default());
                    out
                });
            });
        })
    });
}

fn comb_lending_into_slice_unchecked(c: &mut Criterion) {
    c.bench_function("comb lending into slice unchecked", move |b| {
        b.iter(|| {
            (0..N14).combinations_lending(14).for_each(|mut combo| {
                black_box({
                    let mut out = [0; 14];
                    out.fill_with(|| combo.next().unwrap());
                    out
                });
            });
        })
    });
}

criterion_group!(
    benches,
    comb_lending_c1,
    comb_lending_c2,
    comb_lending_c3,
    comb_lending_c4,
    comb_lending_c14,
    comb_lending_single_use,
    comb_lending_into_hash_set_from_collect,
    comb_lending_into_hash_set_from_extend,
    comb_lending_into_vec_deque_from_collect,
    comb_lending_into_vec_deque_from_extend,
    comb_lending_into_slice,
    comb_lending_into_slice_unchecked,
);

criterion_main!(benches);
