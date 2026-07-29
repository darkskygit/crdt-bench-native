use crdt_bench_native::{automerge_parallel, entry, AutomergeDoc};
use criterion::criterion_main;

pub fn bench() {
    entry::<AutomergeDoc>("automerge");
    automerge_parallel::<AutomergeDoc>("automerge");
}

criterion_main!(bench);
