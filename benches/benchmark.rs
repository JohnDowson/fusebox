use criterion::{black_box, criterion_group, criterion_main, Criterion, PlottingBackend};
use fusebox::FuseBox;
use pprof::criterion::{Output, PProfProfiler};
use rand::prelude::*;

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

trait Calculation {
    fn calculate(&mut self);
    fn get_result(&self) -> f32;
}

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

calc_struct!(A, *; a, b, c, d, e, f);
calc_struct!(B, *; a, b, c, d, e);
calc_struct!(C, *; a, b, c, d);
calc_struct!(D, *; a, b, c);
calc_struct!(E, *; a, b);
calc_struct!(F, *; a);

fn iteration(c: &mut Criterion) {
    let mut g = c.benchmark_group("Linear access");
    for n in [8, 32, 64, 128, 256, 512].iter() {
        g.bench_with_input(format!("Vec_n{n}"), n, |b, &n| {
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
        g.bench_with_input(format!("FuseBox_n{n}"), n, |b, &n| {
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
    }
    g.finish();
}

fn random_access(c: &mut Criterion) {
    let mut g = c.benchmark_group("Random access");
    for n in [8, 32, 64, 128, 256, 512].iter() {
        g.bench_with_input(format!("Vec_n{n}"), n, |b, &n| {
            let mut r = StdRng::seed_from_u64(69);
            let mut v = prepare_vec(n);

            b.iter(|| {
                let n = r.gen_range(0..n);
                let v = &mut v[n];
                v.calculate();

                black_box(v.get_result());
            })
        });
        g.bench_with_input(format!("FuseBox_n{n}"), n, |b, &n| {
            let mut r = StdRng::seed_from_u64(69);
            let mut f = prepare_fused(n);

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

criterion_group!(name = benches;
    config = config(false);
    targets = iteration, random_access);
criterion_main!(benches);
