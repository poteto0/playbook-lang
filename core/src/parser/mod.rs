use crate::ast::*;
use crate::constants::{DEFAULT_BEZIER_CURVE_FACTOR, MAX_ACTIONS_PER_PHASE};
use crate::lexer::{Span, Token, TokenKind};
use std::fmt;

#[derive(Debug, Clone)]
pub enum ParseError {
    UnexpectedToken(Token, String),
    UnexpectedEOF,
    InvalidSyntax(Token, String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::UnexpectedToken(token, msg) => write!(
                f,
                "unexpected token at line {}, column {}: {}",
                token.span.line, token.span.column, msg
            ),
            ParseError::UnexpectedEOF => write!(f, "unexpected end of input"),
            ParseError::InvalidSyntax(token, msg) => write!(
                f,
                "invalid syntax at line {}, column {}: {}",
                token.span.line, token.span.column, msg
            ),
        }
    }
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    errors: Vec<ParseError>,
    comments: Vec<(Span, String)>,
}

fn levenshtein(a: &str, b: &str) -> usize {
    let len_a = a.chars().count();
    let len_b = b.chars().count();
    if len_a == 0 {
        return len_b;
    }
    if len_b == 0 {
        return len_a;
    }

    let mut matrix = vec![vec![0; len_b + 1]; len_a + 1];

    for (i, column) in matrix.iter_mut().enumerate() {
        column[0] = i;
    }
    for (j, row) in matrix[0].iter_mut().enumerate() {
        *row = j;
    }

    for (i, char_a) in a.chars().enumerate() {
        for (j, char_b) in b.chars().enumerate() {
            let cost = if char_a == char_b { 0 } else { 1 };
            matrix[i + 1][j + 1] = std::cmp::min(
                std::cmp::min(matrix[i][j + 1] + 1, matrix[i + 1][j] + 1),
                matrix[i][j] + cost,
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
        Self {
            tokens,
            pos: 0,
            errors: Vec::new(),
            comments: Vec::new(),
        }
    }

    fn peek(&self) -> Token {
        if self.pos < self.tokens.len() {
            self.tokens[self.pos].clone()
        } else {
            Token {
                kind: TokenKind::EOF,
                span: Span {
                    start: 0,
                    end: 0,
                    line: 0,
                    column: 0,
                },
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

    fn error(&mut self, err: ParseError) {
        self.errors.push(err);
    }

    fn recover_until(&mut self, stop_tokens: &[TokenKind]) {
        while self.peek().kind != TokenKind::EOF {
            let current = &self.peek().kind;
            // Check discriminants since TokenKind carries data
            let found = stop_tokens
                .iter()
                .any(|t| std::mem::discriminant(t) == std::mem::discriminant(current));
            if found {
                return;
            }
            self.advance();
        }
    }

    fn expect(&mut self, expected_kind: TokenKind) -> Result<(), ParseError> {
        let token = self.peek();
        if std::mem::discriminant(&token.kind) == std::mem::discriminant(&expected_kind) {
            Ok(())
        } else {
            Err(ParseError::UnexpectedToken(
                token.clone(),
                format!("Expected `{}`, but found `{}`", expected_kind, token.kind),
            ))
        }
    }

    fn expect_and_advance(&mut self, expected_kind: TokenKind) -> Result<(), ParseError> {
        let token = self.peek();
        if std::mem::discriminant(&token.kind) == std::mem::discriminant(&expected_kind) {
            self.advance();
            Ok(())
        } else {
            Err(ParseError::UnexpectedToken(
                token.clone(),
                format!("Expected `{}`, but found `{}`", expected_kind, token.kind),
            ))
        }
    }

    fn expect_identifier(&mut self) -> Result<String, ParseError> {
        let token = self.advance();
        match token.clone().kind {
            TokenKind::Identifier(s) => Ok(s),
            unexpected => Err(ParseError::UnexpectedToken(
                token,
                format!("Expected Identifier, but found '{}'", unexpected),
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
        self.expect_and_advance(TokenKind::LParenthesis)?;
        let token = self.advance();
        let x = match token.clone().kind {
            TokenKind::Number(n) => n,
            unexpected => {
                return Err(ParseError::UnexpectedToken(
                    token,
                    format!(
                        "Expected a numeric value for x-coordinate, but received '{}'",
                        unexpected
                    ),
                ));
            }
        };
        self.expect_and_advance(TokenKind::Comma)?;
        let token = self.advance();
        let y = match token.clone().kind {
            TokenKind::Number(n) => n,
            unexpected => {
                return Err(ParseError::UnexpectedToken(
                    token,
                    format!(
                        "Expected a numeric value for y-coordinate, but received '{}'",
                        unexpected
                    ),
                ));
            }
        };
        self.expect_and_advance(TokenKind::RParenthesis)?;
        Ok((x, y))
    }

    pub fn parse(&mut self) -> (Playbook, Vec<ParseError>) {
        let mut players = Vec::new();
        let mut state = State::default();
        let mut actions = Vec::new();

        while self.peek().kind != TokenKind::EOF {
            match self.peek().kind {
                TokenKind::Comment(ref s) => {
                    let token = self.advance();
                    self.comments.push((token.span, s.clone()));
                }
                TokenKind::Players => {
                    self.parse_players_section(&mut players);
                }
                TokenKind::State => {
                    self.parse_state_section(&mut state);
                }
                TokenKind::Action => {
                    if let Some(action) = self.parse_single_action_section() {
                        actions.push(action);
                    }
                }
                TokenKind::Actions => {
                    self.parse_actions_section(&mut actions);
                }
                TokenKind::Error(msg) => {
                    let token = self.advance();
                    self.error(ParseError::UnexpectedToken(token, msg));
                }
                _ => {
                    let token = self.peek();
                    let mut msg = format!(
                        "Expected section start (players, state, action, actions), but found '{}'",
                        token.clone().kind
                    );
                    let TokenKind::Identifier(ref s) = token.kind else {
                        self.error(ParseError::UnexpectedToken(token, msg));
                        self.advance();
                        continue;
                    };
                    if let Some(sugg) =
                        get_suggestion(s, &["players", "state", "action", "actions"])
                    {
                        msg = format!("Expected section start. Did you mean '{}'?", sugg);
                    }
                    self.error(ParseError::UnexpectedToken(token, msg));
                    self.advance();
                }
            }
        }

        (
            Playbook {
                players,
                state,
                actions,
                comments: self.comments.clone(),
            },
            self.errors.clone(),
        )
    }

    fn parse_players_section(&mut self, players: &mut Vec<String>) {
        self.advance(); // consume 'players'
        if let Err(e) = self.expect_and_advance(TokenKind::Equals) {
            self.error(e);
            // Removed return to attempt recovery
        }
        if let Err(e) = self.expect_and_advance(TokenKind::LBrace) {
            self.error(e);
            // Removed return
        }
        while self.peek().kind != TokenKind::RBrace && self.peek().kind != TokenKind::EOF {
            match self.expect_identifier() {
                Ok(p) => players.push(p),
                Err(e) => {
                    self.error(e);
                    self.recover_until(&[TokenKind::Comma, TokenKind::RBrace]);
                }
            }
            self.consume_if(TokenKind::Comma);
        }
        if let Err(e) = self.expect_and_advance(TokenKind::RBrace) {
            self.error(e);
        }
    }

    fn parse_state_section(&mut self, state: &mut State) {
        self.advance(); // consume 'state'
        if let Err(e) = self.expect_and_advance(TokenKind::Equals) {
            self.error(e);
        }
        if let Err(e) = self.expect_and_advance(TokenKind::LBrace) {
            self.error(e);
        }
        self.parse_state_block(state);
        if let Err(e) = self.expect_and_advance(TokenKind::RBrace) {
            self.error(e);
        }
    }

    fn parse_single_action_section(&mut self) -> Option<Action> {
        self.advance(); // consume 'action'
        if let Err(e) = self.expect_and_advance(TokenKind::Equals) {
            self.error(e);
        }
        if let Err(e) = self.expect_and_advance(TokenKind::LBrace) {
            self.error(e);
        }
        let action = self.parse_action_block();
        if let Err(e) = self.expect_and_advance(TokenKind::RBrace) {
            self.error(e);
        }
        Some(action)
    }

    fn parse_actions_section(&mut self, actions: &mut Vec<Action>) {
        self.advance(); // consume 'actions'
        if let Err(e) = self.expect_and_advance(TokenKind::Equals) {
            self.error(e);
        }
        if let Err(e) = self.expect_and_advance(TokenKind::LBracket) {
            self.error(e);
        }

        while self.peek().kind != TokenKind::RBracket && self.peek().kind != TokenKind::EOF {
            if let TokenKind::Comment(ref s) = self.peek().kind {
                let token = self.advance();
                self.comments.push((token.span, s.clone()));
                continue;
            }

            if let Err(e) = self.expect(TokenKind::Action) {
                self.error(e);
                self.recover_until(&[TokenKind::Comma, TokenKind::RBracket]);
                self.consume_if(TokenKind::Comma);
                continue;
            }

            if actions.len() >= MAX_ACTIONS_PER_PHASE {
                self.error(ParseError::InvalidSyntax(
                    self.peek().clone(),
                    format!("Maximum of {MAX_ACTIONS_PER_PHASE} actions allowed"),
                ));
            }

            if let Some(action) = self.parse_single_action_section() {
                actions.push(action);
            }

            self.consume_if(TokenKind::Comma);
        }
        if let Err(e) = self.expect_and_advance(TokenKind::RBracket) {
            self.error(e);
        }
    }

    fn parse_state_block(&mut self, state: &mut State) {
        while self.peek().kind != TokenKind::RBrace && self.peek().kind != TokenKind::EOF {
            match self.peek().kind {
                TokenKind::Baller => {
                    self.advance();
                    if let Err(e) = self.expect_and_advance(TokenKind::Equals) {
                        self.error(e);
                        self.recover_until(&[TokenKind::Comma, TokenKind::RBrace]);
                    } else {
                        match self.expect_identifier() {
                            Ok(id) => state.baller = Some(id),
                            Err(e) => self.error(e),
                        }
                    }
                    self.consume_if(TokenKind::Comma);
                }
                TokenKind::Position => {
                    self.advance();
                    if let Err(e) = self.expect_and_advance(TokenKind::Equals) {
                        self.error(e);
                        self.recover_until(&[TokenKind::Comma, TokenKind::RBrace]);
                    } else if let Err(e) = self.expect_and_advance(TokenKind::LBrace) {
                        self.error(e);
                        self.recover_until(&[TokenKind::Comma, TokenKind::RBrace]);
                    } else {
                        while self.peek().kind != TokenKind::RBrace
                            && self.peek().kind != TokenKind::EOF
                        {
                            match self.expect_identifier() {
                                Ok(player) => {
                                    if let Err(e) = self.expect_and_advance(TokenKind::Equals) {
                                        self.error(e);
                                    } else {
                                        match self.parse_coordinate() {
                                            Ok(coord) => {
                                                state.positions.insert(player, coord);
                                            }
                                            Err(e) => self.error(e),
                                        }
                                    }
                                }
                                Err(e) => self.error(e),
                            }
                            self.consume_if(TokenKind::Comma);
                        }
                        if let Err(e) = self.expect_and_advance(TokenKind::RBrace) {
                            self.error(e);
                        }
                    }
                    self.consume_if(TokenKind::Comma);
                }
                _ => {
                    let token = self.peek();
                    let mut msg = "Expected state property (baller, position)".to_string();
                    let TokenKind::Identifier(ref s) = token.kind else {
                        self.error(ParseError::UnexpectedToken(token, msg));
                        self.recover_until(&[TokenKind::Comma, TokenKind::RBrace]);
                        self.consume_if(TokenKind::Comma);
                        continue;
                    };
                    if let Some(sugg) = get_suggestion(s, &["baller", "position"]) {
                        msg = format!("Expected state property. Did you mean '{}'?", sugg);
                    }
                    self.error(ParseError::UnexpectedToken(token, msg));
                    self.recover_until(&[TokenKind::Comma, TokenKind::RBrace]);
                    self.consume_if(TokenKind::Comma);
                }
            }
        }
    }

    fn expect_arrow(&mut self) -> Result<PathType, ParseError> {
        let token = self.peek();
        match token.kind {
            TokenKind::Arrow => {
                self.advance();
                Ok(PathType::Straight)
            }
            TokenKind::CurveArrow(ref s) => {
                self.advance();
                if s == "default" {
                    return Ok(PathType::Curve(CurveDirection::Left(
                        DEFAULT_BEZIER_CURVE_FACTOR,
                    )));
                }

                let parts: Vec<&str> = s.split(':').collect();
                let dir_str = parts[0];
                let factor = if parts.len() > 1 {
                    let factor_str = parts[1];
                    if factor_str.len() > 3 {
                        return Err(ParseError::InvalidSyntax(
                            token.clone(),
                            format!(
                                "Curve factor must be at most 3 characters (e.g. 0.5): {}",
                                factor_str
                            ),
                        ));
                    }
                    factor_str.parse::<f64>().map_err(|_| {
                        ParseError::InvalidSyntax(
                            token.clone(),
                            format!("Invalid curve factor: {}", factor_str),
                        )
                    })?
                } else {
                    DEFAULT_BEZIER_CURVE_FACTOR
                };

                match dir_str {
                    "l" | "left" => Ok(PathType::Curve(CurveDirection::Left(factor))),
                    "r" | "right" => Ok(PathType::Curve(CurveDirection::Right(factor))),
                    _ => Err(ParseError::InvalidSyntax(
                        self.peek().clone(),
                        format!("Unknown curve direction: {}", dir_str),
                    )),
                }
            }
            TokenKind::Error(ref msg) => {
                let token = self.peek();
                Err(ParseError::UnexpectedToken(token, msg.clone()))
            }
            unexpected => Err(ParseError::UnexpectedToken(
                self.peek().clone(),
                format!("Expected '->' or '~>', but found '{}'", unexpected),
            )),
        }
    }

    fn parse_action_block(&mut self) -> Action {
        let mut action = Action::default();
        while self.peek().kind != TokenKind::RBrace && self.peek().kind != TokenKind::EOF {
            match self.peek().kind {
                TokenKind::Move => {
                    self.advance();
                    if let Err(e) = self.expect_and_advance(TokenKind::Equals) {
                        self.error(e);
                    } else if let Err(e) = self.expect_and_advance(TokenKind::LBrace) {
                        self.error(e);
                    } else {
                        while self.peek().kind != TokenKind::RBrace
                            && self.peek().kind != TokenKind::EOF
                        {
                            // Try parsing a move line: player -> target
                            match self.expect_identifier() {
                                Ok(player) => match self.expect_arrow() {
                                    Ok(path_type) => match self.parse_coordinate() {
                                        Ok(target) => {
                                            action.moves.push(MoveAction {
                                                player,
                                                target,
                                                path_type,
                                                span: self.peek().clone().span,
                                            });
                                        }
                                        Err(e) => self.error(e),
                                    },
                                    Err(e) => self.error(e),
                                },
                                Err(e) => self.error(e),
                            }
                            self.consume_if(TokenKind::Comma);
                        }
                        if let Err(e) = self.expect_and_advance(TokenKind::RBrace) {
                            self.error(e);
                        }
                    }
                    self.consume_if(TokenKind::Comma);
                }
                TokenKind::Screen => {
                    self.advance();
                    if let Err(e) = self.expect_and_advance(TokenKind::Equals) {
                        self.error(e);
                    } else if let Err(e) = self.expect_and_advance(TokenKind::LBrace) {
                        self.error(e);
                    } else {
                        while self.peek().kind != TokenKind::RBrace
                            && self.peek().kind != TokenKind::EOF
                        {
                            match self.expect_identifier() {
                                Ok(player) => match self.expect_arrow() {
                                    Ok(path_type) => {
                                        let target_res =
                                            if self.peek().kind == TokenKind::LParenthesis {
                                                self.parse_coordinate()
                                                    .map(|(x, y)| ScreenTarget::Coordinate(x, y))
                                            } else {
                                                self.expect_identifier().map(ScreenTarget::Player)
                                            };

                                        match target_res {
                                            Ok(target) => {
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
                                                            self.error(ParseError::UnexpectedToken(
                                                                self.peek(),
                                                                "Expected timing".to_string(),
                                                            ))
                                                        }
                                                    }
                                                }
                                                action.screens.push(ScreenAction {
                                                    player,
                                                    target,
                                                    timing,
                                                    path_type,
                                                    span: self.peek().clone().span,
                                                });
                                            }
                                            Err(e) => self.error(e),
                                        }
                                    }
                                    Err(e) => self.error(e),
                                },
                                Err(e) => self.error(e),
                            }
                            self.consume_if(TokenKind::Comma);
                        }
                        if let Err(e) = self.expect_and_advance(TokenKind::RBrace) {
                            self.error(e);
                        }
                    }
                    self.consume_if(TokenKind::Comma);
                }
                TokenKind::Pass => {
                    self.advance();
                    if let Err(e) = self.expect_and_advance(TokenKind::Equals) {
                        self.error(e);
                    } else if let Err(e) = self.expect_and_advance(TokenKind::LBrace) {
                        self.error(e);
                    } else {
                        while self.peek().kind != TokenKind::RBrace
                            && self.peek().kind != TokenKind::EOF
                        {
                            match self.expect_identifier() {
                                Ok(from) => {
                                    if let Err(e) = self.expect_and_advance(TokenKind::Arrow) {
                                        self.error(e);
                                    } else {
                                        match self.expect_identifier() {
                                            Ok(to) => {
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
                                                            self.error(ParseError::UnexpectedToken(
                                                                self.peek(),
                                                                "Expected timing".to_string(),
                                                            ))
                                                        }
                                                    }
                                                }
                                                action.passes.push(PassAction {
                                                    from,
                                                    to,
                                                    timing,
                                                    span: self.peek().clone().span,
                                                });
                                            }
                                            Err(e) => self.error(e),
                                        }
                                    }
                                }
                                Err(e) => self.error(e),
                            }
                            self.consume_if(TokenKind::Comma);
                        }
                        if let Err(e) = self.expect_and_advance(TokenKind::RBrace) {
                            self.error(e);
                        }
                    }
                    self.consume_if(TokenKind::Comma);
                }
                _ => {
                    let token = self.peek();
                    let mut msg = "Expected action property (move, screen, pass)".to_string();
                    let TokenKind::Identifier(ref s) = token.kind else {
                        self.error(ParseError::UnexpectedToken(token, msg));
                        self.recover_until(&[TokenKind::Comma, TokenKind::RBrace]);
                        self.consume_if(TokenKind::Comma);
                        continue;
                    };
                    if let Some(sugg) = get_suggestion(s, &["move", "screen", "pass"]) {
                        msg = format!("Expected action property. Did you mean '{}'?", sugg);
                    }
                    self.error(ParseError::UnexpectedToken(token, msg));
                    self.recover_until(&[TokenKind::Comma, TokenKind::RBrace]);
                    self.consume_if(TokenKind::Comma);
                }
            }
        }
        action
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
        let (playbook, errors) = parser.parse();
        assert!(errors.is_empty());
        assert_eq!(playbook.players, vec!["p1", "p2"]);
    }

    #[test]
    fn test_parse_error_recovery() {
        // Missing equals
        let input = "players { p1 }";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (playbook, errors) = parser.parse();

        // Now that we recover, we expect 1 error (expected =) and p1 should be parsed!
        assert_eq!(errors.len(), 1);
        match &errors[0] {
            ParseError::UnexpectedToken(_, msg) => assert!(msg.contains("Expected `=`")),
            _ => panic!("Expected UnexpectedToken"),
        }
        // p1 should be in players
        assert_eq!(playbook.players, vec!["p1"]);
    }

    #[test]
    fn test_parse_state_recovery() {
        // Invalid property inside state
        let input = "state = { baller = p1, invalid = 123, position = { p1 = (0,0) } }";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (playbook, errors) = parser.parse();

        assert_eq!(errors.len(), 1);
        assert_eq!(playbook.state.baller, Some("p1".to_string()));
        assert!(playbook.state.positions.contains_key("p1"));
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
        let (playbook, errors) = parser.parse();
        if !errors.is_empty() {
            panic!("Unexpected errors: {:?}", errors);
        }
        assert!(errors.is_empty());

        assert_eq!(playbook.players.len(), 2);
        assert_eq!(playbook.state.baller, Some("p1".to_string()));
        assert_eq!(playbook.state.positions.get("p1"), Some(&(0.0, 0.0)));
        assert_eq!(playbook.state.positions.get("p2"), Some(&(10.0, 20.0)));

        assert_eq!(playbook.actions.len(), 1);
        assert_eq!(playbook.actions[0].moves.len(), 1);
        assert_eq!(playbook.actions[0].moves[0].player, "p2");
        assert_eq!(playbook.actions[0].moves[0].target, (30.0, 40.0));

        assert_eq!(playbook.actions[0].passes.len(), 1);
        assert_eq!(playbook.actions[0].passes[0].from, "p1");
        assert_eq!(playbook.actions[0].passes[0].to, "p2");
        match playbook.actions[0].passes[0].timing {
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

    #[test]
    fn test_parse_curved_path_with_factor() {
        let input = r#"
        players = { p1 }
        state = { }
        action = {
            move = {
                p1 ~[l:0.5]> (10, 10),
                p1 ~[r:0.1]> (20, 20)
            },
        }
        "#;
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (playbook, _) = parser.parse();

        match playbook.actions[0].moves[0].path_type {
            PathType::Curve(CurveDirection::Left(f)) => assert_eq!(f, 0.5),
            _ => panic!("Expected Left Curve with 0.5"),
        }

        match playbook.actions[0].moves[1].path_type {
            PathType::Curve(CurveDirection::Right(f)) => assert_eq!(f, 0.1),
            _ => panic!("Expected Right Curve with 0.1"),
        }
    }

    #[test]
    fn test_parse_invalid_curve_factor_length() {
        let input = r#"
        players = { p1 }
        state = { }
        action = {
            move = {
                p1 ~[l:0.15]> (10, 10)
            },
        }
        "#;
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);

        // Act
        let (_, errors) = parser.parse();

        // Assert
        // We might get cascading errors due to imperfect recovery, which is acceptable
        assert!(!errors.is_empty());

        // At least one should be the specific error
        let found = errors.iter().any(|e| match e {
            ParseError::InvalidSyntax(_, msg) => {
                msg.contains("Curve factor must be at most 3 characters")
            }
            _ => false,
        });
        assert!(found);
    }

    #[test]
    fn test_parse_multiple_actions() {
        let input = r#"
        players = { p1, p2 }
        state = { position = { p1 = (0, 0), p2 = (10, 10) } }
        actions = [
            action = { move = { p1 -> (10, 0) } },
            action = { move = { p1 -> (10, 10) } }
        ]
        "#;
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (playbook, _) = parser.parse();

        assert_eq!(playbook.actions.len(), 2);
        assert_eq!(playbook.actions[0].moves[0].target, (10.0, 0.0));
        assert_eq!(playbook.actions[1].moves[0].target, (10.0, 10.0));
    }

    #[test]
    fn test_parse_error_too_many_actions() {
        // Arrange
        let input = r#"
        players = { p1, p2 }
        state = { position = { p1 = (0, 0), p2 = (10, 10) } }
        actions = [
            action = { move = { p1 -> (10, 0) } },
            action = { move = { p1 -> (10, 10) } },
            action = { move = { p2 -> (5, 5) } },
            action = { move = { p2 -> (0, 0) } }
        ]
        "#;
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);

        // Act
        let (_, errors) = parser.parse();

        // Assert
        assert!(!errors.is_empty());
        let found = errors.iter().any(|e| match e {
            ParseError::InvalidSyntax(_, msg) => msg.contains("Maximum of 3 actions allowed"),
            _ => false,
        });
        assert!(found);
    }
}
