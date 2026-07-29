use crdt_bench_native::{automerge_parallel, entry, YrsDoc};
use criterion::criterion_main;

pub fn yrs() {
    entry::<YrsDoc>("yrs");
    automerge_parallel::<YrsDoc>("yrs");
}

criterion_main!(yrs);
