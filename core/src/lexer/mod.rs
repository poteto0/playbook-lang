#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    // Keywords
    Players,
    State,
    Baller,
    Position,
    Action,
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
    Equals,       // =
    LBrace,       // {
    RBrace,       // }
    LParenthesis, // (
    RParenthesis, // )
    Comma,        // ,
    Arrow,        // ->
    Colon,        // :

    // Special
    Comment(String),
    EOF,
}

pub struct Lexer<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn peek(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += c.len_utf8();
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

    fn read_number(&mut self) -> f64 {
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() || c == '.' || c == '-' {
                self.advance();
            } else {
                break;
            }
        }
        self.input[start..self.pos].parse().unwrap_or(0.0)
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

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        let c = match self.peek() {
            Some(c) => c,
            None => return Token::EOF,
        };

        match c {
            '=' => {
                self.advance();
                Token::Equals
            }
            '{' => {
                self.advance();
                Token::LBrace
            }
            '}' => {
                self.advance();
                Token::RBrace
            }
            '(' => {
                self.advance();
                Token::LParenthesis
            }
            ')' => {
                self.advance();
                Token::RParenthesis
            }
            ',' => {
                self.advance();
                Token::Comma
            }
            ':' => {
                self.advance();
                Token::Colon
            }
            '-' => {
                if self.starts_with("->") {
                    self.advance();
                    self.advance();
                    Token::Arrow
                } else if self.peek().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                    Token::Number(self.read_number())
                } else {
                    // For now, treat isolated '-' as part of identifier or error, but here let's assume it might be a negative number start or just skip/error.
                    // Given the spec, '-' usually starts a negative number or arrow.
                    // If it's not arrow, check if next is digit.
                    let next_char_is_digit = self.input[self.pos + 1..]
                        .chars()
                        .next()
                        .map(|c| c.is_ascii_digit())
                        .unwrap_or(false);
                    if next_char_is_digit {
                        Token::Number(self.read_number())
                    } else {
                        // Fallback or error? For now let's just consume it to avoid infinite loop if unexpected
                        self.advance();
                        Token::Identifier("-".to_string())
                    }
                }
            }
            '/' => {
                if self.starts_with("//") {
                    Token::Comment(self.read_comment())
                } else {
                    self.advance();
                    Token::Identifier("/".to_string()) // Unexpected
                }
            }
            _ if c.is_ascii_digit() => Token::Number(self.read_number()),
            _ if c.is_alphabetic() => {
                let ident = self.read_identifier();
                match ident.as_str() {
                    "players" => Token::Players,
                    "state" => Token::State,
                    "baller" => Token::Baller,
                    "position" => Token::Position,
                    "action" => Token::Action,
                    "move" => Token::Move,
                    "screen" => Token::Screen,
                    "pass" => Token::Pass,
                    "before" => Token::Before,
                    "after" => Token::After,
                    "middle" => Token::Middle,
                    _ => Token::Identifier(ident),
                }
            }
            _ => {
                self.advance();
                Token::Identifier(c.to_string()) // Unexpected char
            }
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token();
            if token == Token::EOF {
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
        assert_eq!(
            tokens,
            vec![
                Token::Players,
                Token::Equals,
                Token::LBrace,
                Token::RBrace,
                Token::Arrow,
                Token::Colon,
                Token::EOF
            ]
        );
    }

    #[test]
    fn test_identifiers_and_numbers() {
        let input = "p1 (10, -20.5)";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        assert_eq!(
            tokens,
            vec![
                Token::Identifier("p1".to_string()),
                Token::LParenthesis,
                Token::Number(10.0),
                Token::Comma,
                Token::Number(-20.5),
                Token::RParenthesis,
                Token::EOF
            ]
        );
    }

    #[test]
    fn test_comments() {
        let input = "players // this is a comment\nstate";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        assert_eq!(
            tokens,
            vec![
                Token::Players,
                Token::Comment("this is a comment".to_string()),
                Token::State,
                Token::EOF
            ]
        );
    }

    #[test]
    fn test_full_snippet() {
        let input = r#"""
            action {
                move = {
                    p2 -> (10, 20)
                }
            }
        """#;
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        // Check a few key tokens
        assert!(tokens.contains(&Token::Action));
        assert!(tokens.contains(&Token::Move));
        assert!(tokens.contains(&Token::Arrow));
        assert!(tokens.contains(&Token::Number(10.0)));
    }
}
