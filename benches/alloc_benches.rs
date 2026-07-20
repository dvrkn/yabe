//! Allocation profile of the core operations, measured with a counting
//! global allocator. Run with `cargo bench --bench alloc_benches`.
//!
//! For each operation it reports bytes allocated, allocation count, and peak
//! live bytes, plus the ratio of allocated bytes to the deep size of the
//! inputs. A ratio well above what the operation fundamentally needs (e.g.
//! ~1x input size for an operation that builds one owned copy) indicates
//! redundant deep cloning.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};

use yaml_rust2::Yaml;
use yabe::diff::{compute_diff, diff_and_common_multiple};
use yabe::merge::merge_yaml;
use yabe::sorter::sort_yaml;

mod common;
use common::{count_nodes, gen_doc, sort_config};

struct CountingAlloc;

static BYTES: AtomicUsize = AtomicUsize::new(0);
static CALLS: AtomicUsize = AtomicUsize::new(0);
static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        BYTES.fetch_add(layout.size(), Relaxed);
        CALLS.fetch_add(1, Relaxed);
        let live = LIVE.fetch_add(layout.size(), Relaxed) + layout.size();
        PEAK.fetch_max(live, Relaxed);
        System.alloc(layout)
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        LIVE.fetch_sub(layout.size(), Relaxed);
        System.dealloc(ptr, layout)
    }
}

#[global_allocator]
static ALLOC: CountingAlloc = CountingAlloc;

fn reset() {
    BYTES.store(0, Relaxed);
    CALLS.store(0, Relaxed);
    PEAK.store(LIVE.load(Relaxed), Relaxed);
}

struct Stats {
    bytes: usize,
    calls: usize,
    peak_growth: usize,
}

fn measure<R>(f: impl FnOnce() -> R) -> Stats {
    let live_before = LIVE.load(Relaxed);
    reset();
    let result = f();
    let stats = Stats {
        bytes: BYTES.load(Relaxed),
        calls: CALLS.load(Relaxed),
        peak_growth: PEAK.load(Relaxed).saturating_sub(live_before),
    };
    drop(result);
    stats
}

/// Deep heap size of a document: bytes allocated by one full clone.
fn deep_size(doc: &Yaml) -> usize {
    measure(|| doc.clone()).bytes
}

fn report(op: &str, input_bytes: usize, s: &Stats) {
    println!(
        "{:<44} {:>12} {:>10} {:>12} {:>7.2}x",
        op,
        s.bytes,
        s.calls,
        s.peak_growth,
        s.bytes as f64 / input_bytes.max(1) as f64
    );
}

fn main() {
    let configs = [("small d2 w8", 2usize, 8usize), ("large d4 w8", 4, 8)];

    println!(
        "{:<44} {:>12} {:>10} {:>12} {:>8}",
        "operation", "alloc bytes", "allocs", "peak growth", "x input"
    );

    for (label, depth, width) in configs {
        let docs: Vec<Yaml> = (0..4).map(|i| gen_doc(depth, width, i as i64)).collect();
        let one_doc_size = deep_size(&docs[0]);
        let all_docs_size: usize = docs.iter().map(deep_size).sum();
        println!(
            "-- {} ({} nodes/doc, {} KiB/doc, 4 docs) --",
            label,
            count_nodes(&docs[0]),
            one_doc_size / 1024
        );

        let refs: Vec<&Yaml> = docs.iter().collect();
        let stats = measure(|| diff_and_common_multiple(&refs, 0.51));
        report("diff_and_common_multiple (4 docs, q51)", all_docs_size, &stats);

        let helm = gen_doc(depth, width, 0);
        let stats = measure(|| compute_diff(&docs[1], &helm));
        report("compute_diff (1 doc vs read base)", one_doc_size, &stats);

        let stats = measure(|| merge_yaml(&docs[0], &docs[1]));
        report("merge_yaml (base + override)", one_doc_size, &stats);

        let config = sort_config();
        let stats = measure(|| sort_yaml(&docs[0], &config));
        report("sort_yaml", one_doc_size, &stats);
    }
}
