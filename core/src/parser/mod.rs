use crate::ast::*;
use crate::lexer::Token;

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

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Token {
        if self.pos < self.tokens.len() {
            self.tokens[self.pos].clone()
        } else {
            Token::EOF
        }
    }

    fn advance(&mut self) -> Token {
        let token = self.peek();
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        token
    }

    fn expect(&mut self, expected: Token) -> Result<(), ParseError> {
        let token = self.peek();
        // Simple equality check for tokens without data
        if std::mem::discriminant(&token) == std::mem::discriminant(&expected) {
            self.advance();
            Ok(())
        } else {
            Err(ParseError::UnexpectedToken(
                token,
                format!("Expected {:?}", expected),
            ))
        }
    }

    fn expect_identifier(&mut self) -> Result<String, ParseError> {
        match self.advance() {
            Token::Identifier(s) => Ok(s),
            t => Err(ParseError::UnexpectedToken(
                t,
                "Expected Identifier".to_string(),
            )),
        }
    }

    // Helper to consume a token if it matches, otherwise do nothing
    fn consume_if(&mut self, expected_discriminant: Token) -> bool {
        if std::mem::discriminant(&self.peek()) == std::mem::discriminant(&expected_discriminant) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn parse_coordinate(&mut self) -> Result<(f64, f64), ParseError> {
        self.expect(Token::LParenthesis)?;
        let x = match self.advance() {
            Token::Number(n) => n,
            t => {
                return Err(ParseError::UnexpectedToken(
                    t,
                    "Expected Number for X".to_string(),
                ));
            }
        };
        self.expect(Token::Comma)?;
        let y = match self.advance() {
            Token::Number(n) => n,
            t => {
                return Err(ParseError::UnexpectedToken(
                    t,
                    "Expected Number for Y".to_string(),
                ));
            }
        };
        self.expect(Token::RParenthesis)?;
        Ok((x, y))
    }

    pub fn parse(&mut self) -> Result<Playbook, ParseError> {
        let mut players = Vec::new();
        let mut state = State::default();
        let mut action = Action::default();

        while self.peek() != Token::EOF {
            match self.peek() {
                Token::Players => {
                    self.advance(); // consume 'players'
                    self.expect(Token::Equals)?;
                    self.expect(Token::LBrace)?;
                    while self.peek() != Token::RBrace {
                        players.push(self.expect_identifier()?);
                        if self.peek() == Token::Comma {
                            self.advance();
                        }
                    }
                    self.expect(Token::RBrace)?;
                }
                Token::State => {
                    self.advance();
                    self.expect(Token::Equals)?;
                    self.expect(Token::LBrace)?;
                    state = self.parse_state_block()?;
                    self.expect(Token::RBrace)?;
                }
                Token::Action => {
                    self.advance();
                    self.expect(Token::Equals)?;
                    self.expect(Token::LBrace)?; // Action doesn't have '=', based on quickStart? Wait, quickStart says `action {` but others have `=`. quickStart: "action {"
                    // Let's re-read spec. "action {" no equals.
                    action = self.parse_action_block()?;
                    self.expect(Token::RBrace)?;
                }
                _ => {
                    return Err(ParseError::UnexpectedToken(
                        self.peek(),
                        "Expected section start".to_string(),
                    ));
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
        while self.peek() != Token::RBrace && self.peek() != Token::EOF {
            match self.peek() {
                Token::Baller => {
                    self.advance();
                    self.expect(Token::Equals)?;
                    state.baller = Some(self.expect_identifier()?);
                    self.consume_if(Token::Comma); // optional comma? Spec doesn't show it but typical
                }
                Token::Position => {
                    self.advance();
                    self.expect(Token::Equals)?;
                    self.expect(Token::LBrace)?;
                    while self.peek() != Token::RBrace {
                        let player = self.expect_identifier()?;
                        self.expect(Token::Equals)?;
                        let coord = self.parse_coordinate()?;
                        state.positions.insert(player, coord);
                        // Optional newline/comma handling implicitly by loop
                        // But we might have commas
                        if self.peek() == Token::Comma {
                            self.advance();
                        }
                    }
                    self.expect(Token::RBrace)?;
                    self.consume_if(Token::Comma);
                }
                _ => {
                    return Err(ParseError::UnexpectedToken(
                        self.peek(),
                        "Expected state property".to_string(),
                    ));
                }
            }
        }
        Ok(state)
    }

    fn parse_action_block(&mut self) -> Result<Action, ParseError> {
        let mut action = Action::default();
        while self.peek() != Token::RBrace && self.peek() != Token::EOF {
            match self.peek() {
                Token::Move => {
                    self.advance();
                    self.expect(Token::Equals)?;
                    self.expect(Token::LBrace)?;
                    while self.peek() != Token::RBrace {
                        // p2 -> (xx2, yy2)
                        let player = self.expect_identifier()?;
                        self.expect(Token::Arrow)?;
                        let target = self.parse_coordinate()?;
                        action.moves.push(MoveAction { player, target });
                        if self.peek() == Token::Comma {
                            self.advance();
                        }
                    }
                    self.expect(Token::RBrace)?;
                    self.consume_if(Token::Comma);
                }
                Token::Screen => {
                    self.advance();
                    self.expect(Token::Equals)?;
                    self.expect(Token::LBrace)?;
                    while self.peek() != Token::RBrace {
                        // p3 -> p2:before
                        let player = self.expect_identifier()?;
                        self.expect(Token::Arrow)?;
                        let target = self.expect_identifier()?;
                        let mut timing = Timing::None;
                        if self.peek() == Token::Colon {
                            self.advance();
                            match self.peek() {
                                Token::Before => {
                                    self.advance();
                                    timing = Timing::Before;
                                }
                                Token::After => {
                                    self.advance();
                                    timing = Timing::After;
                                }
                                Token::Middle => {
                                    self.advance();
                                    timing = Timing::Middle;
                                }
                                t => {
                                    return Err(ParseError::UnexpectedToken(
                                        t,
                                        "Expected timing".to_string(),
                                    ));
                                }
                            }
                        }
                        action.screens.push(ScreenAction {
                            player,
                            target,
                            timing,
                        });
                        if self.peek() == Token::Comma {
                            self.advance();
                        }
                    }
                    self.expect(Token::RBrace)?;
                    self.consume_if(Token::Comma);
                }
                Token::Pass => {
                    self.advance();
                    self.expect(Token::Equals)?;
                    self.expect(Token::LBrace)?;
                    while self.peek() != Token::RBrace {
                        // p1 -> p2:after
                        let from = self.expect_identifier()?;
                        self.expect(Token::Arrow)?;
                        let to = self.expect_identifier()?;
                        let mut timing = Timing::None;
                        if self.peek() == Token::Colon {
                            self.advance();
                            match self.peek() {
                                Token::Before => {
                                    self.advance();
                                    timing = Timing::Before;
                                }
                                Token::After => {
                                    self.advance();
                                    timing = Timing::After;
                                }
                                Token::Middle => {
                                    self.advance();
                                    timing = Timing::Middle;
                                }
                                t => {
                                    return Err(ParseError::UnexpectedToken(
                                        t,
                                        "Expected timing".to_string(),
                                    ));
                                }
                            }
                        }
                        action.passes.push(PassAction { from, to, timing });
                        if self.peek() == Token::Comma {
                            self.advance();
                        }
                    }
                    self.expect(Token::RBrace)?;
                    self.consume_if(Token::Comma);
                }
                _ => {
                    return Err(ParseError::UnexpectedToken(
                        self.peek(),
                        "Expected action property".to_string(),
                    ));
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
        // match enum variant roughly
        match playbook.action.passes[0].timing {
            Timing::After => {}
            _ => panic!("Expected After timing"),
        }
    }
}
