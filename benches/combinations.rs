use std::collections::{HashSet, VecDeque};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use itertools::Itertools;

// approximate 100_000 iterations for each combination
const N1: usize = 100_000;
const N2: usize = 448;
const N3: usize = 86;
const N4: usize = 41;
const N14: usize = 21;

fn comb_for1(c: &mut Criterion) {
    c.bench_function("comb for1", move |b| {
        b.iter(|| {
            for i in 0..N1 {
                black_box(vec![i]);
            }
        })
    });
}

fn comb_for2(c: &mut Criterion) {
    c.bench_function("comb for2", move |b| {
        b.iter(|| {
            for i in 0..N2 {
                for j in (i + 1)..N2 {
                    black_box(vec![i, j]);
                }
            }
        })
    });
}

fn comb_for3(c: &mut Criterion) {
    c.bench_function("comb for3", move |b| {
        b.iter(|| {
            for i in 0..N3 {
                for j in (i + 1)..N3 {
                    for k in (j + 1)..N3 {
                        black_box(vec![i, j, k]);
                    }
                }
            }
        })
    });
}

fn comb_for4(c: &mut Criterion) {
    c.bench_function("comb for4", move |b| {
        b.iter(|| {
            for i in 0..N4 {
                for j in (i + 1)..N4 {
                    for k in (j + 1)..N4 {
                        for l in (k + 1)..N4 {
                            black_box(vec![i, j, k, l]);
                        }
                    }
                }
            }
        })
    });
}

fn comb_c1(c: &mut Criterion) {
    c.bench_function("comb c1", move |b| {
        b.iter(|| {
            for combo in (0..N1).combinations(1) {
                black_box(combo);
            }
        })
    });
}

fn comb_c2(c: &mut Criterion) {
    c.bench_function("comb c2", move |b| {
        b.iter(|| {
            for combo in (0..N2).combinations(2) {
                black_box(combo);
            }
        })
    });
}

fn comb_c3(c: &mut Criterion) {
    c.bench_function("comb c3", move |b| {
        b.iter(|| {
            for combo in (0..N3).combinations(3) {
                black_box(combo);
            }
        })
    });
}

fn comb_c4(c: &mut Criterion) {
    c.bench_function("comb c4", move |b| {
        b.iter(|| {
            for combo in (0..N4).combinations(4) {
                black_box(combo);
            }
        })
    });
}

fn comb_c14(c: &mut Criterion) {
    c.bench_function("comb c14", move |b| {
        b.iter(|| {
            for combo in (0..N14).combinations(14) {
                black_box(combo);
            }
        })
    });
}

fn comb_single_use(c: &mut Criterion) {
    c.bench_function("comb single use", move |b| {
        b.iter(|| {
            let mut combination_bitmask = 0usize;
            (0..N14).combinations(14).for_each(|combo| {
                let compared_bitmask = 0b101010101010101011110000usize;
                combo.into_iter().for_each(|bit_pos| {
                    combination_bitmask |= 1 << bit_pos;
                });
                black_box((combination_bitmask & compared_bitmask).count_ones());
            });
        })
    });
}

fn comb_into_hash_set(c: &mut Criterion) {
    c.bench_function("comb into hash set", move |b| {
        b.iter(|| {
            (0..N14).combinations(14).for_each(|combo| {
                black_box({
                    let mut out = HashSet::with_capacity(14);
                    out.extend(combo);
                    out
                });
            });
        })
    });
}

fn comb_into_vec_deque(c: &mut Criterion) {
    c.bench_function("comb into vec deque", move |b| {
        b.iter(|| {
            (0..N14).combinations(14).for_each(|combo| {
                black_box(VecDeque::from(combo));
            });
        })
    });
}

fn comb_into_slice(c: &mut Criterion) {
    c.bench_function("comb into slice", move |b| {
        b.iter(|| {
            (0..N14).combinations(14).for_each(|combo| {
                black_box({
                    let mut out = [0; 14];
                    let mut combo_iter = combo.into_iter();
                    out.fill_with(|| combo_iter.next().unwrap_or_default());
                    out
                });
            });
        })
    });
}

fn comb_into_slice_unchecked(c: &mut Criterion) {
    c.bench_function("comb into slice unchecked", move |b| {
        b.iter(|| {
            (0..N14).combinations(14).for_each(|combo| {
                black_box({
                    let mut out = [0; 14];
                    let mut combo_iter = combo.into_iter();
                    out.fill_with(|| combo_iter.next().unwrap());
                    out
                });
            });
        })
    });
}

fn comb_into_slice_for_loop(c: &mut Criterion) {
    c.bench_function("comb into slice for loop", move |b| {
        b.iter(|| {
            (0..N14).combinations(14).for_each(|combo| {
                black_box({
                    let mut out = [0; 14];
                    for (i, elem) in combo.into_iter().enumerate() {
                        out[i] = elem;
                    }
                    out
                });
            });
        })
    });
}

criterion_group!(
    benches,
    comb_for1,
    comb_for2,
    comb_for3,
    comb_for4,
    comb_c1,
    comb_c2,
    comb_c3,
    comb_c4,
    comb_c14,
    comb_single_use,
    comb_into_hash_set,
    comb_into_vec_deque,
    comb_into_slice,
    comb_into_slice_unchecked,
    comb_into_slice_for_loop,
);

criterion_main!(benches);
