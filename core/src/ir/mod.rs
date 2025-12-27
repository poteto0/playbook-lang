pub mod generator;
pub use generator::IRGenerator;

#[derive(Debug, PartialEq, Clone)]
pub struct Scene {
    pub entities: Vec<Entity>,
    pub interactions: Vec<Interaction>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Entity {
    pub id: String,
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
}

#[derive(Debug, PartialEq, Clone)]
pub struct MoveLine {
    pub player_id: String,
    pub from: (f64, f64),
    pub to: (f64, f64),
}

#[derive(Debug, PartialEq, Clone)]
pub struct PassLine {
    pub from: (f64, f64),
    pub to: (f64, f64),
}

#[derive(Debug, PartialEq, Clone)]
pub struct ScreenLine {
    pub screener_id: String,
    pub from: (f64, f64),
    pub to: (f64, f64),
}
