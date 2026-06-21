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

    let (_, errors) = parser.parse();

    errors
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
        .collect()
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
        let input = "players = { p1 } state = { baller = p1 } action = { move = { p1 -> (0,0) } }";
        let diagnostics = lint_playbook_internal(input);
        assert!(diagnostics.is_empty());
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
}
