use crate::ast::*;
use crate::lexer::{Token, TokenKind, Span};

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken(Token, String),
    UnexpectedEOF,
    InvalidSyntax(String),
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

fn levenshtein(a: &str, b: &str) -> usize {
    let len_a = a.chars().count();
    let len_b = b.chars().count();
    if len_a == 0 { return len_b; }
    if len_b == 0 { return len_a; }

    let mut matrix = vec![vec![0; len_b + 1]; len_a + 1];

    for i in 0..=len_a { matrix[i][0] = i; }
    for j in 0..=len_b { matrix[0][j] = j; }

    for (i, char_a) in a.chars().enumerate() {
        for (j, char_b) in b.chars().enumerate() {
            let cost = if char_a == char_b { 0 } else { 1 };
            matrix[i + 1][j + 1] = std::cmp::min(
                std::cmp::min(matrix[i][j + 1] + 1, matrix[i + 1][j] + 1),
                matrix[i][j] + cost
            );
        }
    }
    matrix[len_a][len_b]
}

fn get_suggestion(input: &str, candidates: &[&str]) -> Option<String> {
    let mut best_match = None;
    let mut min_dist = 3; // Max allowed distance is 2

    for &candidate in candidates {
        let dist = levenshtein(input, candidate);
        if dist < min_dist {
            min_dist = dist;
            best_match = Some(candidate.to_string());
        }
    }
    best_match
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Token {
        if self.pos < self.tokens.len() {
            self.tokens[self.pos].clone()
        } else {
            Token {
                kind: TokenKind::EOF,
                span: Span { start: 0, end: 0, line: 0, column: 0 }
            }
        }
    }

    fn advance(&mut self) -> Token {
        let token = self.peek();
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        token
    }

    fn expect(&mut self, expected_kind: TokenKind) -> Result<(), ParseError> {
        let token = self.peek();
        if std::mem::discriminant(&token.kind) == std::mem::discriminant(&expected_kind) {
            self.advance();
            Ok(())
        } else {
            Err(ParseError::UnexpectedToken(
                token,
                format!("Expected {:?}", expected_kind),
            ))
        }
    }

    fn expect_identifier(&mut self) -> Result<String, ParseError> {
        let token = self.advance();
        match token.kind {
            TokenKind::Identifier(s) => Ok(s),
            _ => Err(ParseError::UnexpectedToken(
                token,
                "Expected Identifier".to_string(),
            )),
        }
    }

    fn consume_if(&mut self, expected_kind: TokenKind) -> bool {
        if std::mem::discriminant(&self.peek().kind) == std::mem::discriminant(&expected_kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn parse_coordinate(&mut self) -> Result<(f64, f64), ParseError> {
        self.expect(TokenKind::LParenthesis)?;
        let token = self.advance();
        let x = match token.kind {
            TokenKind::Number(n) => n,
            _ => {
                return Err(ParseError::UnexpectedToken(
                    token,
                    "Expected Number for X".to_string(),
                ));
            }
        };
        self.expect(TokenKind::Comma)?;
        let token = self.advance();
        let y = match token.kind {
            TokenKind::Number(n) => n,
            _ => {
                return Err(ParseError::UnexpectedToken(
                    token,
                    "Expected Number for Y".to_string(),
                ));
            }
        };
        self.expect(TokenKind::RParenthesis)?;
        Ok((x, y))
    }

    pub fn parse(&mut self) -> Result<Playbook, ParseError> {
        let mut players = Vec::new();
        let mut state = State::default();
        let mut action = Action::default();

        while self.peek().kind != TokenKind::EOF {
            match self.peek().kind {
                TokenKind::Players => {
                    self.advance(); // consume 'players'
                    self.expect(TokenKind::Equals)?;
                    self.expect(TokenKind::LBrace)?;
                    while self.peek().kind != TokenKind::RBrace && self.peek().kind != TokenKind::EOF {
                        players.push(self.expect_identifier()?);
                        if self.peek().kind == TokenKind::Comma {
                            self.advance();
                        }
                    }
                    self.expect(TokenKind::RBrace)?;
                }
                TokenKind::State => {
                    self.advance();
                    self.expect(TokenKind::Equals)?;
                    self.expect(TokenKind::LBrace)?;
                    state = self.parse_state_block()?;
                    self.expect(TokenKind::RBrace)?;
                }
                TokenKind::Action => {
                    self.advance();
                    self.expect(TokenKind::Equals)?;
                    self.expect(TokenKind::LBrace)?; 
                    action = self.parse_action_block()?;
                    self.expect(TokenKind::RBrace)?;
                }
                _ => {
                    let token = self.peek();
                    let mut msg = "Expected section start (players, state, action)".to_string();
                    if let TokenKind::Identifier(ref s) = token.kind {
                         if let Some(sugg) = get_suggestion(s, &["players", "state", "action"]) {
                             msg = format!("Expected section start. Did you mean '{}'?", sugg);
                         }
                    }
                    return Err(ParseError::UnexpectedToken(token, msg));
                }
            }
        }

        Ok(Playbook {
            players,
            state,
            action,
        })
    }

    fn parse_state_block(&mut self) -> Result<State, ParseError> {
        let mut state = State::default();
        while self.peek().kind != TokenKind::RBrace && self.peek().kind != TokenKind::EOF {
            match self.peek().kind {
                TokenKind::Baller => {
                    self.advance();
                    self.expect(TokenKind::Equals)?;
                    state.baller = Some(self.expect_identifier()?);
                    self.consume_if(TokenKind::Comma); 
                }
                TokenKind::Position => {
                    self.advance();
                    self.expect(TokenKind::Equals)?;
                    self.expect(TokenKind::LBrace)?;
                    while self.peek().kind != TokenKind::RBrace {
                        let player = self.expect_identifier()?;
                        self.expect(TokenKind::Equals)?;
                        let coord = self.parse_coordinate()?;
                        state.positions.insert(player, coord);
                        if self.peek().kind == TokenKind::Comma {
                            self.advance();
                        }
                    }
                    self.expect(TokenKind::RBrace)?;
                    self.consume_if(TokenKind::Comma);
                }
                _ => {
                    let token = self.peek();
                    let mut msg = "Expected state property (baller, position)".to_string();
                    if let TokenKind::Identifier(ref s) = token.kind {
                         if let Some(sugg) = get_suggestion(s, &["baller", "position"]) {
                             msg = format!("Expected state property. Did you mean '{}'?", sugg);
                         }
                    }
                    return Err(ParseError::UnexpectedToken(token, msg));
                }
            }
        }
        Ok(state)
    }

    fn parse_action_block(&mut self) -> Result<Action, ParseError> {
        let mut action = Action::default();
        while self.peek().kind != TokenKind::RBrace && self.peek().kind != TokenKind::EOF {
            match self.peek().kind {
                TokenKind::Move => {
                    self.advance();
                    self.expect(TokenKind::Equals)?;
                    self.expect(TokenKind::LBrace)?;
                    while self.peek().kind != TokenKind::RBrace {
                        let player = self.expect_identifier()?;
                        self.expect(TokenKind::Arrow)?;
                        let target = self.parse_coordinate()?;
                        action.moves.push(MoveAction { player, target });
                        if self.peek().kind == TokenKind::Comma {
                            self.advance();
                        }
                    }
                    self.expect(TokenKind::RBrace)?;
                    self.consume_if(TokenKind::Comma);
                }
                TokenKind::Screen => {
                    self.advance();
                    self.expect(TokenKind::Equals)?;
                    self.expect(TokenKind::LBrace)?;
                    while self.peek().kind != TokenKind::RBrace {
                        let player = self.expect_identifier()?;
                        self.expect(TokenKind::Arrow)?;
                        let target = self.expect_identifier()?;
                        let mut timing = Timing::None;
                        if self.peek().kind == TokenKind::Colon {
                            self.advance();
                            match self.peek().kind {
                                TokenKind::Before => {
                                    self.advance();
                                    timing = Timing::Before;
                                }
                                TokenKind::After => {
                                    self.advance();
                                    timing = Timing::After;
                                }
                                TokenKind::Middle => {
                                    self.advance();
                                    timing = Timing::Middle;
                                }
                                _ => {
                                    return Err(ParseError::UnexpectedToken(
                                        self.peek(),
                                        "Expected timing (before, after, middle)".to_string(),
                                    ));
                                }
                            }
                        }
                        action.screens.push(ScreenAction {
                            player,
                            target,
                            timing,
                        });
                        if self.peek().kind == TokenKind::Comma {
                            self.advance();
                        }
                    }
                    self.expect(TokenKind::RBrace)?;
                    self.consume_if(TokenKind::Comma);
                }
                TokenKind::Pass => {
                    self.advance();
                    self.expect(TokenKind::Equals)?;
                    self.expect(TokenKind::LBrace)?;
                    while self.peek().kind != TokenKind::RBrace {
                        let from = self.expect_identifier()?;
                        self.expect(TokenKind::Arrow)?;
                        let to = self.expect_identifier()?;
                        let mut timing = Timing::None;
                        if self.peek().kind == TokenKind::Colon {
                            self.advance();
                            match self.peek().kind {
                                TokenKind::Before => {
                                    self.advance();
                                    timing = Timing::Before;
                                }
                                TokenKind::After => {
                                    self.advance();
                                    timing = Timing::After;
                                }
                                _ => {
                                    return Err(ParseError::UnexpectedToken(
                                        self.peek(),
                                        "Expected timing (before, after)".to_string(),
                                    ));
                                }
                            }
                        }
                        action.passes.push(PassAction { from, to, timing });
                        if self.peek().kind == TokenKind::Comma {
                            self.advance();
                        }
                    }
                    self.expect(TokenKind::RBrace)?;
                    self.consume_if(TokenKind::Comma);
                }
                _ => {
                    let token = self.peek();
                    let mut msg = "Expected action property (move, screen, pass)".to_string();
                    if let TokenKind::Identifier(ref s) = token.kind {
                         if let Some(sugg) = get_suggestion(s, &["move", "screen", "pass"]) {
                             msg = format!("Expected action property. Did you mean '{}'?", sugg);
                         }
                    }
                    return Err(ParseError::UnexpectedToken(token, msg));
                }
            }
        }
        Ok(action)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    #[test]
    fn test_parse_players() {
        let input = "players = { p1, p2 }";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let playbook = parser.parse().unwrap();
        assert_eq!(playbook.players, vec!["p1", "p2"]);
    }

    #[test]
    fn test_parse_full_example() {
        let input = r#"
        players = { p1, p2 }
        state = {
            baller = p1,
            position = {
                p1 = (0, 0)
                p2 = (10, 20)
            },
        }
        action = {
            move = {
                p2 -> (30, 40)
            },
            pass = {
                p1 -> p2:after
            },
        }
        "#;
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let playbook = parser.parse().unwrap();

        assert_eq!(playbook.players.len(), 2);
        assert_eq!(playbook.state.baller, Some("p1".to_string()));
        assert_eq!(playbook.state.positions.get("p1"), Some(&(0.0, 0.0)));
        assert_eq!(playbook.state.positions.get("p2"), Some(&(10.0, 20.0)));

        assert_eq!(playbook.action.moves.len(), 1);
        assert_eq!(playbook.action.moves[0].player, "p2");
        assert_eq!(playbook.action.moves[0].target, (30.0, 40.0));

        assert_eq!(playbook.action.passes.len(), 1);
        assert_eq!(playbook.action.passes[0].from, "p1");
        assert_eq!(playbook.action.passes[0].to, "p2");
        match playbook.action.passes[0].timing {
            Timing::After => {}
            _ => panic!("Expected After timing"),
        }
    }

    #[test]
    fn test_levenshtein() {
        assert_eq!(levenshtein("kitten", "sitting"), 3);
        assert_eq!(levenshtein("action", "aciton"), 2);
        assert_eq!(levenshtein("pass", "pas"), 1);
    }
}
