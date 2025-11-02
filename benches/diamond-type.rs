use crdt_bench_native::{automerge_parallel, entry, DiamondTypeDoc};
use criterion::criterion_main;

pub fn bench() {
    entry::<DiamondTypeDoc>("diamond-types");
    automerge_parallel::<DiamondTypeDoc>("diamond-types");
}

criterion_main!(bench);
