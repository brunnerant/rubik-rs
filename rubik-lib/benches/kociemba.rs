use criterion::{BatchSize, Criterion, criterion_group};
use rubik_lib::{core::scramble::scramble, solve::kociemba};
use workspace_root::get_workspace_root;

criterion_group!(kociemba, bench_first, bench_22);

pub fn bench_first(c: &mut Criterion) {
    let folder = get_workspace_root().join("data/kociemba");
    let mut solver = kociemba::Solver::from_folder(folder).expect("failed to init solver");
    let mut g = c.benchmark_group("kociemba");
    g.bench_function("first", |b| {
        b.iter_batched(
            || scramble(100).1,
            |s| {
                solver.init(&s);
                solver.step();
            },
            BatchSize::SmallInput,
        );
    });
}

pub fn bench_22(c: &mut Criterion) {
    let folder = get_workspace_root().join("data/kociemba");
    let mut solver = kociemba::Solver::from_folder(folder).expect("failed to init solver");
    let mut g = c.benchmark_group("kociemba");
    g.bench_function("under-22", |b| {
        b.iter_batched(
            || scramble(100).1,
            |s| {
                solver.init(&s);
                while let Some(sol) = solver.step()
                    && sol.len() > 22
                {}
            },
            BatchSize::SmallInput,
        );
    });
}
