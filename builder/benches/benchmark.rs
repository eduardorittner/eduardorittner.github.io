use std::path::{Path, PathBuf};

use async_std::task;
use builder::builder::Site;
use criterion::{criterion_group, criterion_main, Criterion};

fn build(from: PathBuf, to: PathBuf) {
    task::block_on(Site::build(to, from, None)).unwrap();
}

pub fn build_benchmark(c: &mut Criterion) {
    let mut n = 1;
    let from = Path::new("/home/erittner/eduardorittner.github.io/src/");
    let to = Path::new("/tmp/nourl");
    c.bench_function("build without url validation", |b| {
        let to = to.join(n.to_string());
        n += 1;
        b.iter(|| build(from.to_owned(), to.to_owned()))
    });
}

fn build_with_url(from: PathBuf, to: PathBuf) {
    task::block_on(Site::build_with_url_validator(to, from)).unwrap();
}

pub fn build_url_benchmark(c: &mut Criterion) {
    let mut n = 1;
    let from = Path::new("/home/erittner/eduardorittner.github.io/src/");
    let to = Path::new("/tmp/url/");
    c.bench_function("build with url validation", |b| {
        let to = to.join(n.to_string());
        n += 1;
        b.iter(|| build_with_url(from.to_owned(), to.to_owned()))
    });
}

criterion_group!(benches, build_benchmark, build_url_benchmark);
criterion_main!(benches);
