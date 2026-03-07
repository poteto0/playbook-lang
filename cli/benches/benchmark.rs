use criterion::{Criterion, criterion_group, criterion_main};
use playbook_lang_core::Renderer;
use playbook_lang_linter::lint;
use std::fs;
use std::hint::black_box;

pub fn benchmark_playbook(c: &mut Criterion) {
    let input =
        fs::read_to_string("../fixtures/input.playbook").expect("Failed to read input.playbook");

    let mut group = c.benchmark_group("playbook");

    group.bench_function("compile", |b| {
        b.iter(|| {
            let renderer = Renderer::new();
            black_box(renderer.render(black_box(&input)))
        })
    });

    group.bench_function("lint", |b| b.iter(|| black_box(lint(black_box(&input)))));

    group.finish();
}

criterion_group!(benches, benchmark_playbook);
criterion_main!(benches);
