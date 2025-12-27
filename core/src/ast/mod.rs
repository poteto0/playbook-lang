use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone)]
pub struct Playbook {
    pub players: Vec<String>,
    pub state: State,
    pub action: Action,
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct State {
    pub baller: Option<String>,
    pub positions: HashMap<String, (f64, f64)>,
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Action {
    pub moves: Vec<MoveAction>,
    pub screens: Vec<ScreenAction>,
    pub passes: Vec<PassAction>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct MoveAction {
    pub player: String,
    pub target: (f64, f64),
}

#[derive(Debug, PartialEq, Clone)]
pub struct ScreenAction {
    pub player: String,
    pub target: String,
    pub timing: Timing,
}

#[derive(Debug, PartialEq, Clone)]
pub struct PassAction {
    pub from: String,
    pub to: String,
    pub timing: Timing,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Timing {
    Before,
    After,
    None, // Default if not specified
}
