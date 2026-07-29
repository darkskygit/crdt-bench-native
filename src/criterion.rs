use std::time::Duration;

use criterion::{black_box, measurement::WallTime, BenchmarkGroup, Criterion};
use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::{
    automerge::{self, get_automerge_actions, TextAction},
    merge, Crdt,
};

const GC: bool = false;
const COMPRESSION: bool = false;
const PARALLEL_EDIT_BATCH: usize = 1_000;

pub fn bench_random_list_insert<C: Crdt>(n: usize) {
    let mut rng = rand::rngs::StdRng::seed_from_u64(123);
    let mut crdt = C::create(GC, COMPRESSION, None);
    let mut crdt_new = C::create(GC, COMPRESSION, None);
    for i in 0..n {
        crdt.list_insert(rng.gen::<usize>() % (i + 1), i as i32);
        merge(&mut crdt_new, &mut crdt);
    }
}

fn list_random_insert_1k<C: Crdt>(c: &mut BenchmarkGroup<WallTime>) {
    c.bench_function("list_random_insert_1k", |b| {
        b.iter(|| bench_random_list_insert::<C>(black_box(1_000)))
    });
}

fn concurrent_list_inserts<C: Crdt>(b: &mut BenchmarkGroup<WallTime>) {
    b.bench_function("concurrent list inserts", |b| {
        b.iter(|| {
            let mut docs = vec![];
            for _ in 0..100 {
                docs.push(C::create(GC, COMPRESSION, None));
            }

            for doc in docs.iter_mut() {
                doc.text_insert(0, "123");
            }

            for i in 1..100 {
                let (a, b) = arref::array_mut_ref!(&mut docs, [0, i]);
                merge(a, b);
            }

            for i in 1..100 {
                let (a, b) = arref::array_mut_ref!(&mut docs, [0, i]);
                merge(b, a);
            }
        });
    });
}

fn apply_automerge_paper<C: Crdt>(b: &mut BenchmarkGroup<WallTime>) {
    let actions = automerge::get_automerge_actions();
    b.bench_function("automerge - apply", |b| {
        b.iter(|| {
            let mut crdt = C::create(GC, COMPRESSION, None);
            for action in &actions {
                if action.del != 0 {
                    crdt.text_del(action.pos, action.del);
                }

                if !action.ins.is_empty() {
                    crdt.text_insert(action.pos, &action.ins);
                }
            }
        })
    });
    let mut crdt = C::create(GC, COMPRESSION, None);
    for action in &actions {
        if action.del != 0 {
            crdt.text_del(action.pos, action.del);
        }

        if !action.ins.is_empty() {
            crdt.text_insert(action.pos, &action.ins);
        }
    }
    b.bench_function("automerge - encode time", |b| {
        b.iter(|| {
            black_box(crdt.encode_full());
        })
    });
    let encoded = crdt.encode_full();
    b.bench_function("automerge - decode time", |b| {
        b.iter(|| {
            let mut new_crdt = C::create(GC, COMPRESSION, None);
            new_crdt.decode_full(black_box(&encoded));
        })
    });
}

pub fn entry<C: Crdt>(name: &str) {
    let mut criterion: Criterion<_> = Criterion::default()
        .configure_from_args()
        .measurement_time(Duration::from_secs(1))
        .sample_size(10);
    let mut b = criterion.benchmark_group(name);
    list_random_insert_1k::<C>(&mut b);
    concurrent_list_inserts::<C>(&mut b);
    apply_automerge_paper::<C>(&mut b);
}

pub fn automerge_parallel<C: Crdt>(name: &str) {
    let mut criterion: Criterion<_> = Criterion::default()
        .configure_from_args()
        .measurement_time(Duration::from_secs(1))
        .sample_size(10);
    let mut b = criterion.benchmark_group(name);
    b.bench_function("parallel applying automerge edits", |b| {
        b.iter(|| {
            let mut crdt = C::create(false, false, None);
            let mut crdt2 = C::create(false, false, None);
            let mut rng: StdRng = SeedableRng::seed_from_u64(1);
            let mut actions = get_automerge_actions().into_iter();
            loop {
                let mut applied = false;
                for crdt in [&mut crdt, &mut crdt2] {
                    let count = rng.gen_range(1..=PARALLEL_EDIT_BATCH);
                    for _ in 0..count {
                        let Some(action) = actions.next() else {
                            break;
                        };
                        apply_text_action(crdt, action);
                        applied = true;
                    }
                }

                if !applied {
                    break;
                }
                merge(&mut crdt, &mut crdt2);
                assert_eq!(crdt.text_len(), crdt2.text_len());
            }

            assert_eq!(crdt.get_text(), crdt2.get_text());
        })
    });
}

fn apply_text_action<C: Crdt>(crdt: &mut C, mut action: TextAction) {
    let len = crdt.text_len();
    action.pos %= len + 1;
    action.del = action.del.min(len - action.pos);

    if action.del != 0 {
        crdt.text_del(action.pos, action.del);
    }
    if !action.ins.is_empty() {
        crdt.text_insert(action.pos, &action.ins);
    }
}
