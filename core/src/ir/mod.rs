pub mod generator;
use crate::ast::CurveDirection;
use crate::lexer::Span;
pub use generator::IRGenerator;
use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub enum IRError {
    UnexpectedPlayer(Span, String),
    PlayerNotBaller(Span, String),
}

impl fmt::Display for IRError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            IRError::UnexpectedPlayer(span, name) => {
                write!(
                    f,
                    "Player '{}' not found at line {}, column {}",
                    name, span.line, span.column
                )
            }
            IRError::PlayerNotBaller(span, name) => {
                write!(
                    f,
                    "Player '{}' does not have the ball at line {}, column {}",
                    name, span.line, span.column
                )
            }
        }
    }
}

use serde::Serialize;

#[derive(Debug, PartialEq, Clone)]
pub struct Scene {
    pub entities: Vec<Entity>,
    pub interactions: Vec<Interaction>,
    pub final_baller: Option<String>,
}

#[derive(Debug, PartialEq, Clone, Serialize)]
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
    pub curve: Option<CurveDirection>,
    pub is_dribble: bool,
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
    pub curve: Option<CurveDirection>,
}

pub fn is_baller(players: &Vec<Entity>, player_id: &String) -> bool {
    for player in players {
        if player.id == *player_id {
            return player.is_baller;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_baller() {
        let players = vec![
            Entity {
                id: "p1".to_string(),
                label: "1".to_string(),
                start_pos: (0.0, 0.0),
                end_pos: (10.0, 10.0),
                is_baller: true,
            },
            Entity {
                id: "p2".to_string(),
                label: "2".to_string(),
                start_pos: (0.0, 0.0),
                end_pos: (10.0, 10.0),
                is_baller: true,
            },
        ];

        assert!(is_baller(&players, &"p1".to_string()));
        assert!(is_baller(&players, &"p2".to_string()));
        assert!(!is_baller(&players, &"p3".to_string()));
    }
}
