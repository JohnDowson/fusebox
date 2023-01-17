#[cfg(feature = "bench")]
use bumpalo::Bump;
#[cfg(feature = "bench")]
use criterion::{black_box, criterion_group, criterion_main, Criterion, PlottingBackend};
#[cfg(feature = "bench")]
use fusebox::{inline_meta, FuseBox};
#[cfg(feature = "bench")]
use pprof::criterion::{Output, PProfProfiler};
#[cfg(feature = "bench")]
use rand::prelude::*;

#[cfg(feature = "bench")]
macro_rules! calc_struct {
    ($n:ident, $op:tt; $($f:ident),*) => {
        #[derive(Clone, Copy)]
        struct $n {
            result: f32,
            $($f: f32,)*
        }

        impl $n {
            fn new(r: &mut StdRng) -> Self {
                Self {
                    result: r.gen(),
                    $($f: 0.0,)*
                }
            }
            fn boxed(r: &mut StdRng) -> Box<dyn Calculation> {
                let this = Self::new(r);
                Box::new(this)
            }
            fn bump<'b>(r: &mut StdRng, b: &'b Bump) -> &'b mut dyn Calculation {
                let this = Self::new(r);
                b.alloc(this)
            }
        }

        impl Calculation for $n {
            fn calculate(&mut self) {
                self.result = $(self.$f $op)* 1.0;
            }
            fn get_result(&self) -> f32 {
                self.result
            }
        }
    };
}

#[cfg(feature = "bench")]
trait Calculation {
    fn calculate(&mut self);
    fn get_result(&self) -> f32;
}

#[cfg(feature = "bench")]
fn prepare_vec_bumpalo<'b>(n: usize, b: &'b Bump) -> Vec<&'b mut dyn Calculation> {
    let mut r = StdRng::seed_from_u64(69);
    (0..n)
        .map(|_| {
            let u = r.gen_range(0..=5);
            match u {
                0 => A::bump(&mut r, b),
                1 => B::bump(&mut r, b),
                2 => C::bump(&mut r, b),
                3 => D::bump(&mut r, b),
                4 => E::bump(&mut r, b),
                5 => F::bump(&mut r, b),
                _ => unreachable!(),
            }
        })
        .collect()
}

#[cfg(feature = "bench")]
fn prepare_vec(n: usize) -> Vec<Box<dyn Calculation>> {
    let mut r = StdRng::seed_from_u64(69);
    (0..n)
        .map(|_| {
            let u = r.gen_range(0..=5);
            match u {
                0 => A::boxed(&mut r),
                1 => B::boxed(&mut r),
                2 => C::boxed(&mut r),
                3 => D::boxed(&mut r),
                4 => E::boxed(&mut r),
                5 => F::boxed(&mut r),
                _ => unreachable!(),
            }
        })
        .collect()
}

#[cfg(feature = "bench")]
fn prepare_fused(n: usize) -> FuseBox<dyn Calculation> {
    let mut fused = FuseBox::default();
    let mut r = StdRng::seed_from_u64(69);
    for _ in 0..n {
        let u = r.gen_range(0..=5);
        match u {
            0 => fused.push(A::new(&mut r)),
            1 => fused.push(B::new(&mut r)),
            2 => fused.push(C::new(&mut r)),
            3 => fused.push(D::new(&mut r)),
            4 => fused.push(E::new(&mut r)),
            5 => fused.push(F::new(&mut r)),
            _ => unreachable!(),
        }
    }
    fused
}

#[cfg(feature = "bench")]
fn prepare_inline_meta_fused(n: usize) -> inline_meta::FuseBox<dyn Calculation> {
    let mut fused = inline_meta::FuseBox::default();
    let mut r = StdRng::seed_from_u64(69);
    for _ in 0..n {
        let u = r.gen_range(0..=5);
        match u {
            0 => fused.push(A::new(&mut r)),
            1 => fused.push(B::new(&mut r)),
            2 => fused.push(C::new(&mut r)),
            3 => fused.push(D::new(&mut r)),
            4 => fused.push(E::new(&mut r)),
            5 => fused.push(F::new(&mut r)),
            _ => unreachable!(),
        }
    }
    fused
}

#[cfg(feature = "bench")]
calc_struct!(A, *; a, b, c, d, e, f);
#[cfg(feature = "bench")]
calc_struct!(B, *; a, b, c, d, e);
#[cfg(feature = "bench")]
calc_struct!(C, *; a, b, c, d);
#[cfg(feature = "bench")]
calc_struct!(D, *; a, b, c);
#[cfg(feature = "bench")]
calc_struct!(E, *; a, b);
#[cfg(feature = "bench")]
calc_struct!(F, *; a);

#[cfg(feature = "bench")]
fn iteration(c: &mut Criterion) {
    let mut g = c.benchmark_group("Linear access");
    for n in (8..=512).step_by(8) {
        g.bench_with_input(format!("Vec_n{n}"), &n, |b, &n| {
            let mut v = prepare_vec(n);

            b.iter(|| {
                for v in v.iter_mut() {
                    v.calculate()
                }
                for v in v.iter() {
                    black_box(v.get_result());
                }
            })
        });
        g.bench_with_input(format!("Vec_bumpalo_n{n}"), &n, |b, &n| {
            let bump = Bump::new();
            let mut v = prepare_vec_bumpalo(n, &bump);

            b.iter(|| {
                for v in v.iter_mut() {
                    v.calculate()
                }
                for v in v.iter() {
                    black_box(v.get_result());
                }
            })
        });
        g.bench_with_input(format!("FuseBox_n{n}"), &n, |b, &n| {
            let mut f = prepare_fused(n);

            b.iter(|| {
                for v in f.iter_mut() {
                    v.calculate()
                }
                for v in f.iter() {
                    black_box(v.get_result());
                }
            })
        });
        g.bench_with_input(format!("inline_meta_FuseBox_n{n}"), &n, |b, &n| {
            let mut f = prepare_inline_meta_fused(n);

            b.iter(|| {
                for v in f.iter_mut() {
                    v.calculate()
                }
                for v in f.iter() {
                    black_box(v.get_result());
                }
            })
        });
    }
    g.finish();
}

#[cfg(feature = "bench")]
fn random_access(c: &mut Criterion) {
    let mut g = c.benchmark_group("Random access");
    for n in (8..=512).step_by(8) {
        g.bench_with_input(format!("Vec_n{n}"), &n, |b, &n| {
            let mut r = StdRng::seed_from_u64(69);
            let mut v = prepare_vec(n);

            b.iter(|| {
                let n = r.gen_range(0..n);
                let v = &mut v[n];
                v.calculate();

                black_box(v.get_result());
            })
        });
        g.bench_with_input(format!("Vec_bumpalo_n{n}"), &n, |b, &n| {
            let mut r = StdRng::seed_from_u64(69);
            let bump = Bump::new();
            let mut v = prepare_vec_bumpalo(n, &bump);

            b.iter(|| {
                let n = r.gen_range(0..n);
                let v = &mut v[n];
                v.calculate();

                black_box(v.get_result());
            })
        });
        g.bench_with_input(format!("FuseBox_n{n}"), &n, |b, &n| {
            let mut r = StdRng::seed_from_u64(69);
            let mut f = prepare_fused(n);

            b.iter(|| {
                let n = r.gen_range(0..n);
                let v = &mut f[n];
                v.calculate();
                v.get_result();
            })
        });
        g.bench_with_input(format!("inline_meta_FuseBox_n{n}"), &n, |b, &n| {
            let mut r = StdRng::seed_from_u64(69);
            let mut f = prepare_inline_meta_fused(n);

            b.iter(|| {
                let n = r.gen_range(0..n);
                let v = &mut f[n];
                v.calculate();
                v.get_result();
            })
        });
    }
    g.finish();
}

#[cfg(feature = "bench")]
fn config(pprof: bool) -> Criterion {
    let c = Criterion::default()
        .sample_size(200)
        .plotting_backend(PlottingBackend::Gnuplot)
        .with_plots();
    if pprof {
        c.with_profiler(PProfProfiler::new(500, Output::Flamegraph(None)))
    } else {
        c
    }
}

#[cfg(feature = "bench")]
criterion_group!(name = benches;
    config = config(false);
    targets = iteration, random_access);
#[cfg(feature = "bench")]
criterion_main!(benches);

#[cfg(not(feature = "bench"))]
fn main() {}
