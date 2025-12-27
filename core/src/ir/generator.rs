use crate::ast::{Playbook, Timing};
use crate::ir::*;

pub struct IRGenerator;

impl IRGenerator {
    pub fn generate(playbook: Playbook) -> Scene {
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
            let from = *start_positions.get(&move_action.player).unwrap_or(&(0.0, 0.0));
            interactions.push(Interaction::Move(MoveLine {
                player_id: move_action.player,
                from,
                to: move_action.target,
            }));
        }

        // Passes
        for pass in playbook.action.passes {
            let from = *end_positions.get(&pass.from).unwrap_or(&(0.0, 0.0)); // Ball moves after or during action
            let to = match pass.timing {
                Timing::Before => *start_positions.get(&pass.to).unwrap_or(&(0.0, 0.0)),
                Timing::After | Timing::None => *end_positions.get(&pass.to).unwrap_or(&(0.0, 0.0)),
            };
            interactions.push(Interaction::Pass(PassLine { from, to }));
        }

        // Screens
        for screen in playbook.action.screens {
            let from = *start_positions.get(&screen.player).unwrap_or(&(0.0, 0.0));
            let to = match screen.timing {
                Timing::Before => *start_positions.get(&screen.target).unwrap_or(&(0.0, 0.0)),
                Timing::After | Timing::None => *end_positions.get(&screen.target).unwrap_or(&(0.0, 0.0)),
            };
            interactions.push(Interaction::Screen(ScreenLine {
                screener_id: screen.player,
                from,
                to,
            }));
        }

        Scene {
            entities,
            interactions,
        }
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
                }],
                passes: vec![PassAction {
                    from: "p1".to_string(),
                    to: "p2".to_string(),
                    timing: Timing::After,
                }],
                ..Default::default()
            },
        };

        let scene = IRGenerator::generate(playbook);

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
}
