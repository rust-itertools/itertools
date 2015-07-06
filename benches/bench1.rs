#![feature(test)]

extern crate test;
extern crate itertools;

use test::{black_box};
use itertools::Stride;
use itertools::Itertools;

#[cfg(not(feature = "unstable"))]
use itertools::Zip;

#[cfg(feature = "unstable")]
use itertools::{Zip, ZipTrusted};

use std::iter::repeat;
use std::cmp;

#[bench]
fn slice_iter(b: &mut test::Bencher)
{
    let xs: Vec<_> = repeat(1i32).take(20).collect();
    b.iter(|| for elt in xs.iter() {
        test::black_box(elt);
    })
}

#[bench]
fn slice_iter_rev(b: &mut test::Bencher)
{
    let xs: Vec<_> = repeat(1i32).take(20).collect();
    b.iter(|| for elt in xs.iter().rev() {
        test::black_box(elt);
    })
}

#[bench]
fn stride_iter(b: &mut test::Bencher)
{
    let xs: Vec<_> = repeat(1i32).take(20).collect();
    b.iter(|| for elt in Stride::from_slice(&xs, 1) {
        test::black_box(elt);
    })
}

#[bench]
fn stride_iter_rev(b: &mut test::Bencher)
{
    let xs: Vec<_> = repeat(1i32).take(20).collect();
    b.iter(|| for elt in Stride::from_slice(&xs, 1).rev() {
        test::black_box(elt);
    })
}

#[derive(Copy, Clone)]
struct ZipSlices<'a, T: 'a, U :'a>
{
    t_ptr: *const T,
    t_end: *const T,
    u_ptr: *const U,
    mark: PhantomData<&'a (T, U)>,
}

impl<'a, T, U> ZipSlices<'a, T, U>
{
    pub fn new(t: &'a [T], u: &'a [U]) -> Self
    {
        assert!(mem::size_of::<T>() != 0);
        assert!(mem::size_of::<U>() != 0);
        let minl = cmp::min(t.len(), u.len());
        let tptr = t.as_ptr();
        let uptr = u.as_ptr();
        let end_ptr = unsafe {
            tptr.offset(minl as isize)
        };
        ZipSlices {
            t_ptr: tptr,
            t_end: end_ptr,
            u_ptr: uptr,
            mark: PhantomData,
        }
    }
}

impl<'a, T, U> Iterator for ZipSlices<'a, T, U>
{
    type Item = (&'a T, &'a U);

    #[inline]
    fn next(&mut self) -> Option<(&'a T, &'a U)>
    {
        if self.t_ptr == self.t_end {
            return None
        }
        let t_elt: &T;
        let u_elt: &U;
        unsafe {
            t_elt = &*self.t_ptr;
            self.t_ptr = self.t_ptr.offset(1);
            u_elt = &*self.u_ptr;
            self.u_ptr = self.u_ptr.offset(1);
        }
        Some((t_elt, u_elt))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>)
    {
        let len = self.t_end as usize - self.t_ptr as usize;
        (len, Some(len))
    }
}

#[bench]
fn zip_default_zip(b: &mut test::Bencher)
{
    let xs = vec![0; 1024];
    let ys = vec![0; 768];
    let xs = black_box(xs);
    let ys = black_box(ys);

    b.iter(|| {
        for (&x, &y) in xs.iter().zip(&ys) {
            test::black_box(x);
            test::black_box(y);
        }
    })
}

#[bench]
fn zip_default_zip3(b: &mut test::Bencher)
{
    let xs = vec![0; 1024];
    let ys = vec![0; 768];
    let zs = vec![0; 766];
    let xs = black_box(xs);
    let ys = black_box(ys);
    let zs = black_box(zs);

    b.iter(|| {
        for ((&x, &y), &z) in xs.iter().zip(&ys).zip(&zs) {
            test::black_box(x);
            test::black_box(y);
            test::black_box(z);
        }
    })
}

/*
#[bench]
fn zip_slices_ziptuple(b: &mut test::Bencher)
{
    let xs = vec![0; 1024];
    let ys = vec![0; 768];

    b.iter(|| {
        let xs = black_box(&xs);
        let ys = black_box(&ys);
        for (&x, &y) in Zip::new((xs, ys)) {
            test::black_box(x);
            test::black_box(y);
        }
    })
}
*/

#[bench]
fn zip_slices(b: &mut test::Bencher)
{
    let xs = vec![0; 1024];
    let ys = vec![0; 768];
    let xs = black_box(xs);
    let ys = black_box(ys);

    b.iter(|| {
        for (&x, &y) in ZipSlices::new(&xs, &ys) {
            test::black_box(x);
            test::black_box(y);
        }
    })
}

#[cfg(feature = "unstable")]
#[bench]
fn ziptrusted(b: &mut test::Bencher)
{
    let xs = vec![0; 1024];
    let ys = vec![0; 768];
    let xs = black_box(xs);
    let ys = black_box(ys);

    b.iter(|| {
        for (&x, &y) in ZipTrusted::new((xs.iter(), ys.iter())) {
            test::black_box(x);
            test::black_box(y);
        }
    })
}

#[cfg(feature = "unstable")]
#[bench]
fn ziptrusted3(b: &mut test::Bencher)
{
    let xs = vec![0; 1024];
    let ys = vec![0; 768];
    let zs = vec![0; 766];
    let xs = black_box(xs);
    let ys = black_box(ys);
    let zs = black_box(zs);

    b.iter(|| {
        for (&x, &y, &z) in ZipTrusted::new((xs.iter(), ys.iter(), zs.iter())) {
            test::black_box(x);
            test::black_box(y);
            test::black_box(z);
        }
    })
}

#[bench]
fn zip_checked_counted_loop(b: &mut test::Bencher)
{
    let xs = vec![0; 1024];
    let ys = vec![0; 768];
    let xs = black_box(xs);
    let ys = black_box(ys);

    b.iter(|| {
        let xs = &xs[..];
        let ys = &ys[..];
        let len = cmp::min(xs.len(), ys.len());

        for i in 0..len {
            let x = xs[i];
            let y = ys[i];
            test::black_box(x);
            test::black_box(y);
        }
    })
}

#[bench]
fn zip_unchecked_counted_loop(b: &mut test::Bencher)
{
    let xs = vec![0; 1024];
    let ys = vec![0; 768];
    let xs = black_box(xs);
    let ys = black_box(ys);

    b.iter(|| {
        let len = cmp::min(xs.len(), ys.len());
        for i in 0..len {
            unsafe {
            let x = *xs.get_unchecked(i);
            let y = *ys.get_unchecked(i);
            test::black_box(x);
            test::black_box(y);
            }
        }
    })
}

#[bench]
fn zip_unchecked_counted_loop3(b: &mut test::Bencher)
{
    let xs = vec![0; 1024];
    let ys = vec![0; 768];
    let zs = vec![0; 766];
    let xs = black_box(xs);
    let ys = black_box(ys);
    let zs = black_box(zs);

    b.iter(|| {
        let len = cmp::min(xs.len(), cmp::min(ys.len(), zs.len()));
        for i in 0..len {
            unsafe {
            let x = *xs.get_unchecked(i);
            let y = *ys.get_unchecked(i);
            let z = *zs.get_unchecked(i);
            test::black_box(x);
            test::black_box(y);
            test::black_box(z);
            }
        }
    })
}

#[bench]
fn group_by_lazy_1(b: &mut test::Bencher) {
    let mut data = vec![0; 1024];
    for (index, elt) in data.iter_mut().enumerate() {
        *elt = index / 10;
    }

    b.iter(|| {
        let iter = test::black_box(data.iter());
        for (_key, group) in &iter.group_by_lazy(|elt| **elt) {
            for elt in group {
                test::black_box(elt);
            }
        }
    })
}

#[bench]
fn group_by_lazy_2(b: &mut test::Bencher) {
    let mut data = vec![0; 1024];
    for (index, elt) in data.iter_mut().enumerate() {
        *elt = index / 2;
    }

    b.iter(|| {
        let iter = test::black_box(data.iter());
        for (_key, group) in &iter.group_by_lazy(|elt| **elt) {
            for elt in group {
                test::black_box(elt);
            }
        }
    })
}

#[bench]
fn equal(b: &mut test::Bencher) {
    let data = vec![7; 1024];
    let l = data.len();
    b.iter(|| {
        let a = test::black_box(&data[1..]);
        let b = test::black_box(&data[..l - 1]);
        itertools::equal(a, b)
    })
}
