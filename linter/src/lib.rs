use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use playbook_lang_core::parser::{Parser, ParseError};
use playbook_lang_core::lexer::{Lexer};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct LintDiagnostic {
    pub line: usize,
    pub column: usize,
    pub message: String,
    pub severity: String,
}

fn lint_playbook_internal(input: &str) -> Vec<LintDiagnostic> {
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    
    match parser.parse() {
        Ok(_) => vec![],
        Err(e) => {
            let diagnostic = match e {
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
            };
            vec![diagnostic]
        }
    }
}

#[wasm_bindgen]
pub fn lint_playbook(input: &str) -> Result<JsValue, JsValue> {
    let diagnostics = lint_playbook_internal(input);
    let serializer = serde_wasm_bindgen::Serializer::new().serialize_maps_as_objects(true);
    Ok(diagnostics.serialize(&serializer)?)
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
        // Curve factor too long triggers InvalidSyntax
        let input = "players={p1} state={} action={ move={ p1 ~[l:0.1234]> (0,0) } }";
        let diagnostics = lint_playbook_internal(input);
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("Curve factor must be at most 3 characters"));
    }
}
