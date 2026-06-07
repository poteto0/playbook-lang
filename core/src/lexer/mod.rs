#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

impl Span {
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenKind {
    // Keywords
    Players,
    State,
    Baller,
    Position,
    Action,
    Actions,
    Move,
    Screen,
    Pass,
    Before,
    After,
    Middle,

    // Identifiers & Values
    Identifier(String),
    Number(f64), // Coordinates can be numbers

    // Symbols
    Equals,             // =
    LBrace,             // {
    RBrace,             // }
    LBracket,           // [
    RBracket,           // ]
    LParenthesis,       // (
    RParenthesis,       // )
    Comma,              // ,
    Arrow,              // ->
    CurveArrow(String), // ~> (default: l), ~[l]>, ~[r]>
    Colon,              // :

    // Special
    Comment(String),
    EOF,
    Error(String),
}

impl std::fmt::Display for TokenKind {
    // call on to_string()
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TokenKind::Players => write!(f, "players"),
            TokenKind::State => write!(f, "state"),
            TokenKind::Baller => write!(f, "baller"),
            TokenKind::Position => write!(f, "position"),
            TokenKind::Action => write!(f, "action"),
            TokenKind::Actions => write!(f, "actions"),
            TokenKind::Move => write!(f, "move"),
            TokenKind::Screen => write!(f, "screen"),
            TokenKind::Pass => write!(f, "pass"),
            TokenKind::Before => write!(f, "before"),
            TokenKind::After => write!(f, "after"),
            TokenKind::Middle => write!(f, "middle"),
            TokenKind::Identifier(s) => write!(f, "{}", s),
            TokenKind::Number(n) => write!(f, "{}", n),
            TokenKind::Equals => write!(f, "="),
            TokenKind::LBrace => write!(f, "{{"),
            TokenKind::RBrace => write!(f, "}}"),
            TokenKind::LBracket => write!(f, "["),
            TokenKind::RBracket => write!(f, "]"),
            TokenKind::LParenthesis => write!(f, "("),
            TokenKind::RParenthesis => write!(f, ")"),
            TokenKind::Comma => write!(f, ","),
            TokenKind::Arrow => write!(f, "->"),
            TokenKind::CurveArrow(_) => write!(f, "~>"),
            TokenKind::Colon => write!(f, ":"),
            TokenKind::Comment(s) => write!(f, "// {}", s),
            TokenKind::EOF => write!(f, "EOF"),
            TokenKind::Error(s) => write!(f, "Error: {}", s),
        }
    }
}

pub struct Lexer<'a> {
    input: &'a str,
    pos: usize,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            pos: 0,
            line: 1,
            column: 1,
        }
    }

    fn peek(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += c.len_utf8();
        if c == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        Some(c)
    }

    fn starts_with(&self, s: &str) -> bool {
        self.input[self.pos..].starts_with(s)
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn read_identifier(&mut self) -> String {
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                self.advance();
            } else {
                break;
            }
        }
        self.input[start..self.pos].to_string()
    }

    fn read_number(&mut self) -> TokenKind {
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() || c == '.' || c == '-' {
                self.advance();
            } else {
                break;
            }
        }
        let raw = &self.input[start..self.pos];
        match raw.parse() {
            Ok(n) => TokenKind::Number(n),
            Err(_) => TokenKind::Error(format!("Invalid number: '{}'", raw)),
        }
    }

    fn read_comment(&mut self) -> String {
        // Skip //
        self.advance();
        self.advance();

        let start = self.pos;
        while let Some(c) = self.peek() {
            if c == '\n' {
                break;
            }
            self.advance();
        }
        self.input[start..self.pos].trim().to_string()
    }

    fn read_inside_brackets(&mut self) -> Result<String, String> {
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c == ']' {
                return Ok(self.input[start..self.pos].to_string());
            }
            if c == '\n' {
                return Err("Unexpected newline in curve parameters".to_string());
            }
            self.advance();
        }
        Err("Unclosed bracket".to_string())
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        let start_pos = self.pos;
        let start_line = self.line;
        let start_column = self.column;

        let c = match self.peek() {
            Some(c) => c,
            None => {
                return Token {
                    kind: TokenKind::EOF,
                    span: Span {
                        start: start_pos,
                        end: start_pos,
                        line: start_line,
                        column: start_column,
                    },
                };
            }
        };

        let kind = match c {
            '=' => {
                self.advance();
                TokenKind::Equals
            }
            '{' => {
                self.advance();
                TokenKind::LBrace
            }
            '}' => {
                self.advance();
                TokenKind::RBrace
            }
            '[' => {
                self.advance();
                TokenKind::LBracket
            }
            ']' => {
                self.advance();
                TokenKind::RBracket
            }
            '(' => {
                self.advance();
                TokenKind::LParenthesis
            }
            ')' => {
                self.advance();
                TokenKind::RParenthesis
            }
            ',' => {
                self.advance();
                TokenKind::Comma
            }
            ':' => {
                self.advance();
                TokenKind::Colon
            }
            '-' => {
                if self.starts_with("->") {
                    self.advance();
                    self.advance();
                    TokenKind::Arrow
                } else if self.peek().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                    self.read_number()
                } else {
                    let next_char_is_digit = self.input[self.pos + 1..]
                        .chars()
                        .next()
                        .map(|c| c.is_ascii_digit())
                        .unwrap_or(false);
                    if next_char_is_digit {
                        self.read_number()
                    } else {
                        self.advance();
                        TokenKind::Identifier("-".to_string())
                    }
                }
            }
            '~' => {
                if self.starts_with("~>") {
                    self.advance();
                    self.advance();
                    TokenKind::CurveArrow("default".to_string())
                } else if self.starts_with("~[") {
                    self.advance(); // ~
                    self.advance(); // [
                    match self.read_inside_brackets() {
                        Ok(content) => {
                            if self.starts_with("]>") {
                                self.advance(); // ]
                                self.advance(); // >
                                TokenKind::CurveArrow(content)
                            } else {
                                TokenKind::Error(format!(
                                    "Expected ]> after curve parameters, but found '{}",
                                    content
                                ))
                            }
                        }
                        Err(msg) => TokenKind::Error(msg),
                    }
                } else {
                    self.advance();
                    TokenKind::Identifier("~".to_string())
                }
            }
            '/' => {
                if self.starts_with("//") {
                    TokenKind::Comment(self.read_comment())
                } else {
                    self.advance();
                    TokenKind::Identifier("/".to_string())
                }
            }
            _ if c.is_ascii_digit() => self.read_number(),
            _ if c.is_alphabetic() => {
                let ident = self.read_identifier();
                match ident.as_str() {
                    "players" => TokenKind::Players,
                    "state" => TokenKind::State,
                    "baller" => TokenKind::Baller,
                    "position" => TokenKind::Position,
                    "action" => TokenKind::Action,
                    "actions" => TokenKind::Actions,
                    "move" => TokenKind::Move,
                    "screen" => TokenKind::Screen,
                    "pass" => TokenKind::Pass,
                    "before" => TokenKind::Before,
                    "after" => TokenKind::After,
                    "middle" => TokenKind::Middle,
                    _ => TokenKind::Identifier(ident),
                }
            }
            _ => {
                self.advance();
                TokenKind::Identifier(c.to_string())
            }
        };

        Token {
            kind,
            span: Span {
                start: start_pos,
                end: self.pos,
                line: start_line,
                column: start_column,
            },
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let max_size = std::env::var("MAX_INPUT_SIZE")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(100 * 1024); // 100KB

        if self.input.len() > max_size {
            return vec![Token {
                kind: TokenKind::Error(format!(
                    "Input too large: {} bytes (max: {} bytes)",
                    self.input.len(),
                    max_size
                )),
                span: Span {
                    start: 0,
                    end: 0,
                    line: 0,
                    column: 0,
                },
            }];
        }

        let mut tokens = Vec::new();
        loop {
            let token = self.next_token();
            if let TokenKind::EOF = token.kind {
                tokens.push(token);
                break;
            }
            tokens.push(token);
        }
        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keywords_and_symbols() {
        let input = "players = { } -> :";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let kinds: Vec<TokenKind> = tokens.into_iter().map(|t| t.kind).collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::Players,
                TokenKind::Equals,
                TokenKind::LBrace,
                TokenKind::RBrace,
                TokenKind::Arrow,
                TokenKind::Colon,
                TokenKind::EOF
            ]
        );
    }

    #[test]
    fn test_identifiers_and_numbers() {
        let input = "p1 (10, -20.5)";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let kinds: Vec<TokenKind> = tokens.into_iter().map(|t| t.kind).collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::Identifier("p1".to_string()),
                TokenKind::LParenthesis,
                TokenKind::Number(10.0),
                TokenKind::Comma,
                TokenKind::Number(-20.5),
                TokenKind::RParenthesis,
                TokenKind::EOF
            ]
        );
    }

    #[test]
    fn test_comments() {
        let input = "players // this is a comment\nstate";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let kinds: Vec<TokenKind> = tokens.into_iter().map(|t| t.kind).collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::Players,
                TokenKind::Comment("this is a comment".to_string()),
                TokenKind::State,
                TokenKind::EOF
            ]
        );
    }

    #[test]
    fn test_span() {
        let input = "players";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        assert_eq!(tokens[0].span.line, 1);
        assert_eq!(tokens[0].span.column, 1);

        let input = "\n  players";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        assert_eq!(tokens[0].span.line, 2);
        assert_eq!(tokens[0].span.column, 3);
    }

    #[test]
    fn test_input_too_large() {
        let input = "a".repeat(100 * 1024 + 1);
        let mut lexer = Lexer::new(&input);
        let tokens = lexer.tokenize();
        assert_eq!(tokens.len(), 1);
        match &tokens[0].kind {
            TokenKind::Error(msg) => assert!(msg.contains("Input too large")),
            _ => panic!("Expected Error token"),
        }
    }
}
