use crate::ast::{Playbook, Timing};
use crate::ir::*;

pub struct IRGenerator;

impl IRGenerator {
    pub fn generate(playbook: Playbook) -> Result<Scene, String> {
        let mut entities = Vec::new();
        let mut interactions = Vec::new();

        let initial_positions = playbook.state.positions.clone();
        let mut current_positions = initial_positions.clone();
        let mut current_baller = playbook.state.baller.clone();

        for action in playbook.actions {
            let phase_start_positions = current_positions.clone();
            let mut phase_end_positions = phase_start_positions.clone();

            // 1. Resolve end positions for this phase
            for move_action in &action.moves {
                phase_end_positions.insert(move_action.player.clone(), move_action.target);
            }

            // 2. Create Interactions for this phase
            // Moves
            for move_action in action.moves {
                let from = *phase_start_positions
                    .get(&move_action.player)
                    .unwrap_or(&(0.0, 0.0));

                let curve = match move_action.path_type {
                    crate::ast::PathType::Straight => None,
                    crate::ast::PathType::Curve(d) => Some(d),
                };

                let is_dribble = current_baller.as_ref() == Some(&move_action.player);

                interactions.push(Interaction::Move(MoveLine {
                    player_id: move_action.player,
                    from,
                    to: move_action.target,
                    curve,
                    is_dribble,
                }));
            }

            // Screens
            for screen in action.screens {
                let from = *phase_start_positions
                    .get(&screen.player)
                    .unwrap_or(&(0.0, 0.0));
                let to = match &screen.target {
                    crate::ast::ScreenTarget::Player(target_id) => match screen.timing {
                        Timing::Before => {
                            *phase_start_positions.get(target_id).unwrap_or(&(0.0, 0.0))
                        }
                        Timing::Middle => {
                            let start =
                                *phase_start_positions.get(target_id).unwrap_or(&(0.0, 0.0));
                            let end = *phase_end_positions.get(target_id).unwrap_or(&(0.0, 0.0));
                            ((start.0 + end.0) / 2.0, (start.1 + end.1) / 2.0)
                        }
                        Timing::After | Timing::None => {
                            *phase_end_positions.get(target_id).unwrap_or(&(0.0, 0.0))
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

            // Passes
            for pass in action.passes {
                if current_baller.as_ref() != Some(&pass.from) {
                    return Err(format!("Player {} does not have the ball", pass.from));
                }

                let from = *phase_end_positions.get(&pass.from).unwrap_or(&(0.0, 0.0));
                let to = match pass.timing {
                    Timing::Before => *phase_start_positions.get(&pass.to).unwrap_or(&(0.0, 0.0)),
                    Timing::Middle => {
                        let start = *phase_start_positions.get(&pass.to).unwrap_or(&(0.0, 0.0));
                        let end = *phase_end_positions.get(&pass.to).unwrap_or(&(0.0, 0.0));
                        ((start.0 + end.0) / 2.0, (start.1 + end.1) / 2.0)
                    }
                    Timing::After | Timing::None => {
                        *phase_end_positions.get(&pass.to).unwrap_or(&(0.0, 0.0))
                    }
                };
                interactions.push(Interaction::Pass(PassLine { from, to }));
                current_baller = Some(pass.to.clone());
            }

            // Update current positions for the next phase
            current_positions = phase_end_positions;
        }

        // 3. Create Entities with final state
        // entities for drawing
        let initial_baller = playbook.state.baller.as_ref();
        for player_id in playbook.players {
            let start_pos = *initial_positions.get(&player_id).unwrap_or(&(0.0, 0.0));
            let end_pos = *current_positions.get(&player_id).unwrap_or(&start_pos);
            let is_baller = initial_baller == Some(&player_id);

            entities.push(Entity {
                id: player_id.clone(),
                label: player_id.replace("p", ""), // p1 -> 1
                start_pos,
                end_pos,
                is_baller,
            });
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
            actions: vec![Action {
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
            }],
        };

        let scene = IRGenerator::generate(playbook).unwrap();

        assert_eq!(scene.entities.len(), 2);
        let p1_entity = scene.entities.iter().find(|e| e.id == "p1").unwrap();
        let p2_entity = scene.entities.iter().find(|e| e.id == "p2").unwrap();
        assert_eq!(p2_entity.start_pos, (10.0, 10.0));
        assert_eq!(p2_entity.end_pos, (20.0, 20.0));
        assert!(p1_entity.is_baller);

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
            actions: vec![Action {
                passes: vec![PassAction {
                    from: "p1".to_string(), // p1 tries to pass
                    to: "p2".to_string(),
                    timing: Timing::After,
                }],
                ..Default::default()
            }],
        };

        let result = IRGenerator::generate(playbook);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Player p1 does not have the ball");
    }

    #[test]
    fn test_multiple_phases_ir() {
        let mut positions = HashMap::new();
        positions.insert("p1".to_string(), (0.0, 0.0));
        positions.insert("p2".to_string(), (10.0, 10.0));

        let playbook = Playbook {
            players: vec!["p1".to_string(), "p2".to_string()],
            state: State {
                baller: Some("p1".to_string()),
                positions,
            },
            actions: vec![
                Action {
                    moves: vec![MoveAction {
                        player: "p1".to_string(),
                        target: (10.0, 0.0),
                        path_type: PathType::Straight,
                    }],
                    ..Default::default()
                },
                Action {
                    moves: vec![MoveAction {
                        player: "p1".to_string(),
                        target: (10.0, 10.0),
                        path_type: PathType::Straight,
                    }],
                    passes: vec![PassAction {
                        from: "p1".to_string(),
                        to: "p2".to_string(),
                        timing: Timing::After,
                    }],
                    ..Default::default()
                },
            ],
        };

        let scene = IRGenerator::generate(playbook).unwrap();

        let p1_entity = scene.entities.iter().find(|e| e.id == "p1").unwrap();
        assert!(p1_entity.is_baller);

        // Should have 2 moves for p1 and 1 pass
        assert_eq!(scene.interactions.len(), 3);

        // Phase 1 Move
        if let Interaction::Move(m) = &scene.interactions[0] {
            assert_eq!(m.player_id, "p1");
            assert_eq!(m.from, (0.0, 0.0));
            assert_eq!(m.to, (10.0, 0.0));
            assert!(m.is_dribble); // p1 is baller
        } else {
            panic!("Expected Move interaction");
        }

        // Phase 2 Move
        if let Interaction::Move(m) = &scene.interactions[1] {
            assert_eq!(m.player_id, "p1");
            assert_eq!(m.from, (10.0, 0.0)); // From Phase 1 end
            assert_eq!(m.to, (10.0, 10.0));
            assert!(m.is_dribble); // p1 is still baller
        } else {
            panic!("Expected Move interaction");
        }

        // Pass
        if let Interaction::Pass(p) = &scene.interactions[2] {
            assert_eq!(p.from, (10.0, 10.0));
            assert_eq!(p.to, (10.0, 10.0)); // p2 stayed at (10, 10)
        } else {
            panic!("Expected Pass interaction");
        }
    }

    #[test]
    fn test_sequential_passes() {
        let mut positions = HashMap::new();
        positions.insert("p1".to_string(), (0.0, 0.0));
        positions.insert("p2".to_string(), (10.0, 10.0));
        positions.insert("p3".to_string(), (20.0, 20.0));

        let playbook = Playbook {
            players: vec!["p1".to_string(), "p2".to_string(), "p3".to_string()],
            state: State {
                baller: Some("p1".to_string()),
                positions,
            },
            actions: vec![Action {
                passes: vec![
                    PassAction {
                        from: "p1".to_string(),
                        to: "p2".to_string(),
                        timing: Timing::None,
                    },
                    PassAction {
                        from: "p2".to_string(),
                        to: "p3".to_string(),
                        timing: Timing::None,
                    },
                ],
                ..Default::default()
            }],
        };

        let scene = IRGenerator::generate(playbook).expect("Sequential passes should work");
        assert_eq!(scene.interactions.len(), 2);

        // Even after multiple passes, initial baller for rendering is p1
        let p1_entity = scene.entities.iter().find(|e| e.id == "p1").unwrap();
        let p2_entity = scene.entities.iter().find(|e| e.id == "p2").unwrap();
        assert!(p1_entity.is_baller);
        assert!(!p2_entity.is_baller);
    }
}
