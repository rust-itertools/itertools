#![feature(test)]

extern crate test;
extern crate itertools;

macro_rules! def_benchs {
    ($N:expr,
     $FUN:ident,
     $BENCH_NAME:ident,
     ) => (
        mod $BENCH_NAME {
        use itertools::Itertools;
        use itertools::cloned;
        use test::Bencher;

        #[bench]
        fn sum(b: &mut Bencher) {
            let v: Vec<u32> = (0.. $N).collect();
            b.iter(|| {
                cloned(&v).$FUN(|x, y| x + y)
            });
        }

        #[bench]
        fn string_format(b: &mut Bencher) {
            let v: Vec<u32> = (0.. $N).collect();
            b.iter(|| {
                cloned(&v).map(|x| x.to_string()).$FUN(|x, y| format!("{} + {}", x, y))
            });
        }
        }
    )
}

def_benchs!{
    10_000,
    fold1,
    fold1_10k,
}

def_benchs!{
    10_000,
    tree_fold1,
    tree_fold1_stack_10k,
}

def_benchs!{
    10_000,
    fold1_balanced,
    tree_fold1_vec_10k,
}

def_benchs!{
    100,
    fold1,
    fold1_100,
}

def_benchs!{
    100,
    tree_fold1,
    tree_fold1_stack_100,
}

def_benchs!{
    100,
    fold1_balanced,
    tree_fold1_vec_100,
}

def_benchs!{
    8,
    fold1,
    fold1_08,
}

def_benchs!{
    8,
    tree_fold1,
    tree_fold1_stack_08,
}

def_benchs!{
    8,
    fold1_balanced,
    tree_fold1_vec_08,
}
