use crate::ast::*;
use crate::constants::{
    DEFAULT_BEZIER_CURVE_FACTOR, DEFAULT_DEFENSE_OFFSET, MAX_ACTIONS_PER_PHASE,
};
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

/// Controls how `Parser::parse_braced_block` recovers from a missing `=` or
/// `{` in a block header. The five original hand-written copies of this
/// scaffold split into two clusters here (see the pinning tests in
/// `tests::test_recovery_*`):
#[derive(Clone, Copy)]
struct BracedBlockRecovery {
    /// Call `recover_until(&[Comma, RBrace])` immediately after a `=` or `{`
    /// parse failure. The move/screen/pass loops leave this `false`; the
    /// `position` loop and `parse_defense_block` set it `true`.
    recover_on_header_error: bool,
    /// If `true`, always attempt `{` (even if `=` failed) and always enter
    /// the entry loop and attempt the closing `}` (even if the header
    /// failed). Only `parse_defense_block` sets this `true`; the
    /// move/screen/pass/position loops leave it `false`, which
    /// short-circuits the body entirely on any header failure.
    always_parse_body: bool,
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

    fn peek_span(&self) -> Span {
        self.tokens
            .get(self.pos)
            .map(|token| token.span)
            .unwrap_or(Span {
                start: 0,
                end: 0,
                line: 0,
                column: 0,
            })
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

    /// Parses a `<keyword> = { entry, entry, ... }` brace-delimited block:
    /// consume the section keyword, consume `=`, consume `{`, loop calling
    /// `parse_entry` for each comma-separated entry until `}`/EOF, then
    /// consume `}`. Shared by the move/screen/pass loops in
    /// `parse_action_block`, the `position` loop in `parse_state_block`,
    /// and `parse_defense_block`.
    ///
    /// `parse_entry` is responsible for its own error recording (and, if it
    /// wants per-entry recovery, calling `recover_until` itself) -- this
    /// helper does not add any recovery around individual entries, so each
    /// call site's existing per-entry behavior is preserved verbatim.
    ///
    /// `recovery` controls the two ways the five original copies differ in
    /// *header* (`=`/`{`) recovery; see `BracedBlockRecovery` for what each
    /// flag preserves.
    fn parse_braced_block<F>(&mut self, recovery: BracedBlockRecovery, mut parse_entry: F)
    where
        F: FnMut(&mut Self),
    {
        self.advance(); // consume section keyword

        let eq_ok = match self.expect_and_advance(TokenKind::Equals) {
            Ok(()) => true,
            Err(e) => {
                self.error(e);
                if recovery.recover_on_header_error {
                    self.recover_until(&[TokenKind::Comma, TokenKind::RBrace]);
                }
                false
            }
        };

        let brace_ok = if recovery.always_parse_body || eq_ok {
            match self.expect_and_advance(TokenKind::LBrace) {
                Ok(()) => true,
                Err(e) => {
                    self.error(e);
                    if recovery.recover_on_header_error {
                        self.recover_until(&[TokenKind::Comma, TokenKind::RBrace]);
                    }
                    false
                }
            }
        } else {
            false
        };

        if recovery.always_parse_body || (eq_ok && brace_ok) {
            while self.peek().kind != TokenKind::RBrace && self.peek().kind != TokenKind::EOF {
                parse_entry(self);
                self.consume_if(TokenKind::Comma);
            }
            if let Err(e) = self.expect_and_advance(TokenKind::RBrace) {
                self.error(e);
            }
        }
    }

    /// Parses an optional `: before` / `: after` (/ `: middle`) timing
    /// suffix. Returns `Timing::None` without consuming anything if the
    /// next token is not `:`. Otherwise consumes the `:` and the timing
    /// keyword, accepting `before`/`after` always and `middle` only when
    /// `allow_middle`; anything else records an "Expected timing" error
    /// (without advancing past the offending token) and yields
    /// `Timing::None`.
    fn parse_optional_timing(&mut self, allow_middle: bool) -> Timing {
        if !self.consume_if(TokenKind::Colon) {
            return Timing::None;
        }
        match self.peek().kind {
            TokenKind::Before => {
                self.advance();
                Timing::Before
            }
            TokenKind::After => {
                self.advance();
                Timing::After
            }
            TokenKind::Middle if allow_middle => {
                self.advance();
                Timing::Middle
            }
            _ => {
                self.error(ParseError::UnexpectedToken(
                    self.peek(),
                    "Expected timing".to_string(),
                ));
                Timing::None
            }
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
        let mut defenders = Vec::new();
        let mut state = State::default();
        let mut actions = Vec::new();

        while self.peek().kind != TokenKind::EOF {
            match self.peek().kind {
                TokenKind::Comment(ref s) => {
                    let token = self.advance();
                    self.comments.push((token.span, s.clone()));
                }
                TokenKind::Players => {
                    self.parse_identifier_list_section(&mut players);
                }
                TokenKind::Defenders => {
                    self.parse_identifier_list_section(&mut defenders);
                }
                TokenKind::State => {
                    self.parse_state_section(&mut state);
                }
                TokenKind::Action => {
                    actions.push(self.parse_single_action_section());
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
                        "Expected section start (players, defenders, state, action, actions), but found '{}'",
                        token.clone().kind
                    );
                    let TokenKind::Identifier(ref s) = token.kind else {
                        self.error(ParseError::UnexpectedToken(token, msg));
                        self.advance();
                        continue;
                    };
                    if let Some(sugg) =
                        get_suggestion(s, &["players", "defenders", "state", "action", "actions"])
                    {
                        msg = format!("Expected section start. Did you mean '{}'?", sugg);
                    }
                    self.error(ParseError::UnexpectedToken(token, msg));
                    self.advance();
                }
            }
        }

        let mut colliding_ids: Vec<&String> =
            players.iter().filter(|p| defenders.contains(p)).collect();
        if !colliding_ids.is_empty() {
            colliding_ids.sort();
            let ids = colliding_ids
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            self.error(ParseError::InvalidSyntax(
                self.peek(),
                format!(
                    "Identifier(s) used in both 'players' and 'defenders': {}",
                    ids
                ),
            ));
        }

        (
            Playbook {
                players,
                defenders,
                state,
                actions,
                comments: self.comments.clone(),
            },
            self.errors.clone(),
        )
    }

    /// Parses a `<keyword> = { id, id, ... }` section (used for `players` and `defenders`).
    fn parse_identifier_list_section(&mut self, ids: &mut Vec<String>) {
        self.advance(); // consume section keyword
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
                Ok(p) => ids.push(p),
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

    fn parse_single_action_section(&mut self) -> Action {
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
        action
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

            actions.push(self.parse_single_action_section());

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
                    self.parse_braced_block(
                        BracedBlockRecovery {
                            recover_on_header_error: true,
                            always_parse_body: false,
                        },
                        |p| match p.expect_identifier() {
                            Ok(player) => {
                                if let Err(e) = p.expect_and_advance(TokenKind::Equals) {
                                    p.error(e);
                                } else {
                                    match p.parse_coordinate() {
                                        Ok(coord) => {
                                            state.positions.insert(player, coord);
                                        }
                                        Err(e) => p.error(e),
                                    }
                                }
                            }
                            Err(e) => p.error(e),
                        },
                    );
                    self.consume_if(TokenKind::Comma);
                }
                TokenKind::Defense => {
                    for (defender, target, span) in self.parse_defense_block(false) {
                        state.defense.insert(defender, (target, span));
                    }
                    self.consume_if(TokenKind::Comma);
                }
                _ => {
                    let token = self.peek();
                    let mut msg = "Expected state property (baller, position, defense)".to_string();
                    let TokenKind::Identifier(ref s) = token.kind else {
                        self.error(ParseError::UnexpectedToken(token, msg));
                        self.recover_until(&[TokenKind::Comma, TokenKind::RBrace]);
                        self.consume_if(TokenKind::Comma);
                        continue;
                    };
                    if let Some(sugg) = get_suggestion(s, &["baller", "position", "defense"]) {
                        msg = format!("Expected state property. Did you mean '{}'?", sugg);
                    }
                    self.error(ParseError::UnexpectedToken(token, msg));
                    self.recover_until(&[TokenKind::Comma, TokenKind::RBrace]);
                    self.consume_if(TokenKind::Comma);
                }
            }
        }
    }

    /// Consumes a defense mark arrow (`->` or `-[N]>`), returning the offset
    /// distance to use (the default when a plain arrow is used).
    fn expect_defense_mark_arrow(&mut self) -> Result<f64, ParseError> {
        let token = self.peek();
        match token.kind {
            TokenKind::Arrow => {
                self.advance();
                Ok(DEFAULT_DEFENSE_OFFSET)
            }
            TokenKind::OffsetArrow(ref s) => {
                self.advance();
                let value = s.parse::<f64>().map_err(|_| {
                    ParseError::InvalidSyntax(
                        token.clone(),
                        format!("Invalid defense offset: {}", s),
                    )
                })?;
                if !value.is_finite() {
                    return Err(ParseError::InvalidSyntax(
                        token.clone(),
                        format!("Defense offset must be a finite number: {}", s),
                    ));
                }
                Ok(value)
            }
            TokenKind::Error(ref msg) => {
                Err(ParseError::UnexpectedToken(token.clone(), msg.clone()))
            }
            unexpected => Err(ParseError::UnexpectedToken(
                self.peek().clone(),
                format!("Expected '->' or '-[N]>', but found '{}'", unexpected),
            )),
        }
    }

    /// Parses a single `defense` block entry, shared between `state.defense`
    /// and `action.defense`: `d = (x, y)` sets a fixed position, while
    /// `d -> p` / `d -[N]> p` marks (tracks) a player. A `:before` /
    /// `:middle` / `:after` timing suffix on the marked player is only
    /// meaningful while the action plays out, so it is accepted only when
    /// `allow_timing` (action.defense) and rejected in state.defense.
    fn parse_defense_entry(
        &mut self,
        allow_timing: bool,
    ) -> Result<(String, DefenseTarget, Span), ParseError> {
        let span = self.peek_span();
        let defender = self.expect_identifier()?;
        match self.peek().kind {
            TokenKind::Equals => {
                self.advance();
                let (x, y) = self.parse_coordinate()?;
                Ok((defender, DefenseTarget::Position(x, y), span))
            }
            TokenKind::Arrow | TokenKind::OffsetArrow(_) => {
                let is_explicit_offset = matches!(self.peek().kind, TokenKind::OffsetArrow(_));
                let arrow_token = self.peek();
                let offset = self.expect_defense_mark_arrow()?;
                if self.peek().kind == TokenKind::LParenthesis {
                    // Consume the coordinate even when rejecting, so a bad
                    // explicit offset doesn't leave dangling tokens for the
                    // caller's error recovery to stumble over.
                    let (x, y) = self.parse_coordinate()?;
                    if is_explicit_offset {
                        return Err(ParseError::InvalidSyntax(
                            arrow_token,
                            format!(
                                "Explicit offset '-[{}]>' has no effect before a fixed position; use '->' instead",
                                offset
                            ),
                        ));
                    }
                    Ok((defender, DefenseTarget::Position(x, y), span))
                } else {
                    let player = self.expect_identifier()?;
                    // Parse (and consume) any timing suffix even when it is
                    // not allowed, so it doesn't split the entry during the
                    // caller's error recovery; reject it afterwards.
                    let colon_token = self.peek();
                    let mut timing = self.parse_optional_timing(true);
                    if !allow_timing && timing != Timing::None {
                        self.error(ParseError::InvalidSyntax(
                            colon_token,
                            "Timing suffix is not allowed in state.defense; timing only applies to action.defense marks".to_string(),
                        ));
                        timing = Timing::None;
                    }
                    Ok((
                        defender,
                        DefenseTarget::Mark {
                            player,
                            offset,
                            timing,
                        },
                        span,
                    ))
                }
            }
            TokenKind::Error(ref msg) => Err(ParseError::UnexpectedToken(self.peek(), msg.clone())),
            unexpected => Err(ParseError::UnexpectedToken(
                self.peek(),
                format!("Expected '=', '->' or '-[N]>', but found '{}'", unexpected),
            )),
        }
    }

    /// Parses a `defense = { ... }` block, shared between `state.defense`
    /// (folded by the caller into a `HashMap`) and `action.defense` (folded
    /// into a `Vec` that preserves order and spans).
    fn parse_defense_block(&mut self, allow_timing: bool) -> Vec<(String, DefenseTarget, Span)> {
        let mut entries = Vec::new();
        self.parse_braced_block(
            BracedBlockRecovery {
                recover_on_header_error: true,
                always_parse_body: true,
            },
            |p| match p.parse_defense_entry(allow_timing) {
                Ok(entry) => entries.push(entry),
                Err(e) => {
                    p.error(e);
                    p.recover_until(&[TokenKind::Comma, TokenKind::RBrace]);
                }
            },
        );
        entries
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
                    let value = factor_str.parse::<f64>().map_err(|_| {
                        ParseError::InvalidSyntax(
                            token.clone(),
                            format!("Invalid curve factor: {}", factor_str),
                        )
                    })?;
                    if !value.is_finite() {
                        return Err(ParseError::InvalidSyntax(
                            token.clone(),
                            format!("Curve factor must be a finite number: {}", factor_str),
                        ));
                    }
                    value
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
                    self.parse_braced_block(
                        BracedBlockRecovery {
                            recover_on_header_error: false,
                            always_parse_body: false,
                        },
                        |p| {
                            // Try parsing a move line: player -> target
                            let span = p.peek_span();
                            match p.expect_identifier() {
                                Ok(player) => match p.expect_arrow() {
                                    Ok(path_type) => match p.parse_coordinate() {
                                        Ok(target) => {
                                            action.moves.push(MoveAction {
                                                player,
                                                target,
                                                path_type,
                                                span,
                                            });
                                        }
                                        Err(e) => p.error(e),
                                    },
                                    Err(e) => p.error(e),
                                },
                                Err(e) => p.error(e),
                            }
                        },
                    );
                    self.consume_if(TokenKind::Comma);
                }
                TokenKind::Screen => {
                    self.parse_braced_block(
                        BracedBlockRecovery {
                            recover_on_header_error: false,
                            always_parse_body: false,
                        },
                        |p| {
                            let span = p.peek_span();
                            match p.expect_identifier() {
                                Ok(player) => match p.expect_arrow() {
                                    Ok(path_type) => {
                                        let target_res = if p.peek().kind == TokenKind::LParenthesis
                                        {
                                            p.parse_coordinate()
                                                .map(|(x, y)| ScreenTarget::Coordinate(x, y))
                                        } else {
                                            p.expect_identifier().map(ScreenTarget::Player)
                                        };

                                        match target_res {
                                            Ok(target) => {
                                                let timing = p.parse_optional_timing(true);
                                                action.screens.push(ScreenAction {
                                                    player,
                                                    target,
                                                    timing,
                                                    path_type,
                                                    span,
                                                });
                                            }
                                            Err(e) => p.error(e),
                                        }
                                    }
                                    Err(e) => p.error(e),
                                },
                                Err(e) => p.error(e),
                            }
                        },
                    );
                    self.consume_if(TokenKind::Comma);
                }
                TokenKind::Pass => {
                    self.parse_braced_block(
                        BracedBlockRecovery {
                            recover_on_header_error: false,
                            always_parse_body: false,
                        },
                        |p| {
                            let span = p.peek_span();
                            match p.expect_identifier() {
                                Ok(from) => {
                                    if let Err(e) = p.expect_and_advance(TokenKind::Arrow) {
                                        p.error(e);
                                    } else {
                                        match p.expect_identifier() {
                                            Ok(to) => {
                                                let timing = p.parse_optional_timing(false);
                                                action.passes.push(PassAction {
                                                    from,
                                                    to,
                                                    timing,
                                                    span,
                                                });
                                            }
                                            Err(e) => p.error(e),
                                        }
                                    }
                                }
                                Err(e) => p.error(e),
                            }
                        },
                    );
                    self.consume_if(TokenKind::Comma);
                }
                TokenKind::Defense => {
                    for (defender, target, span) in self.parse_defense_block(true) {
                        action.defenses.push(DefenseAction {
                            defender,
                            target,
                            span,
                        });
                    }
                    self.consume_if(TokenKind::Comma);
                }
                _ => {
                    let token = self.peek();
                    let mut msg =
                        "Expected action property (move, screen, pass, defense)".to_string();
                    let TokenKind::Identifier(ref s) = token.kind else {
                        self.error(ParseError::UnexpectedToken(token, msg));
                        self.recover_until(&[TokenKind::Comma, TokenKind::RBrace]);
                        self.consume_if(TokenKind::Comma);
                        continue;
                    };
                    if let Some(sugg) = get_suggestion(s, &["move", "screen", "pass", "defense"]) {
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
    fn test_parse_curve_factor_multi_digit_and_negative() {
        // Values longer than 3 characters (0.25, -0.5) used to be rejected by an
        // arbitrary byte-length check. They are valid coefficients and must parse.
        let input = r#"
        players = { p1 }
        state = { }
        action = {
            move = {
                p1 ~[l:0.25]> (10, 10),
                p1 ~[r:-0.5]> (20, 20)
            },
        }
        "#;
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (playbook, errors) = parser.parse();

        assert!(errors.is_empty(), "unexpected errors: {:?}", errors);

        match playbook.actions[0].moves[0].path_type {
            PathType::Curve(CurveDirection::Left(f)) => assert_eq!(f, 0.25),
            _ => panic!("Expected Left Curve with 0.25"),
        }
        match playbook.actions[0].moves[1].path_type {
            PathType::Curve(CurveDirection::Right(f)) => assert_eq!(f, -0.5),
            _ => panic!("Expected Right Curve with -0.5"),
        }
    }

    #[test]
    fn test_parse_invalid_curve_factor() {
        let input = r#"
        players = { p1 }
        state = { }
        action = {
            move = {
                p1 ~[l:abc]> (10, 10)
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
            ParseError::InvalidSyntax(_, msg) => msg.contains("Invalid curve factor"),
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

    #[test]
    fn test_action_span_points_to_player_token() {
        // The player identifier starts at line 4, column 1.
        // Before the fix, the span pointed at the trailing comma instead.
        let input = "players = { p1 }\nstate = { position = { p1 = (0, 0) } }\nactions = [ action = { move = {\np1 -> (5, 5),\n} } ]";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (playbook, _) = parser.parse();

        let span = playbook.actions[0].moves[0].span;
        assert_eq!(span.line, 4);
        assert_eq!(span.column, 1);
    }

    #[test]
    fn test_parse_defenders_section() {
        let input = "defenders = { d1, d2 }";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (playbook, errors) = parser.parse();
        assert!(errors.is_empty());
        assert_eq!(playbook.defenders, vec!["d1", "d2"]);
    }

    #[test]
    fn test_parse_state_defense_position_and_mark() {
        let input = r#"
        players = { p1 }
        defenders = { d1, d2, d3 }
        state = {
            position = { p1 = (0, 0) },
            defense = {
                d1 -> p1,
                d2 = (-90, -80),
                d3 -[7]> p1,
            },
        }
        "#;
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (playbook, errors) = parser.parse();
        assert!(errors.is_empty(), "unexpected errors: {:?}", errors);

        match playbook.state.defense.get("d1") {
            Some((DefenseTarget::Mark { player, offset, .. }, _)) => {
                assert_eq!(player, "p1");
                assert_eq!(*offset, 20.0);
            }
            other => panic!("Expected default Mark for d1, got {:?}", other),
        }
        match playbook.state.defense.get("d2") {
            Some((DefenseTarget::Position(x, y), _)) => assert_eq!((*x, *y), (-90.0, -80.0)),
            other => panic!("Expected Position for d2, got {:?}", other),
        }
        match playbook.state.defense.get("d3") {
            Some((DefenseTarget::Mark { player, offset, .. }, _)) => {
                assert_eq!(player, "p1");
                assert_eq!(*offset, 7.0);
            }
            other => panic!("Expected Mark with offset 7 for d3, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_action_defense_move_and_mark() {
        let input = r#"
        players = { p1 }
        defenders = { d1, d2 }
        state = { position = { p1 = (0, 0) } }
        action = {
            defense = {
                d1 -> (70, 20),
                d2 -> p1,
            },
        }
        "#;
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (playbook, errors) = parser.parse();
        assert!(errors.is_empty(), "unexpected errors: {:?}", errors);

        assert_eq!(playbook.actions[0].defenses.len(), 2);
        assert_eq!(playbook.actions[0].defenses[0].defender, "d1");
        match &playbook.actions[0].defenses[0].target {
            DefenseTarget::Position(x, y) => assert_eq!((*x, *y), (70.0, 20.0)),
            other => panic!("Expected Position for d1, got {:?}", other),
        }
        assert_eq!(playbook.actions[0].defenses[1].defender, "d2");
        match &playbook.actions[0].defenses[1].target {
            DefenseTarget::Mark { player, offset, .. } => {
                assert_eq!(player, "p1");
                assert_eq!(*offset, 20.0);
            }
            other => panic!("Expected Mark for d2, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_action_defense_mark_timing() {
        let input = r#"
        players = { p1, p2, p3 }
        defenders = { d1, d2, d3 }
        state = { position = { p1 = (0, 0), p2 = (0, 0), p3 = (0, 0) } }
        action = {
            defense = {
                d1 -> p1:before,
                d2 -> p2:after,
                d3 -> p3:middle,
            },
        }
        "#;
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (playbook, errors) = parser.parse();
        assert!(errors.is_empty(), "unexpected errors: {:?}", errors);

        let timing_of = |defender: &str| {
            playbook.actions[0]
                .defenses
                .iter()
                .find(|d| d.defender == defender)
                .map(|d| match &d.target {
                    DefenseTarget::Mark { timing, .. } => timing.clone(),
                    other => panic!("Expected Mark for {}, got {:?}", defender, other),
                })
                .unwrap()
        };
        assert_eq!(timing_of("d1"), Timing::Before);
        assert_eq!(timing_of("d2"), Timing::After);
        assert_eq!(timing_of("d3"), Timing::Middle);
    }

    #[test]
    fn test_parse_state_defense_mark_timing_is_rejected() {
        let input = r#"
        players = { p1, p2 }
        defenders = { d1, d2 }
        state = {
            position = { p1 = (0, 0), p2 = (0, 0) },
            defense = { d1 -> p1:middle, d2 -> p2 },
        }
        "#;
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (playbook, errors) = parser.parse();

        let found = errors.iter().any(|e| match e {
            ParseError::InvalidSyntax(_, msg) => msg.contains("not allowed in state.defense"),
            _ => false,
        });
        assert!(found, "expected InvalidSyntax error, got {:?}", errors);

        // Both marks survive; the rejected timing falls back to None.
        for defender in ["d1", "d2"] {
            match &playbook.state.defense.get(defender).unwrap().0 {
                DefenseTarget::Mark { timing, .. } => assert_eq!(*timing, Timing::None),
                other => panic!("Expected Mark for {}, got {:?}", defender, other),
            }
        }
    }

    #[test]
    fn test_parse_action_defense_mark_invalid_timing_keeps_mark() {
        let input = r#"
        players = { p1, p2 }
        defenders = { d1, d2 }
        state = { position = { p1 = (0, 0), p2 = (0, 0) } }
        action = {
            defense = { d1 -> p1:oops, d2 -> p2 },
        }
        "#;
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (playbook, errors) = parser.parse();

        let found = errors.iter().any(|e| match e {
            ParseError::UnexpectedToken(_, msg) => msg.contains("Expected timing"),
            _ => false,
        });
        assert!(found, "expected 'Expected timing' error, got {:?}", errors);

        // Like screens, an invalid timing keeps the parsed mark with
        // Timing::None instead of discarding the whole entry.
        let defenses = &playbook.actions[0].defenses;
        for defender in ["d1", "d2"] {
            let target = &defenses
                .iter()
                .find(|d| d.defender == defender)
                .unwrap_or_else(|| panic!("mark for {} was discarded", defender))
                .target;
            match target {
                DefenseTarget::Mark { timing, .. } => assert_eq!(*timing, Timing::None),
                other => panic!("Expected Mark for {}, got {:?}", defender, other),
            }
        }
    }

    #[test]
    fn test_parse_invalid_defense_offset() {
        let input = r#"
        players = { p1 }
        defenders = { d1 }
        state = {
            defense = { d1 -[abc]> p1 },
        }
        "#;
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (_, errors) = parser.parse();

        let found = errors.iter().any(|e| match e {
            ParseError::InvalidSyntax(_, msg) => msg.contains("Invalid defense offset"),
            _ => false,
        });
        assert!(found, "expected InvalidSyntax error, got {:?}", errors);
    }

    #[test]
    fn test_parse_explicit_offset_before_coordinate_is_rejected() {
        let input = r#"
        defenders = { d1 }
        state = {
            defense = { d1 -[15]> (70, 20) },
        }
        "#;
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (_, errors) = parser.parse();

        let found = errors.iter().any(|e| match e {
            ParseError::InvalidSyntax(_, msg) => msg.contains("has no effect"),
            _ => false,
        });
        assert!(found, "expected InvalidSyntax error, got {:?}", errors);
    }

    #[test]
    fn test_parse_players_and_defenders_ids_must_be_disjoint() {
        let input = "players = { p1 }\ndefenders = { p1 }";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (_, errors) = parser.parse();

        let found = errors.iter().any(|e| match e {
            ParseError::InvalidSyntax(_, msg) => {
                msg.contains("both 'players' and 'defenders'") && msg.contains("p1")
            }
            _ => false,
        });
        assert!(found, "expected InvalidSyntax error, got {:?}", errors);
    }

    // ------------------------------------------------------------------
    // Braced-block error-recovery pinning tests.
    //
    // These pin the *current* recovery behavior of the five "consume `=`,
    // consume `{`, loop entries with comma-separated recovery, consume `}`"
    // copies in this file (move/screen/pass loops in `parse_action_block`,
    // the `position` loop in `parse_state_block`, and `parse_defense_block`,
    // shared by `state.defense` and `action.defense`) before they are
    // unified behind a shared `parse_braced_block` helper. Exploration
    // (see PR follow-up notes) found the five copies fall into three
    // distinct behavior clusters:
    //
    // Cluster A (move, screen, pass): on a missing `=` or `{`, the error is
    // recorded but `recover_until` is NOT called, and the entry loop /
    // closing-brace check are skipped entirely (short-circuited), leaving
    // leftover tokens for the *caller's* own recovery loop to reinterpret
    // (visible as extra cascading errors below).
    //
    // Cluster B (state.position): same short-circuiting as cluster A, but
    // DOES call `recover_until(&[Comma, RBrace])` after a missing `=` or
    // `{`.
    //
    // Cluster C (parse_defense_block, shared by state.defense and
    // action.defense): calls `recover_until` after a missing `=` or `{`
    // (like B), but does NOT short-circuit -- it always attempts `{`
    // (even if `=` failed) and always enters the entry loop and attempts
    // the closing `}` (even if the header failed).
    //
    // Per-entry recovery differs too: move/screen/pass/position entries
    // record an error and move on (no recover_until), while
    // parse_defense_block's entries call recover_until on failure.
    //
    // The refactor's `parse_braced_block` helper is parameterized
    // (`recover_on_header_error`, `always_parse_body`) precisely to
    // preserve these three clusters rather than silently harmonizing them.
    // ------------------------------------------------------------------

    #[test]
    fn test_recovery_move_missing_equals() {
        let input = "players = { p1, p2 }\nstate = { position = { p1 = (0,0), p2 = (1,1) } }\naction = { move { p1 -> (1,1) } }";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (playbook, errors) = parser.parse();
        // Cluster A: header failure short-circuits the entry loop entirely,
        // and leftover tokens (`{ p1 -> (1,1) } }`) cascade into further
        // "expected action property" / "expected section start" errors.
        assert_eq!(errors.len(), 4, "errors: {:?}", errors);
        assert!(
            matches!(&errors[0], ParseError::UnexpectedToken(_, msg) if msg.contains("Expected `=`"))
        );
        assert_eq!(playbook.actions[0].moves.len(), 0);
    }

    #[test]
    fn test_recovery_move_missing_lbrace() {
        let input = "players = { p1, p2 }\nstate = { position = { p1 = (0,0), p2 = (1,1) } }\naction = { move = p1 -> (1,1) } }";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (playbook, errors) = parser.parse();
        assert_eq!(errors.len(), 4, "errors: {:?}", errors);
        assert!(
            matches!(&errors[0], ParseError::UnexpectedToken(_, msg) if msg.contains("Expected `{`"))
        );
        assert_eq!(playbook.actions[0].moves.len(), 0);
    }

    #[test]
    fn test_recovery_move_malformed_entry_mid_block() {
        let input = "players = { p1, p2 }\nstate = { position = { p1 = (0,0), p2 = (1,1) } }\naction = { move = { p1 -> (1,1), ^^^, p2 -> (2,2) } }";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (playbook, errors) = parser.parse();
        // No recover_until on entry failure: each bad `^` char is its own
        // Error token / its own failed `expect_identifier`, producing one
        // error apiece, but both well-formed entries around it still parse.
        assert_eq!(errors.len(), 3, "errors: {:?}", errors);
        assert_eq!(playbook.actions[0].moves.len(), 2);
    }

    #[test]
    fn test_recovery_move_trailing_comma() {
        let input = "players = { p1 }\nstate = { position = { p1 = (0,0) } }\naction = { move = { p1 -> (1,1), } }";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (playbook, errors) = parser.parse();
        assert!(errors.is_empty(), "errors: {:?}", errors);
        assert_eq!(playbook.actions[0].moves.len(), 1);
    }

    #[test]
    fn test_recovery_move_unexpected_eof() {
        let input = "players = { p1 }\nstate = { position = { p1 = (0,0) } }\naction = { move = { p1 -> (1,1)";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (playbook, errors) = parser.parse();
        // One "Expected `}`, found EOF" from the move block's own closing
        // brace, and a second from the enclosing action block's.
        assert_eq!(errors.len(), 2, "errors: {:?}", errors);
        assert!(errors.iter().all(
            |e| matches!(e, ParseError::UnexpectedToken(_, msg) if msg.contains("Expected `}`"))
        ));
        assert_eq!(playbook.actions[0].moves.len(), 1);
    }

    #[test]
    fn test_recovery_screen_missing_equals() {
        let input = "players = { p1, p2 }\nstate = { position = { p1 = (0,0), p2 = (1,1) } }\naction = { screen { p1 -> p2 } }";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (playbook, errors) = parser.parse();
        assert_eq!(errors.len(), 3, "errors: {:?}", errors);
        assert!(
            matches!(&errors[0], ParseError::UnexpectedToken(_, msg) if msg.contains("Expected `=`"))
        );
        assert_eq!(playbook.actions[0].screens.len(), 0);
    }

    #[test]
    fn test_recovery_screen_missing_lbrace() {
        let input = "players = { p1, p2 }\nstate = { position = { p1 = (0,0), p2 = (1,1) } }\naction = { screen = p1 -> p2 } }";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (playbook, errors) = parser.parse();
        assert_eq!(errors.len(), 3, "errors: {:?}", errors);
        assert!(
            matches!(&errors[0], ParseError::UnexpectedToken(_, msg) if msg.contains("Expected `{`"))
        );
        assert_eq!(playbook.actions[0].screens.len(), 0);
    }

    #[test]
    fn test_recovery_screen_malformed_entry_mid_block() {
        let input = "players = { p1, p2 }\nstate = { position = { p1 = (0,0), p2 = (1,1) } }\naction = { screen = { p1 -> p2, ^^^, p2 -> p1 } }";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (playbook, errors) = parser.parse();
        assert_eq!(errors.len(), 3, "errors: {:?}", errors);
        assert_eq!(playbook.actions[0].screens.len(), 2);
    }

    #[test]
    fn test_recovery_screen_trailing_comma() {
        let input = "players = { p1, p2 }\nstate = { position = { p1 = (0,0), p2 = (1,1) } }\naction = { screen = { p1 -> p2, } }";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (playbook, errors) = parser.parse();
        assert!(errors.is_empty(), "errors: {:?}", errors);
        assert_eq!(playbook.actions[0].screens.len(), 1);
    }

    #[test]
    fn test_recovery_screen_unexpected_eof() {
        let input = "players = { p1, p2 }\nstate = { position = { p1 = (0,0), p2 = (1,1) } }\naction = { screen = { p1 -> p2";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (playbook, errors) = parser.parse();
        assert_eq!(errors.len(), 2, "errors: {:?}", errors);
        assert_eq!(playbook.actions[0].screens.len(), 1);
    }

    #[test]
    fn test_recovery_pass_missing_equals() {
        let input = "players = { p1, p2 }\nstate = { baller = p1, position = { p1 = (0,0), p2 = (1,1) } }\naction = { pass { p1 -> p2 } }";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (playbook, errors) = parser.parse();
        assert_eq!(errors.len(), 3, "errors: {:?}", errors);
        assert!(
            matches!(&errors[0], ParseError::UnexpectedToken(_, msg) if msg.contains("Expected `=`"))
        );
        assert_eq!(playbook.actions[0].passes.len(), 0);
    }

    #[test]
    fn test_recovery_pass_missing_lbrace() {
        let input = "players = { p1, p2 }\nstate = { baller = p1, position = { p1 = (0,0), p2 = (1,1) } }\naction = { pass = p1 -> p2 } }";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (playbook, errors) = parser.parse();
        assert_eq!(errors.len(), 3, "errors: {:?}", errors);
        assert!(
            matches!(&errors[0], ParseError::UnexpectedToken(_, msg) if msg.contains("Expected `{`"))
        );
        assert_eq!(playbook.actions[0].passes.len(), 0);
    }

    #[test]
    fn test_recovery_pass_malformed_entry_mid_block() {
        let input = "players = { p1, p2 }\nstate = { baller = p1, position = { p1 = (0,0), p2 = (1,1) } }\naction = { pass = { p1 -> p2, ^^^, p2 -> p1 } }";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (playbook, errors) = parser.parse();
        assert_eq!(errors.len(), 3, "errors: {:?}", errors);
        assert_eq!(playbook.actions[0].passes.len(), 2);
    }

    #[test]
    fn test_recovery_pass_trailing_comma() {
        let input = "players = { p1, p2 }\nstate = { baller = p1, position = { p1 = (0,0), p2 = (1,1) } }\naction = { pass = { p1 -> p2, } }";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (playbook, errors) = parser.parse();
        assert!(errors.is_empty(), "errors: {:?}", errors);
        assert_eq!(playbook.actions[0].passes.len(), 1);
    }

    #[test]
    fn test_recovery_pass_unexpected_eof() {
        let input = "players = { p1, p2 }\nstate = { baller = p1, position = { p1 = (0,0), p2 = (1,1) } }\naction = { pass = { p1 -> p2";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (playbook, errors) = parser.parse();
        assert_eq!(errors.len(), 2, "errors: {:?}", errors);
        assert_eq!(playbook.actions[0].passes.len(), 1);
    }

    #[test]
    fn test_recovery_position_missing_equals() {
        let input = "players = { p1 }\nstate = { position { p1 = (0,0) } }";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (playbook, errors) = parser.parse();
        // Cluster B: header failure calls recover_until AND short-circuits
        // the entry loop (unlike A, but same short-circuiting), so no
        // positions are parsed and leftover tokens still cascade.
        assert_eq!(errors.len(), 3, "errors: {:?}", errors);
        assert!(
            matches!(&errors[0], ParseError::UnexpectedToken(_, msg) if msg.contains("Expected `=`"))
        );
        assert_eq!(playbook.state.positions.len(), 0);
    }

    #[test]
    fn test_recovery_position_missing_lbrace() {
        let input = "players = { p1 }\nstate = { position = p1 = (0,0) } }";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (playbook, errors) = parser.parse();
        assert_eq!(errors.len(), 3, "errors: {:?}", errors);
        assert!(
            matches!(&errors[0], ParseError::UnexpectedToken(_, msg) if msg.contains("Expected `{`"))
        );
        assert_eq!(playbook.state.positions.len(), 0);
    }

    #[test]
    fn test_recovery_position_malformed_entry_mid_block() {
        let input = "players = { p1, p2 }\nstate = { position = { p1 = (0,0), ^^^, p2 = (1,1) } }";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (playbook, errors) = parser.parse();
        assert_eq!(errors.len(), 3, "errors: {:?}", errors);
        assert_eq!(playbook.state.positions.len(), 2);
    }

    #[test]
    fn test_recovery_position_trailing_comma() {
        let input = "players = { p1 }\nstate = { position = { p1 = (0,0), } }";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (playbook, errors) = parser.parse();
        assert!(errors.is_empty(), "errors: {:?}", errors);
        assert_eq!(playbook.state.positions.len(), 1);
    }

    #[test]
    fn test_recovery_position_unexpected_eof() {
        let input = "players = { p1 }\nstate = { position = { p1 = (0,0)";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (playbook, errors) = parser.parse();
        assert_eq!(errors.len(), 2, "errors: {:?}", errors);
        assert_eq!(playbook.state.positions.len(), 1);
    }

    #[test]
    fn test_recovery_state_defense_missing_equals() {
        let input = "players = { p1 }\ndefenders = { d1 }\nstate = { position = { p1 = (0,0) }, defense { d1 -> p1 } }";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (playbook, errors) = parser.parse();
        // Cluster C (parse_defense_block): unlike A/B, a header failure does
        // NOT short-circuit -- `{` is still attempted (and also fails here,
        // since recover_until left us at `}`), each recovering via
        // recover_until, and no cascade beyond the block itself.
        assert_eq!(errors.len(), 2, "errors: {:?}", errors);
        assert!(
            matches!(&errors[0], ParseError::UnexpectedToken(_, msg) if msg.contains("Expected `=`"))
        );
        assert!(
            matches!(&errors[1], ParseError::UnexpectedToken(_, msg) if msg.contains("Expected `{`"))
        );
        assert_eq!(playbook.state.defense.len(), 0);
    }

    #[test]
    fn test_recovery_state_defense_missing_lbrace() {
        let input = "players = { p1 }\ndefenders = { d1 }\nstate = { position = { p1 = (0,0) }, defense = d1 -> p1 } }";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (playbook, errors) = parser.parse();
        assert_eq!(errors.len(), 1, "errors: {:?}", errors);
        assert!(
            matches!(&errors[0], ParseError::UnexpectedToken(_, msg) if msg.contains("Expected `{`"))
        );
        assert_eq!(playbook.state.defense.len(), 0);
    }

    #[test]
    fn test_recovery_state_defense_malformed_entry_mid_block() {
        let input = "players = { p1 }\ndefenders = { d1, d2 }\nstate = { position = { p1 = (0,0) }, defense = { d1 -> p1, ^^^, d2 -> p1 } }";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (playbook, errors) = parser.parse();
        // Unlike A/B, per-entry failure DOES call recover_until, so only a
        // single error is recorded for the whole run of bad `^` tokens.
        assert_eq!(errors.len(), 1, "errors: {:?}", errors);
        assert_eq!(playbook.state.defense.len(), 2);
    }

    #[test]
    fn test_recovery_state_defense_trailing_comma() {
        let input = "players = { p1 }\ndefenders = { d1 }\nstate = { position = { p1 = (0,0) }, defense = { d1 -> p1, } }";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (playbook, errors) = parser.parse();
        assert!(errors.is_empty(), "errors: {:?}", errors);
        assert_eq!(playbook.state.defense.len(), 1);
    }

    #[test]
    fn test_recovery_state_defense_unexpected_eof() {
        let input = "players = { p1 }\ndefenders = { d1 }\nstate = { position = { p1 = (0,0) }, defense = { d1 -> p1";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (playbook, errors) = parser.parse();
        assert_eq!(errors.len(), 2, "errors: {:?}", errors);
        assert_eq!(playbook.state.defense.len(), 1);
    }

    #[test]
    fn test_recovery_action_defense_missing_equals() {
        let input = "players = { p1 }\ndefenders = { d1 }\nstate = { position = { p1 = (0,0) } }\naction = { defense { d1 -> p1 } }";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (playbook, errors) = parser.parse();
        assert_eq!(errors.len(), 2, "errors: {:?}", errors);
        assert_eq!(playbook.actions[0].defenses.len(), 0);
    }

    #[test]
    fn test_recovery_action_defense_missing_lbrace() {
        let input = "players = { p1 }\ndefenders = { d1 }\nstate = { position = { p1 = (0,0) } }\naction = { defense = d1 -> p1 } }";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (playbook, errors) = parser.parse();
        assert_eq!(errors.len(), 1, "errors: {:?}", errors);
        assert_eq!(playbook.actions[0].defenses.len(), 0);
    }

    #[test]
    fn test_recovery_action_defense_malformed_entry_mid_block() {
        let input = "players = { p1 }\ndefenders = { d1, d2 }\nstate = { position = { p1 = (0,0) } }\naction = { defense = { d1 -> p1, ^^^, d2 -> p1 } }";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (playbook, errors) = parser.parse();
        assert_eq!(errors.len(), 1, "errors: {:?}", errors);
        assert_eq!(playbook.actions[0].defenses.len(), 2);
    }

    #[test]
    fn test_recovery_action_defense_trailing_comma() {
        let input = "players = { p1 }\ndefenders = { d1 }\nstate = { position = { p1 = (0,0) } }\naction = { defense = { d1 -> p1, } }";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (playbook, errors) = parser.parse();
        assert!(errors.is_empty(), "errors: {:?}", errors);
        assert_eq!(playbook.actions[0].defenses.len(), 1);
    }

    #[test]
    fn test_recovery_action_defense_unexpected_eof() {
        let input = "players = { p1 }\ndefenders = { d1 }\nstate = { position = { p1 = (0,0) } }\naction = { defense = { d1 -> p1";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (playbook, errors) = parser.parse();
        assert_eq!(errors.len(), 2, "errors: {:?}", errors);
        assert_eq!(playbook.actions[0].defenses.len(), 1);
    }
}
