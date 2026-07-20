use criterion::{black_box, criterion_group, criterion_main, Criterion};
use yaml_rust2::Yaml;

use yabe::diff::{compute_diff, diff_and_common_multiple};
use yabe::merge::merge_yaml;
use yabe::sorter::sort_yaml;

mod common;
use common::{gen_doc, sort_config};

// depth/width pairs: "small" ~0.8k nodes, "large" ~50k nodes per document
const SMALL: (usize, usize) = (2, 8);
const LARGE: (usize, usize) = (4, 8);

fn bench_diff_and_common_multiple(c: &mut Criterion) {
    let mut g = c.benchmark_group("diff_and_common_multiple");
    for (name, (depth, width), files) in [
        ("small_4files", SMALL, 4),
        ("large_4files", LARGE, 4),
        ("large_10files", LARGE, 10),
    ] {
        let docs: Vec<Yaml> = (0..files).map(|i| gen_doc(depth, width, i as i64)).collect();
        let refs: Vec<&Yaml> = docs.iter().collect();
        g.sample_size(20);
        g.bench_function(name, |b| {
            b.iter(|| black_box(diff_and_common_multiple(&refs, 0.51)))
        });
    }
    g.finish();
}

fn bench_compute_diff(c: &mut Criterion) {
    let mut g = c.benchmark_group("compute_diff");
    for (name, (depth, width)) in [("small", SMALL), ("large", LARGE)] {
        let helm = gen_doc(depth, width, 0);
        let obj = gen_doc(depth, width, 1);
        g.sample_size(20);
        g.bench_function(name, |b| b.iter(|| black_box(compute_diff(&obj, &helm))));
    }
    g.finish();
}

fn bench_merge_yaml(c: &mut Criterion) {
    let mut g = c.benchmark_group("merge_yaml");
    for (name, (depth, width)) in [("small", SMALL), ("large", LARGE)] {
        let base = gen_doc(depth, width, 0);
        let override_doc = gen_doc(depth, width, 1);
        g.sample_size(20);
        g.bench_function(name, |b| b.iter(|| black_box(merge_yaml(&base, &override_doc))));
    }
    g.finish();
}

fn bench_sort_yaml(c: &mut Criterion) {
    let mut g = c.benchmark_group("sort_yaml");
    let config = sort_config();
    for (name, (depth, width)) in [("small", SMALL), ("large", LARGE)] {
        let doc = gen_doc(depth, width, 0);
        g.sample_size(20);
        g.bench_function(name, |b| b.iter(|| black_box(sort_yaml(&doc, &config))));
    }
    g.finish();
}

criterion_group!(
    benches,
    bench_diff_and_common_multiple,
    bench_compute_diff,
    bench_merge_yaml,
    bench_sort_yaml
);
criterion_main!(benches);
