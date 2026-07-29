use crdt_bench_native::{automerge_parallel, entry, YOctoDoc};
use criterion::criterion_main;

pub fn yrs() {
    entry::<YOctoDoc>("y-octo");
    automerge_parallel::<YOctoDoc>("y-octo");
}

criterion_main!(yrs);
