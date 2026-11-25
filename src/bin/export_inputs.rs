use std::fs;
use std::io::Write;
use std::path::Path;

use crdt_bench_native::{
    get_automerge_actions, list_random_insert_positions, parallel_automerge_batches,
    run_text_append, run_text_backspace, run_text_random_edits, run_text_tombstone, Crdt,
    YOctoDoc,
};

fn fnv64(s: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in s.as_bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h
}

fn hex_encode(s: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(s.len() * 2);
    for &b in s.as_bytes() {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0xf) as usize] as char);
    }
    out
}

fn main() {
    let out_dir = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "target/pion-inputs".to_string());
    let out = Path::new(&out_dir);
    fs::create_dir_all(out).unwrap();

    // 1. automerge-paper.ops: "pos del hex" per line
    let actions = get_automerge_actions();
    {
        let mut f = std::io::BufWriter::new(fs::File::create(out.join("automerge-paper.ops")).unwrap());
        for a in &actions {
            writeln!(f, "{} {} {}", a.pos, a.del, hex_encode(&a.ins)).unwrap();
        }
    }

    // 2. list_random_insert_1k.pos: exact StdRng::seed_from_u64(123) positions
    let positions = list_random_insert_positions(1_000);
    {
        let mut f = std::io::BufWriter::new(
            fs::File::create(out.join("list_random_insert_1k.pos")).unwrap(),
        );
        for p in positions {
            writeln!(f, "{p}").unwrap();
        }
    }

    // 3. parallel_batches.txt: exact StdRng::seed_from_u64(1) batch sizes
    let batches = parallel_automerge_batches(actions.len());
    {
        let mut f =
            std::io::BufWriter::new(fs::File::create(out.join("parallel_batches.txt")).unwrap());
        for b in batches {
            writeln!(f, "{b}").unwrap();
        }
    }

    // 4. Optional reference hashes for --verify
    if std::env::args().any(|a| a == "--hashes") {
        let mut doc = YOctoDoc::create(false, false, Some(1));
        for a in &actions {
            if a.del != 0 {
                doc.text_del(a.pos, a.del);
            }
            if !a.ins.is_empty() {
                doc.text_insert(a.pos, &a.ins);
            }
        }
        let auto_txt = doc.get_text();
        let app_txt = run_text_append::<YOctoDoc>(50_000);
        let rand_txt = run_text_random_edits::<YOctoDoc>(20_000);
        let tomb_txt = run_text_tombstone::<YOctoDoc>(50_000);
        let bs_txt = run_text_backspace::<YOctoDoc>(50_000);
        println!("REF\tautomerge - apply\t{}\t{}", auto_txt.len(), fnv64(&auto_txt));
        println!("REF\ttext - append 50k\t{}\t{}", app_txt.len(), fnv64(&app_txt));
        println!("REF\ttext - random edits 20k\t{}\t{}", rand_txt.len(), fnv64(&rand_txt));
        println!("REF\ttext - tombstone 50k\t{}\t{}", tomb_txt.len(), fnv64(&tomb_txt));
        println!("REF\ttext - backspace 50k\t{}\t{}", bs_txt.len(), fnv64(&bs_txt));
    }
}
