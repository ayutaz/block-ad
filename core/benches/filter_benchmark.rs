use adblock_core::{AdBlockCore, Config};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_filter_engine(c: &mut Criterion) {
    let mut group = c.benchmark_group("filter_engine");

    // Create engine with sample rules
    let config = Config::default();
    let mut core = AdBlockCore::new(config);

    let filter_rules = r#"
||doubleclick.net^
||googleadservices.com^
||googlesyndication.com^
||google-analytics.com^
||googletagmanager.com^
||facebook.com/tr^
||amazon-adsystem.com^
"#;

    core.load_filter_list(filter_rules);

    group.bench_function("should_block_ad_url", |b| {
        b.iter(|| core.should_block(black_box("https://doubleclick.net/ads/banner.js")))
    });

    group.bench_function("should_block_normal_url", |b| {
        b.iter(|| core.should_block(black_box("https://example.com/index.html")))
    });

    group.bench_function("should_block_mixed_urls", |b| {
        let urls = vec![
            "https://doubleclick.net/ads/1",
            "https://example.com/page",
            "https://googleadservices.com/pagead/js",
            "https://github.com/user/repo",
            "https://googlesyndication.com/ad",
        ];

        b.iter(|| {
            for url in &urls {
                black_box(core.should_block(url));
            }
        })
    });

    group.finish();
}

fn benchmark_filter_loading(c: &mut Criterion) {
    let mut group = c.benchmark_group("filter_loading");

    let small_filter_list = r#"
||doubleclick.net^
||googleadservices.com^
||googlesyndication.com^
"#;

    let large_filter_list = include_str!("../tests/fixtures/easylist_sample.txt");

    group.bench_function("load_small_filter_list", |b| {
        b.iter(|| {
            let config = Config::default();
            let mut core = AdBlockCore::new(config);
            core.load_filter_list(black_box(small_filter_list));
        })
    });

    group.bench_function("load_large_filter_list", |b| {
        b.iter(|| {
            let config = Config::default();
            let mut core = AdBlockCore::new(config);
            core.load_filter_list(black_box(large_filter_list));
        })
    });

    group.finish();
}

criterion_group!(benches, benchmark_filter_engine, benchmark_filter_loading);
criterion_main!(benches);
