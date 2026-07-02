use playbook_lang_core::ir::{IRError, IRGenerator};
use playbook_lang_core::lexer::Lexer;
use playbook_lang_core::parser::{ParseError, Parser};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct LintDiagnostic {
    pub line: usize,
    pub column: usize,
    pub message: String,
    pub severity: String,
}

/// Converts an `IRError` into a diagnostic, mirroring the message text used
/// by `Renderer::format_ir_error` (core/src/renderer/mod.rs) so `lint` and
/// `render`/`play` report the same wording for the same semantic error.
fn ir_error_to_diagnostic(e: &IRError) -> LintDiagnostic {
    match e {
        IRError::UnexpectedPlayer(span, name) => LintDiagnostic {
            line: span.line,
            column: span.column,
            message: format!("Player '{}' not found in state", name),
            severity: "error".to_string(),
        },
        IRError::PlayerNotBaller(span, name) => LintDiagnostic {
            line: span.line,
            column: span.column,
            message: format!("Player '{}' does not have the ball", name),
            severity: "error".to_string(),
        },
    }
}

fn lint_playbook_internal(input: &str) -> Vec<LintDiagnostic> {
    let max_size = std::env::var("MAX_INPUT_SIZE")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(100 * 1024); // 100KB

    if input.len() > max_size {
        return vec![LintDiagnostic {
            line: 0,
            column: 0,
            message: format!("Input exceeds maximum allowed size of {} bytes", max_size),
            severity: "error".to_string(),
        }];
    }

    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);

    let (playbook, errors) = parser.parse();

    if !errors.is_empty() {
        return errors
            .into_iter()
            .map(|e| {
                match e {
                    ParseError::UnexpectedToken(token, msg) => LintDiagnostic {
                        line: token.span.line,
                        column: token.span.column,
                        message: msg,
                        severity: "error".to_string(),
                    },
                    ParseError::InvalidSyntax(token, msg) => LintDiagnostic {
                        line: token.span.line,
                        column: token.span.column,
                        message: msg,
                        severity: "error".to_string(),
                    },
                    ParseError::UnexpectedEOF => LintDiagnostic {
                        line: 0,
                        column: 0, // TODO: improve location for EOF
                        message: "Unexpected End of File".to_string(),
                        severity: "error".to_string(),
                    },
                }
            })
            .collect();
    }

    // Lexing and parsing succeeded with no diagnostics; also run IR
    // generation so semantic errors (e.g. a `move`/`pass`/`defense` entry
    // referencing an unknown player) are caught by `lint`/`lint_playbook`
    // the same way they are by `render`/`play`.
    match IRGenerator::generate(playbook) {
        Ok(_) => Vec::new(),
        Err(e) => vec![ir_error_to_diagnostic(&e)],
    }
}

#[wasm_bindgen]
pub fn lint_playbook(input: &str) -> Result<JsValue, JsValue> {
    let diagnostics = lint_playbook_internal(input);
    let serializer = serde_wasm_bindgen::Serializer::new().serialize_maps_as_objects(true);
    Ok(diagnostics.serialize(&serializer)?)
}

pub fn lint(input: &str) -> Vec<LintDiagnostic> {
    lint_playbook_internal(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lint_valid_playbook() {
        // p1 needs a starting position, otherwise the `move` below is itself
        // a semantic error (an unknown-position player), which is exactly
        // the class of bug this linter pass now catches.
        let input = "players = { p1 } state = { baller = p1, position = { p1 = (0, 0) } } action = { move = { p1 -> (0,0) } }";
        let diagnostics = lint_playbook_internal(input);
        assert!(diagnostics.is_empty(), "diagnostics: {:?}", diagnostics);
    }

    #[test]
    fn test_lint_error_unexpected_token() {
        let input = "players = { p1 } state = { invalid = p1 }";
        let diagnostics = lint_playbook_internal(input);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].severity, "error");
        assert!(diagnostics[0].message.contains("Expected state property"));
        // Check line/column roughly
        assert!(diagnostics[0].line > 0);
    }

    #[test]
    fn test_lint_error_invalid_syntax() {
        // A non-numeric curve factor triggers InvalidSyntax
        let input = "players={p1} state={} action={ move={ p1 ~[l:abc]> (0,0) } }";
        let diagnostics = lint_playbook_internal(input);
        assert!(!diagnostics.is_empty());
        let found = diagnostics
            .iter()
            .any(|d| d.message.contains("Invalid curve factor"));
        assert!(
            found,
            "Expected error message not found in diagnostics: {:?}",
            diagnostics
        );
    }

    #[test]
    fn test_input_too_large() {
        let input = "a".repeat(200 * 1024); // 200KB
        let diagnostics = lint_playbook_internal(&input);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].severity, "error");
        assert!(
            diagnostics[0]
                .message
                .contains("Input exceeds maximum allowed size")
        );
    }

    #[test]
    fn test_lint_move_unknown_player_is_semantic_error() {
        // Lexing and parsing succeed (no syntax diagnostics), but the move
        // references a player that was never defined, which previously only
        // surfaced as an IRError from `render`/`play`, never from `lint`.
        let input = r#"
        players = { p1 }
        state = { position = { p1 = (0, 0) } }
        action = { move = { p2 -> (0, 0) } }
        "#;
        let diagnostics = lint_playbook_internal(input);
        assert_eq!(diagnostics.len(), 1, "diagnostics: {:?}", diagnostics);
        assert_eq!(diagnostics[0].severity, "error");
        assert!(diagnostics[0].message.contains("p2"));
        assert!(diagnostics[0].message.contains("not found"));
    }

    #[test]
    fn test_lint_pass_without_ball_is_semantic_error() {
        let input = r#"
        players = { p1, p2 }
        state = { baller = p2, position = { p1 = (0, 0), p2 = (10, 10) } }
        action = { pass = { p1 -> p2 } }
        "#;
        let diagnostics = lint_playbook_internal(input);
        assert_eq!(diagnostics.len(), 1, "diagnostics: {:?}", diagnostics);
        assert_eq!(diagnostics[0].severity, "error");
        assert!(diagnostics[0].message.contains("p1"));
        assert!(diagnostics[0].message.contains("does not have the ball"));
    }

    #[test]
    fn test_lint_screen_unknown_player_is_semantic_error() {
        let input = r#"
        players = { p1 }
        state = { position = { p1 = (0, 0) } }
        action = { screen = { p1 -> p2 } }
        "#;
        let diagnostics = lint_playbook_internal(input);
        assert_eq!(diagnostics.len(), 1, "diagnostics: {:?}", diagnostics);
        assert_eq!(diagnostics[0].severity, "error");
        assert!(diagnostics[0].message.contains("p2"));
        assert!(diagnostics[0].message.contains("not found"));
    }

    #[test]
    fn test_lint_defense_mark_unknown_player_is_semantic_error() {
        // The case called out in the review: `defense = { d1 -> nosuchplayer }`
        // must produce a lint diagnostic, matching `render`/`play`'s
        // `IRError::UnexpectedPlayer`.
        let input = r#"
        players = { p1 }
        defenders = { d1 }
        state = { position = { p1 = (0, 0) } }
        action = { defense = { d1 -> nosuchplayer } }
        "#;
        let diagnostics = lint_playbook_internal(input);
        assert_eq!(diagnostics.len(), 1, "diagnostics: {:?}", diagnostics);
        assert_eq!(diagnostics[0].severity, "error");
        assert!(diagnostics[0].message.contains("nosuchplayer"));
        assert!(diagnostics[0].message.contains("not found"));
    }

    #[test]
    fn test_lint_state_defense_mark_unknown_player_is_semantic_error() {
        let input = r#"
        players = { p1 }
        defenders = { d1 }
        state = {
            position = { p1 = (0, 0) },
            defense = { d1 -> nosuchplayer },
        }
        "#;
        let diagnostics = lint_playbook_internal(input);
        assert_eq!(diagnostics.len(), 1, "diagnostics: {:?}", diagnostics);
        assert_eq!(diagnostics[0].severity, "error");
        assert!(diagnostics[0].message.contains("nosuchplayer"));
    }

    #[test]
    fn test_lint_valid_playbook_with_defense_lints_clean() {
        // A syntactically and semantically valid file (including the new
        // defense-mode syntax) must still lint clean end-to-end.
        let input = r#"
        players = { p1, p2 }
        defenders = { d1 }
        state = {
            baller = p1,
            position = { p1 = (0, 0), p2 = (10, 10) },
            defense = { d1 -> p2 },
        }
        action = {
            move = { p1 -> (5, 5) },
            pass = { p1 -> p2 },
            defense = { d1 -> p2 },
        }
        "#;
        let diagnostics = lint_playbook_internal(input);
        assert!(diagnostics.is_empty(), "diagnostics: {:?}", diagnostics);
    }
}
