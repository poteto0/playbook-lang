//! Round-trip property test for the formatter (#85): for any comment
//! placement in a valid playbook, `fmt` must produce output that
//!
//! 1. re-parses without errors, and
//! 2. is formatted identically by a second pass (idempotency).
//!
//! Comments are injected between every pair of adjacent source "chunks"
//! (each chunk is one or more tokens that a comment may not split, such as
//! a numeric literal or a `~[r:2]>` path), which simulates a comment
//! appearing anywhere a user could write one — top level, inside blocks,
//! before commas, or on the same line as preceding code.

use playbook_lang_core::lexer::Lexer;
use playbook_lang_core::parser::Parser;
use playbook_lang_formatter::format_checked;

/// A tiny xorshift64 PRNG so the test is deterministic and dependency-free.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Rng(seed.max(1))
    }
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    fn uint(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
    fn chance(&mut self, one_in: usize) -> bool {
        self.uint(one_in) == 0
    }
}

/// Builds source as a flat chunk list that must stay contiguous.
#[derive(Default)]
struct Builder(Vec<String>);

impl Builder {
    fn push(&mut self, chunk: &str) {
        self.0.push(chunk.to_owned());
    }
}

fn push_coord(b: &mut Builder, rng: &mut Rng) {
    b.push(&((rng.uint(3) as i32 - 1).to_string()));
    b.push(",");
    b.push(&((rng.uint(3) as i32 - 1).to_string()));
}

/// `players = { p1, p2 }` (possibly empty: `players = {}`).
fn identifier_list(b: &mut Builder, keyword: &str, id: char, rng: &mut Rng) {
    if rng.chance(4) {
        return;
    }
    // Empty lists are valid but dropped by `fmt`, so generate them too.
    let count = if rng.chance(8) { 0 } else { 1 + rng.uint(3) };
    b.push(keyword);
    b.push("=");
    b.push("{");
    for i in 0..count {
        if i > 0 {
            b.push(",");
        }
        b.push(&format!("{}{}", id, i + 1));
    }
    b.push("}");
}

fn gen_chunks(rng: &mut Rng) -> Vec<String> {
    let mut b = Builder::default();
    identifier_list(&mut b, "players", 'p', rng);
    identifier_list(&mut b, "defenders", 'd', rng);

    if !rng.chance(4) {
        b.push("state");
        b.push("=");
        b.push("{");
        if rng.chance(2) {
            b.push("baller");
            b.push("=");
            b.push("p1");
            b.push(",");
        }
        if rng.chance(2) {
            b.push("position");
            b.push("=");
            b.push("{");
            for i in 0..1 + rng.uint(2) {
                if i > 0 {
                    b.push(",");
                }
                b.push(&format!("p{}", i + 1));
                b.push("=");
                b.push("(");
                push_coord(&mut b, rng);
                b.push(")");
            }
            b.push("}");
            b.push(",");
        }
        if rng.chance(2) {
            b.push("defense");
            b.push("=");
            b.push("{");
            for i in 0..1 + rng.uint(2) {
                if i > 0 {
                    b.push(",");
                }
                b.push(&format!("d{}", i + 1));
                if rng.chance(2) {
                    b.push("-[3]>");
                    b.push("p1");
                } else {
                    b.push("=");
                    b.push("(");
                    push_coord(&mut b, rng);
                    b.push(")");
                }
            }
            b.push("}");
            b.push(",");
        }
        b.push("}");
    }

    // The parser allows at most 3 action phases playbook-wide (#88).
    let mut actions_left = 3;
    let sections = 1 + rng.uint(3);
    for _ in 0..sections {
        if actions_left == 0 {
            break;
        }
        let list = rng.uint(4) == 0 && actions_left > 1;
        if list {
            b.push("actions");
            b.push("=");
            b.push("[");
        }
        let phases = 1 + rng.uint(2).min(actions_left - 1);
        for i in 0..phases {
            if i > 0 && list {
                b.push(",");
            }
            b.push("action");
            b.push("=");
            b.push("{");
            let items = ["move", "screen", "pass", "defense"];
            let mut included = 0;
            for item in items {
                if rng.uint(3) > 0 {
                    continue;
                }
                if included > 0 {
                    b.push(",");
                }
                included += 1;
                b.push(item);
                b.push("=");
                b.push("{");
                match item {
                    "move" => {
                        b.push("p1");
                        if rng.chance(3) {
                            // Curve factor lives inside the brackets.
                            b.push(&format!("~[r:{}]>", 1 + rng.uint(5)));
                        } else {
                            b.push("->");
                        }
                        b.push("(");
                        push_coord(&mut b, rng);
                        b.push(")");
                    }
                    "screen" => {
                        b.push("p1");
                        b.push("->");
                        if rng.chance(2) {
                            b.push("p2");
                        } else {
                            b.push("(");
                            push_coord(&mut b, rng);
                            b.push(")");
                        }
                    }
                    "pass" => {
                        b.push("p1");
                        b.push("->");
                        b.push("p2");
                    }
                    "defense" => {
                        b.push("d1");
                        b.push("-[1]>");
                        b.push("p1");
                    }
                    _ => unreachable!(),
                }
                b.push("}");
            }
            b.push("}");
        }
        if list {
            b.push("]");
        }
        actions_left -= phases;
    }
    b.0
}

#[test]
fn fmt_output_reparses_and_is_idempotent() {
    for seed in 0..20000u64 {
        let mut rng = Rng::new(seed);
        let chunks = gen_chunks(&mut rng);

        // The generated source must be valid before fmt touches it.
        let src = chunks.join(" ");
        let (_, errors) = Parser::new(Lexer::new(&src).tokenize()).parse();
        assert!(
            errors.is_empty(),
            "seed {}: generator produced invalid source: {:?}\n--- source:\n{}",
            seed,
            errors,
            src
        );

        // Inject 0–4 comments between chunks, plus one at end of file
        // sometimes; join either with spaces or newlines.
        let mut with_comments = chunks.clone();
        let count = rng.uint(4) + if rng.chance(2) { 1 } else { 0 };
        for k in 0..count {
            if with_comments.len() < 2 {
                break;
            }
            let at = rng.uint(with_comments.len() - 1) + 1;
            with_comments.insert(at, format!("// c{}-{}\n", seed, k));
        }
        if rng.chance(3) {
            with_comments.push(format!("// eof{}\n", seed));
        }
        let sep = if rng.uint(2) == 0 { " " } else { "\n" };
        let src2 = with_comments.join(sep);

        for (label, source) in [("plain", src.as_str()), ("commented", src2.as_str())] {
            check_roundtrip(seed, label, source);
        }
    }
}

fn check_roundtrip(seed: u64, label: &str, source: &str) {
    let first = format_checked(source).unwrap_or_else(|errors| {
        panic!(
            "seed {} ({}): fmt failed: {:?}\n--- source:\n{}",
            seed, label, errors, source
        )
    });
    let (_, errors) = Parser::new(Lexer::new(&first).tokenize()).parse();
    assert!(
        errors.is_empty(),
        "seed {} ({}): fmt output does not re-parse: {:?}\n--- source:\n{}\n--- fmt:\n{}",
        seed,
        label,
        errors,
        source,
        first
    );

    match format_checked(&first) {
        Ok(second) if second == first => {}
        Ok(second) => panic!(
            "seed {} ({}): fmt is not idempotent\n--- pass1:\n{}\n--- pass2:\n{}",
            seed, label, first, second
        ),
        Err(errors) => panic!(
            "seed {} ({}): second fmt failed: {:?}\n--- pass1:\n{}",
            seed, label, errors, first
        ),
    }
}

/// All in-repo fixture files that parse cleanly must also round-trip.
#[test]
fn fmt_roundtrip_fixtures() {
    let dir = match std::fs::read_dir("../fixtures") {
        Ok(dir) => dir,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            eprintln!("skipping: fixtures directory not found");
            return;
        }
        Err(err) => panic!("{}", err),
    };
    let mut paths: Vec<_> = dir
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "playbook"))
        .collect();
    paths.sort();
    assert!(
        !paths.is_empty(),
        "no .playbook fixtures found; is the paths heuristic broken?"
    );

    // The CLI strips the `state` block, so `linter_error.playbook` fails.
    for path in paths {
        let source = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("{}: {}", path.display(), err));
        let (_, errors) = Parser::new(Lexer::new(&source).tokenize()).parse();
        if !errors.is_empty() {
            // Inputs with parse errors pass through `fmt` unchanged by
            // design; nothing to round-trip.
            continue;
        }
        check_roundtrip(0, &path.display().to_string(), &source);
    }
}

/// Small deterministic regression cases for comment placements at the
/// boundaries the generator exercises, kept explicit for debuggability.
#[test]
fn fmt_roundtrip_named_cases() {
    let cases = [
        // Top-level comment between two action sections (#85 reproduction).
        "action = { move = { p1 -> (1, 1) } } // c\naction = { move = { p1 -> (2, 2) } }",
        // Start-of-file comment, when `players` / `defenders` are absent and
        // `fmt` anchors to the first section it writes itself.
        "// top\nstate = { }",
        // Comment before an empty state block that fmt writes itself.
        "players = { p1 } // c\n",
        // Comment inside empty identifier lists that fmt drops.
        "players = { // c\n} // c2\ndefenders = {} // c3\n",
        // Comment between a block header and its first entry.
        "state = { position = { // c\np1 = (0, 0) } } action = { move = { p1 -> (0, 0) } }",
        // Trailing comment inside an actions list, before the closing bracket.
        "actions = [ action = { move = { p1 -> (0, 0) } } // c\n ]",
        // Comment at end of file.
        "action = { move = { p1 -> (0, 0) } }\n// eof",
    ];
    for (i, source) in cases.iter().enumerate() {
        // Every case must be valid input first.
        let (_, errors) = Parser::new(Lexer::new(source).tokenize()).parse();
        assert!(
            errors.is_empty(),
            "case {} invalid: {:?}\n{}",
            i,
            errors,
            source
        );
        check_roundtrip(0, &format!("case-{}", i), source);
    }
}
