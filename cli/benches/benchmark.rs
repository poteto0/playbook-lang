use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use playbook_lang_core::Renderer;
use playbook_lang_core::constants::MAX_PHASES;
use playbook_lang_core::ir::{IRGenerator, Scene};
use playbook_lang_core::lexer::Lexer;
use playbook_lang_core::parser::Parser;
use playbook_lang_linter::lint;
use std::fmt::Write;
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

/// A playbook whose phases each move every one of `players` players,
/// giving `players * MAX_PHASES` move interactions in one scene.
fn many_moves_playbook(players: usize) -> String {
    let ids: Vec<String> = (0..players).map(|i| format!("p{i}")).collect();
    let mut src = format!("players = {{ {} }}\n", ids.join(", "));
    src.push_str("state = {\n  baller = p0,\n  position = {\n");
    for (i, id) in ids.iter().enumerate() {
        let _ = writeln!(src, "    {id} = ({}, {}),", i % 100, i / 100);
    }
    src.push_str("  },\n}\nactions = [\n");
    for phase in 0..MAX_PHASES {
        src.push_str("  action = {\n    move = {\n");
        for (i, id) in ids.iter().enumerate() {
            let _ = writeln!(src, "      {id} -> ({}, {}),", i % 100, phase * 10);
        }
        src.push_str("    }\n  },\n");
    }
    src.push_str("]\n");
    src
}

fn build_scene(input: &str) -> Scene {
    let (playbook, errors) = Parser::new(Lexer::new(input).tokenize()).parse();
    assert!(errors.is_empty(), "benchmark input must parse: {errors:?}");
    IRGenerator::generate(playbook).expect("benchmark input must generate IR")
}

pub fn benchmark_render_scene(c: &mut Criterion) {
    let renderer = Renderer::new();
    let mut group = c.benchmark_group("render_scene");

    for players in [100, 500, 1000] {
        let scene = build_scene(&many_moves_playbook(players));
        group.bench_with_input(
            BenchmarkId::new("moves", scene.interactions.len()),
            &scene,
            |b, scene| b.iter(|| black_box(renderer.render_scene(black_box(scene)))),
        );
    }

    group.finish();
}

criterion_group!(benches, benchmark_playbook, benchmark_render_scene);
criterion_main!(benches);
