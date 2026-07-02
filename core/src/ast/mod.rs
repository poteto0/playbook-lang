use crate::lexer::Span;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone)]
pub struct Playbook {
    pub players: Vec<String>,
    pub defenders: Vec<String>,
    pub state: State,
    pub actions: Vec<Action>,
    pub comments: Vec<(Span, String)>,
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct State {
    pub baller: Option<String>,
    pub positions: HashMap<String, (f64, f64)>,
    pub defense: HashMap<String, (DefenseTarget, Span)>,
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Action {
    pub moves: Vec<MoveAction>,
    pub screens: Vec<ScreenAction>,
    pub passes: Vec<PassAction>,
    pub defenses: Vec<DefenseAction>,
}

/// Where a defender is assigned to: a fixed court coordinate, or marking
/// (tracking) a player, offset towards the center of the court by `offset`.
/// A marked player is tracked at the position given by `timing` within the
/// action phase (its start, end, or midpoint).
#[derive(Debug, PartialEq, Clone)]
pub enum DefenseTarget {
    Position(f64, f64),
    Mark {
        player: String,
        offset: f64,
        timing: Timing,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub struct DefenseAction {
    pub defender: String,
    pub target: DefenseTarget,
    pub span: Span,
}

#[derive(Debug, PartialEq, Clone)]
pub struct MoveAction {
    pub player: String,
    pub target: (f64, f64),
    pub path_type: PathType,
    pub span: Span,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ScreenTarget {
    Player(String),
    Coordinate(f64, f64),
}

#[derive(Debug, PartialEq, Clone)]
pub struct ScreenAction {
    pub player: String,
    pub target: ScreenTarget,
    pub timing: Timing,
    pub path_type: PathType,
    pub span: Span,
}

#[derive(Debug, PartialEq, Clone)]
pub struct PassAction {
    pub from: String,
    pub to: String,
    pub timing: Timing,
    pub span: Span,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Timing {
    Before,
    After,
    Middle,
    None, // Default if not specified
}

#[derive(Debug, PartialEq, Clone)]
pub enum PathType {
    Straight,
    Curve(CurveDirection),
}

#[derive(Debug, PartialEq, Clone)]
pub enum CurveDirection {
    Left(f64),
    Right(f64),
}
