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

const ASCII_LETTERS: [&str; 26] = [
    "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s",
    "t", "u", "v", "w", "x", "y", "z",
];

struct Lcg32(u32);

impl Lcg32 {
    fn new(seed: u32) -> Self {
        Self(seed)
    }

    fn next(&mut self) -> usize {
        self.0 = self.0.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (self.0 >> 1) as usize
    }
}

pub fn list_random_insert_positions(n: usize) -> Vec<usize> {
    let mut rng = StdRng::seed_from_u64(123);
    (0..n).map(|i| rng.gen::<usize>() % (i + 1)).collect()
}

pub fn parallel_automerge_batches(total_actions: usize) -> Vec<usize> {
    let mut rng = StdRng::seed_from_u64(1);
    let mut batches = Vec::new();
    let mut remaining = total_actions;
    while remaining > 0 {
        for _ in 0..2 {
            let count = rng.gen_range(1..=PARALLEL_EDIT_BATCH);
            batches.push(count);
            remaining = remaining.saturating_sub(count);
        }
    }
    batches
}

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

pub fn run_text_append<C: Crdt>(n: usize) -> Box<str> {
    let mut crdt = C::create(GC, COMPRESSION, Some(1));
    for i in 0..n {
        crdt.text_insert(i, ASCII_LETTERS[i % 26]);
    }
    crdt.get_text()
}

pub fn run_text_random_edits<C: Crdt>(ops: usize) -> Box<str> {
    const WORDS: [&str; 8] = ["a", "the ", "pion ", "crdt ", "x", "yjs ", "fast ", "sync "];
    let mut rng = Lcg32::new(123_456_789);
    let mut crdt = C::create(GC, COMPRESSION, Some(1));
    let mut cursor = 0usize;
    for _ in 0..ops {
        let r = rng.next();
        let cur_len = crdt.text_len();
        let mode = r % 20;
        if mode < 12 || cur_len == 0 {
            if cursor > cur_len {
                cursor = cur_len;
            }
            let w = WORDS[(r >> 4) % 8];
            crdt.text_insert(cursor, w);
            cursor += w.len();
        } else if mode < 17 {
            let pos = (r >> 5) % (cur_len + 1);
            let w = WORDS[(r >> 12) % 8];
            crdt.text_insert(pos, w);
            cursor = pos + w.len();
        } else {
            let pos = (r >> 5) % cur_len;
            let max_d = cur_len - pos;
            let dlen = 1 + ((r >> 12) % 4);
            crdt.text_del(pos, dlen.min(max_d));
            cursor = pos;
        }
    }
    crdt.get_text()
}

pub fn run_text_tombstone<C: Crdt>(n: usize) -> Box<str> {
    let mut rng = Lcg32::new(987_654_321);
    let mut crdt = C::create(GC, COMPRESSION, Some(1));
    for i in 0..n {
        crdt.text_insert(i, ASCII_LETTERS[i % 26]);
    }
    for p in (0..n).rev() {
        if p % 2 == 1 {
            crdt.text_del(p, 1);
        }
    }
    for _ in 0..(n / 5) {
        let pos = rng.next() % (crdt.text_len() + 1);
        crdt.text_insert(pos, "X");
    }
    crdt.get_text()
}

pub fn run_text_backspace<C: Crdt>(n: usize) -> Box<str> {
    let mut seed = String::with_capacity(n);
    for i in 0..n {
        seed.push_str(ASCII_LETTERS[i % 26]);
    }
    let mut crdt = C::create(GC, COMPRESSION, Some(1));
    crdt.text_insert(0, &seed);
    for p in (0..n).rev() {
        crdt.text_del(p, 1);
    }
    crdt.get_text()
}

fn text_workloads<C: Crdt>(c: &mut BenchmarkGroup<WallTime>) {
    c.bench_function("text - append 50k", |b| {
        b.iter(|| black_box(run_text_append::<C>(black_box(50_000))))
    });
    c.bench_function("text - random edits 20k", |b| {
        b.iter(|| black_box(run_text_random_edits::<C>(black_box(20_000))))
    });
    c.bench_function("text - tombstone 50k", |b| {
        b.iter(|| black_box(run_text_tombstone::<C>(black_box(50_000))))
    });
    c.bench_function("text - backspace 50k", |b| {
        b.iter(|| black_box(run_text_backspace::<C>(black_box(50_000))))
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
    text_workloads::<C>(&mut b);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AutomergeDoc, DiamondTypeDoc, Loro, YOctoDoc, YrsDoc};

    #[test]
    fn text_workloads_match_across_all_crdts() {
        let a_app = run_text_append::<YOctoDoc>(2_000);
        assert_eq!(a_app, run_text_append::<YrsDoc>(2_000));
        assert_eq!(a_app, run_text_append::<Loro>(2_000));
        assert_eq!(a_app, run_text_append::<AutomergeDoc>(2_000));
        assert_eq!(a_app, run_text_append::<DiamondTypeDoc>(2_000));

        let a_rand = run_text_random_edits::<YOctoDoc>(2_000);
        assert_eq!(a_rand, run_text_random_edits::<YrsDoc>(2_000));
        assert_eq!(a_rand, run_text_random_edits::<Loro>(2_000));
        assert_eq!(a_rand, run_text_random_edits::<AutomergeDoc>(2_000));
        assert_eq!(a_rand, run_text_random_edits::<DiamondTypeDoc>(2_000));

        let a_tomb = run_text_tombstone::<YOctoDoc>(2_000);
        assert_eq!(a_tomb, run_text_tombstone::<YrsDoc>(2_000));
        assert_eq!(a_tomb, run_text_tombstone::<Loro>(2_000));
        assert_eq!(a_tomb, run_text_tombstone::<AutomergeDoc>(2_000));
        assert_eq!(a_tomb, run_text_tombstone::<DiamondTypeDoc>(2_000));

        let a_bs = run_text_backspace::<YOctoDoc>(2_000);
        assert!(a_bs.is_empty());
        assert_eq!(a_bs, run_text_backspace::<YrsDoc>(2_000));
        assert_eq!(a_bs, run_text_backspace::<Loro>(2_000));
        assert_eq!(a_bs, run_text_backspace::<AutomergeDoc>(2_000));
        assert_eq!(a_bs, run_text_backspace::<DiamondTypeDoc>(2_000));
    }
}
