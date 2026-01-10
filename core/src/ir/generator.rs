use crate::ast::{Playbook, Timing};
use crate::ir::*;

pub struct IRGenerator;

impl IRGenerator {
    pub fn generate(playbook: Playbook) -> Result<Scene, String> {
        let mut entities = Vec::new();
        let mut interactions = Vec::new();

        // 1. Resolve positions
        // Default end_pos to start_pos if no move is specified
        let start_positions = playbook.state.positions.clone();
        let mut end_positions = start_positions.clone();

        for move_action in &playbook.action.moves {
            end_positions.insert(move_action.player.clone(), move_action.target);
        }

        // 2. Create Entities
        for player_id in playbook.players {
            let start_pos = *start_positions.get(&player_id).unwrap_or(&(0.0, 0.0));
            let end_pos = *end_positions.get(&player_id).unwrap_or(&start_pos);
            let is_baller = playbook.state.baller.as_ref() == Some(&player_id);

            entities.push(Entity {
                id: player_id.clone(),
                label: player_id.replace("p", ""), // p1 -> 1
                start_pos,
                end_pos,
                is_baller,
            });
        }

        // 3. Create Interactions
        // Moves
        for move_action in playbook.action.moves {
            let from = *start_positions
                .get(&move_action.player)
                .unwrap_or(&(0.0, 0.0));

            let curve = match move_action.path_type {
                crate::ast::PathType::Straight => None,
                crate::ast::PathType::Curve(d) => Some(d),
            };

            interactions.push(Interaction::Move(MoveLine {
                player_id: move_action.player,
                from,
                to: move_action.target,
                curve,
            }));
        }

        // Passes
        for pass in playbook.action.passes {
            // check from is passer?
            if !is_baller(&entities, &pass.from) {
                return Err(format!("Player {} does not have the ball", pass.from));
            }

            let from = *end_positions.get(&pass.from).unwrap_or(&(0.0, 0.0)); // Ball moves after or during action
            let to = match pass.timing {
                Timing::Before => *start_positions.get(&pass.to).unwrap_or(&(0.0, 0.0)),
                Timing::Middle => {
                    let start = *start_positions.get(&pass.to).unwrap_or(&(0.0, 0.0));
                    let end = *end_positions.get(&pass.to).unwrap_or(&(0.0, 0.0));
                    ((start.0 + end.0) / 2.0, (start.1 + end.1) / 2.0)
                }
                Timing::After | Timing::None => *end_positions.get(&pass.to).unwrap_or(&(0.0, 0.0)),
            };
            interactions.push(Interaction::Pass(PassLine { from, to }));
        }

        // Screens
        for screen in playbook.action.screens {
            let from = *start_positions.get(&screen.player).unwrap_or(&(0.0, 0.0));
            let to = match &screen.target {
                crate::ast::ScreenTarget::Player(target_id) => match screen.timing {
                    Timing::Before => *start_positions.get(target_id).unwrap_or(&(0.0, 0.0)),
                    Timing::Middle => {
                        let start = *start_positions.get(target_id).unwrap_or(&(0.0, 0.0));
                        let end = *end_positions.get(target_id).unwrap_or(&(0.0, 0.0));
                        ((start.0 + end.0) / 2.0, (start.1 + end.1) / 2.0)
                    }
                    Timing::After | Timing::None => {
                        *end_positions.get(target_id).unwrap_or(&(0.0, 0.0))
                    }
                },
                crate::ast::ScreenTarget::Coordinate(x, y) => (*x, *y),
            };

            let curve = match screen.path_type {
                crate::ast::PathType::Straight => None,
                crate::ast::PathType::Curve(d) => Some(d),
            };

            interactions.push(Interaction::Screen(ScreenLine {
                screener_id: screen.player,
                from,
                to,
                curve,
            }));
        }

        Ok(Scene {
            entities,
            interactions,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::*;
    use std::collections::HashMap;

    #[test]
    fn test_ir_generation() {
        let mut positions = HashMap::new();
        positions.insert("p1".to_string(), (0.0, 0.0));
        positions.insert("p2".to_string(), (10.0, 10.0));

        let playbook = Playbook {
            players: vec!["p1".to_string(), "p2".to_string()],
            state: State {
                baller: Some("p1".to_string()),
                positions,
            },
            action: Action {
                moves: vec![MoveAction {
                    player: "p2".to_string(),
                    target: (20.0, 20.0),
                    path_type: PathType::Straight,
                }],
                passes: vec![PassAction {
                    from: "p1".to_string(),
                    to: "p2".to_string(),
                    timing: Timing::After,
                }],
                ..Default::default()
            },
        };

        let scene = IRGenerator::generate(playbook).unwrap();

        assert_eq!(scene.entities.len(), 2);
        let p2_entity = scene.entities.iter().find(|e| e.id == "p2").unwrap();
        assert_eq!(p2_entity.start_pos, (10.0, 10.0));
        assert_eq!(p2_entity.end_pos, (20.0, 20.0));

        // Pass should go to p2's end_pos because timing is After
        if let Interaction::Pass(pass) = &scene.interactions[1] {
            assert_eq!(pass.to, (20.0, 20.0));
        } else {
            panic!("Expected Pass interaction");
        }
    }

    #[test]
    fn test_pass_without_ball() {
        let mut positions = HashMap::new();
        positions.insert("p1".to_string(), (0.0, 0.0));
        positions.insert("p2".to_string(), (10.0, 10.0));

        let playbook = Playbook {
            players: vec!["p1".to_string(), "p2".to_string()],
            state: State {
                baller: Some("p2".to_string()), // p2 has the ball
                positions,
            },
            action: Action {
                passes: vec![PassAction {
                    from: "p1".to_string(), // p1 tries to pass
                    to: "p2".to_string(),
                    timing: Timing::After,
                }],
                ..Default::default()
            },
        };

        let result = IRGenerator::generate(playbook);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Player p1 does not have the ball");
    }
}
