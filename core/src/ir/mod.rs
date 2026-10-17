pub mod generator;
use crate::ast::CurveDirection;
use crate::lexer::Span;
pub use generator::IRGenerator;
use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub enum IRError {
    UnexpectedPlayer(Span, String),
    PlayerNotBaller(Span, String),
    /// A player listed in `players` has no entry in `state.position`. The
    /// span points at the `players` section keyword.
    MissingPosition(Span, String),
    /// A defender listed in `defenders` has no entry in `state.defense`. The
    /// span points at the `defenders` section keyword.
    MissingDefensePosition(Span, String),
    /// The same player (or defender) is given the same action kind more than
    /// once inside one `action` block, e.g. two `move` entries for `p1`. The
    /// span points at the duplicate (second) entry.
    DuplicateAction(Span, &'static str, String),
}

impl IRError {
    /// The span of the source text that caused this error.
    pub fn span(&self) -> &Span {
        match self {
            IRError::UnexpectedPlayer(span, _)
            | IRError::PlayerNotBaller(span, _)
            | IRError::MissingPosition(span, _)
            | IRError::MissingDefensePosition(span, _)
            | IRError::DuplicateAction(span, _, _) => span,
        }
    }

    /// The canonical error message, without any positional suffix.
    ///
    /// This is the single source of truth for the wording; every output
    /// destination (`Display`, `Renderer::format_ir_error`, the linter)
    /// formats this text instead of duplicating it.
    pub fn message(&self) -> String {
        match self {
            IRError::UnexpectedPlayer(_, name) => format!("Player '{}' not found in state", name),
            IRError::PlayerNotBaller(_, name) => {
                format!("Player '{}' does not have the ball", name)
            }
            IRError::MissingPosition(_, name) => {
                format!("Player '{}' has no position in state", name)
            }
            IRError::MissingDefensePosition(_, name) => {
                format!("Defender '{}' has no defense entry in state", name)
            }
            IRError::DuplicateAction(_, kind, name) => {
                format!(
                    "Duplicate action '{}' for '{}' in the same action block",
                    kind, name
                )
            }
        }
    }
}

impl fmt::Display for IRError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let span = self.span();
        write!(
            f,
            "{} (line {}, column {})",
            self.message(),
            span.line,
            span.column
        )
    }
}

use serde::Serialize;

#[derive(Debug, PartialEq, Clone)]
pub struct Scene {
    pub entities: Vec<Entity>,
    pub interactions: Vec<Interaction>,
    pub final_baller: Option<String>,
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize)]
pub enum EntityKind {
    Player,
    Defender,
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct Entity {
    pub id: String,
    pub kind: EntityKind,
    pub label: String,
    pub start_pos: (f64, f64),
    pub end_pos: (f64, f64),
    pub is_baller: bool,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Interaction {
    Move(MoveLine),
    Pass(PassLine),
    Screen(ScreenLine),
    Defense(DefenseLine),
}

#[derive(Debug, PartialEq, Clone)]
pub struct MoveLine {
    pub player_id: String,
    pub from: (f64, f64),
    pub to: (f64, f64),
    pub curve: Option<CurveDirection>,
    pub is_dribble: bool,
}

#[derive(Debug, PartialEq, Clone)]
pub struct PassLine {
    pub from: (f64, f64),
    pub to: (f64, f64),
}

#[derive(Debug, PartialEq, Clone)]
pub struct DefenseLine {
    pub defender_id: String,
    pub from: (f64, f64),
    pub to: (f64, f64),
}

#[derive(Debug, PartialEq, Clone)]
pub struct ScreenLine {
    pub screener_id: String,
    pub from: (f64, f64),
    pub to: (f64, f64),
    pub curve: Option<CurveDirection>,
}
