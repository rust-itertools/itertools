use criterion::{black_box, criterion_group, criterion_main, Criterion};
use itertools::Itertools;

fn bench_sum(c: &mut Criterion) {
    let mut group = c.benchmark_group("fold sum accumulator");

    group.bench_function("fold", |b| {
        b.iter(|| {
            (0i64..1_000_000)
                .map(black_box)
                .fold(0, |sum, n| sum + n)
        })
    });

    group.bench_function("fold_mut", |b| {
        b.iter(|| {
            (0i64..1_000_000).map(black_box).fold_mut(0, |sum, n| {
                *sum += n;
            })
        })
    });

    group.finish();
}

fn bench_vec(c: &mut Criterion) {
    let mut group = c.benchmark_group("fold vec accumulator");

    group.bench_function("fold", |b| {
        b.iter(|| {
            (0i64..1_000_000)
                .map(black_box)
                .fold(Vec::new(), |mut v, n| {
                    v.push(n);
                    v
                })
        })
    });

    group.bench_function("fold_mut", |b| {
        b.iter(|| {
            (0i64..1_000_000)
                .map(black_box)
                .fold_mut(Vec::new(), |v, n| {
                    v.push(n);
                })
        })
    });

    group.finish();
}

fn bench_num_with_chain(c: &mut Criterion) {
    let mut group = c.benchmark_group("fold chained iterator with num accumulator");

    group.bench_function("fold", |b| {
        b.iter(|| {
            (0i64..1_000_000)
                .chain(0i64..1_000_000)
                .map(black_box)
                .fold(0, |sum, n| sum + n)
        })
    });

    group.bench_function("fold_mut", |b| {
        b.iter(|| {
            (0i64..1_000_000)
                .chain(0i64..1_000_000)
                .map(black_box)
                .fold_mut(0, |sum, n| {
                    *sum += n;
                })
        })
    });

    group.finish();
}

fn bench_vec_with_chain(c: &mut Criterion) {
    let mut group = c.benchmark_group("fold chained iterator with vec accumulator");

    group.bench_function("fold", |b| {
        b.iter(|| {
            (0i64..1_000_000)
                .chain(0i64..1_000_000)
                .map(black_box)
                .fold(Vec::new(), |mut v, n| {
                    v.push(n);
                    v
                })
        })
    });

    group.bench_function("fold_mut", |b| {
        b.iter(|| {
            (0i64..1_000_000)
                .chain(0i64..1_000_000)
                .map(black_box)
                .fold_mut(Vec::new(), |v, n| {
                    v.push(n);
                })
        })
    });

    group.finish();
}

criterion_group!(benches, bench_sum, bench_vec, bench_num_with_chain, bench_vec_with_chain);
criterion_main!(benches);
