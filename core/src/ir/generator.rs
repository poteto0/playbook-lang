use crate::ast::{PathType, Playbook, ScreenTarget, Timing};
use crate::ir::*;
use crate::lexer::Span;
use std::collections::HashMap;

pub struct IRGenerator;

/// Derives an entity's display label from its player id by stripping a single
/// leading `p` (e.g. `p1` -> `1`). Ids without the prefix are kept verbatim.
fn player_label(player_id: &str) -> &str {
    player_id.strip_prefix('p').unwrap_or(player_id)
}

/// Resolves a player's position for a given timing relative to a phase.
///
/// `Before` uses the phase's start positions, `After`/`None` its end
/// positions, and `Middle` the average of the two.
fn resolve_timed_position(
    timing: Timing,
    player_id: &str,
    span: Span,
    start_positions: &HashMap<String, (f64, f64)>,
    end_positions: &HashMap<String, (f64, f64)>,
) -> Result<(f64, f64), IRError> {
    let lookup = |positions: &HashMap<String, (f64, f64)>| {
        positions
            .get(player_id)
            .copied()
            .ok_or_else(|| IRError::UnexpectedPlayer(span, player_id.to_string()))
    };

    match timing {
        Timing::Before => lookup(start_positions),
        Timing::After | Timing::None => lookup(end_positions),
        Timing::Middle => {
            let start = lookup(start_positions)?;
            let end = lookup(end_positions)?;
            Ok(((start.0 + end.0) / 2.0, (start.1 + end.1) / 2.0))
        }
    }
}

impl IRGenerator {
    pub fn generate(playbook: Playbook) -> Result<Scene, IRError> {
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
                let from = match phase_start_positions.get(&move_action.player) {
                    Some(pos) => *pos,
                    None => {
                        return Err(IRError::UnexpectedPlayer(
                            move_action.span,
                            move_action.player.clone(),
                        ));
                    }
                };

                let curve = match move_action.path_type {
                    PathType::Straight => None,
                    PathType::Curve(d) => Some(d),
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
                let from = match phase_start_positions.get(&screen.player) {
                    Some(pos) => *pos,
                    None => {
                        return Err(IRError::UnexpectedPlayer(
                            screen.span,
                            screen.player.clone(),
                        ));
                    }
                };

                let to = match &screen.target {
                    ScreenTarget::Player(target_id) => resolve_timed_position(
                        screen.timing,
                        target_id,
                        screen.span,
                        &phase_start_positions,
                        &phase_end_positions,
                    )?,
                    ScreenTarget::Coordinate(x, y) => (*x, *y),
                };

                let curve = match screen.path_type {
                    PathType::Straight => None,
                    PathType::Curve(d) => Some(d),
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
                    return Err(IRError::PlayerNotBaller(pass.span, pass.from.clone()));
                }

                let from = *phase_end_positions
                    .get(&pass.from)
                    .ok_or_else(|| IRError::UnexpectedPlayer(pass.span, pass.from.clone()))?;

                let to = resolve_timed_position(
                    pass.timing,
                    &pass.to,
                    pass.span,
                    &phase_start_positions,
                    &phase_end_positions,
                )?;
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
            // If a player listed in 'players' is not in 'state.positions', that's an issue
            // but we don't have a specific span for 'players' list entries here easily unless passed.
            // But they should be in initial_positions.
            // If not, we can default to (0,0) or error.
            // Since we don't have a span for the player definition here, let's skip or error with dummy span.
            // Better: use a dummy span or change the return type.
            // For now, let's assume they exist or error with a generic span if possible,
            // but we don't have one.
            // We will use unwrap_or for now as this is less critical than action errors,
            // or we could use the first action's span if available? No.
            // Let's keep unwrap_or for entity generation as a fallback, or better:
            // Since initial_positions comes from state, if they aren't there, they aren't on court.

            let start_pos = *initial_positions.get(&player_id).unwrap_or(&(0.0, 0.0));
            let end_pos = *current_positions.get(&player_id).unwrap_or(&start_pos);
            let is_baller = initial_baller == Some(&player_id);

            entities.push(Entity {
                id: player_id.clone(),
                label: player_label(&player_id).to_string(),
                start_pos,
                end_pos,
                is_baller,
            });
        }

        Ok(Scene {
            entities,
            interactions,
            final_baller: current_baller,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::*;
    use crate::lexer::Span;
    use std::collections::HashMap;

    // Helper to create a dummy span
    fn dummy_span() -> Span {
        Span {
            start: 0,
            end: 0,
            line: 0,
            column: 0,
        }
    }

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
                    span: dummy_span(),
                }],
                passes: vec![PassAction {
                    from: "p1".to_string(),
                    to: "p2".to_string(),
                    timing: Timing::After,
                    span: dummy_span(),
                }],
                ..Default::default()
            }],
            comments: vec![],
        };

        let scene = IRGenerator::generate(playbook).unwrap();

        assert_eq!(scene.entities.len(), 2);
        let p1_entity = scene.entities.iter().find(|e| e.id == "p1").unwrap();
        let p2_entity = scene.entities.iter().find(|e| e.id == "p2").unwrap();
        assert_eq!(p2_entity.start_pos, (10.0, 10.0));
        assert_eq!(p2_entity.end_pos, (20.0, 20.0));
        assert!(p1_entity.is_baller);
        assert_eq!(scene.final_baller, Some("p2".to_string()));
    }

    #[test]
    fn test_pass_without_ball() {
        let mut positions = HashMap::new();
        positions.insert("p1".to_string(), (0.0, 0.0));
        positions.insert("p2".to_string(), (10.0, 10.0));

        let playbook = Playbook {
            players: vec!["p1".to_string(), "p2".to_string()],
            state: State {
                baller: Some("p2".to_string()),
                positions,
            },
            actions: vec![Action {
                passes: vec![PassAction {
                    from: "p1".to_string(),
                    to: "p2".to_string(),
                    timing: Timing::After,
                    span: dummy_span(),
                }],
                ..Default::default()
            }],
            comments: vec![],
        };

        let result = IRGenerator::generate(playbook);
        assert!(result.is_err());
        match result.unwrap_err() {
            IRError::PlayerNotBaller(_, name) => assert_eq!(name, "p1"),
            _ => panic!("Expected PlayerNotBaller"),
        }
    }

    #[test]
    fn test_undefined_player_error() {
        let mut positions = HashMap::new();
        positions.insert("p1".to_string(), (0.0, 0.0));

        let playbook = Playbook {
            players: vec!["p1".to_string()],
            state: State {
                baller: Some("p1".to_string()),
                positions,
            },
            actions: vec![Action {
                moves: vec![MoveAction {
                    player: "p99".to_string(),
                    target: (10.0, 10.0),
                    path_type: PathType::Straight,
                    span: dummy_span(),
                }],
                ..Default::default()
            }],
            comments: vec![],
        };

        let result = IRGenerator::generate(playbook);
        assert!(result.is_err());
        match result.unwrap_err() {
            IRError::UnexpectedPlayer(_, name) => assert_eq!(name, "p99"),
            _ => panic!("Expected UnexpectedPlayer"),
        }
    }

    #[test]
    fn test_label_strips_only_leading_p() {
        let mut positions = HashMap::new();
        positions.insert("p1".to_string(), (0.0, 0.0));
        positions.insert("player3".to_string(), (1.0, 1.0));
        positions.insert("pp7".to_string(), (2.0, 2.0));
        positions.insert("top".to_string(), (3.0, 3.0));

        let playbook = Playbook {
            players: vec![
                "p1".to_string(),
                "player3".to_string(),
                "pp7".to_string(),
                "top".to_string(),
            ],
            state: State {
                baller: Some("p1".to_string()),
                positions,
            },
            actions: vec![],
            comments: vec![],
        };

        let scene = IRGenerator::generate(playbook).unwrap();

        let label = |id: &str| {
            scene
                .entities
                .iter()
                .find(|e| e.id == id)
                .unwrap()
                .label
                .clone()
        };

        assert_eq!(label("p1"), "1");
        assert_eq!(label("player3"), "layer3");
        assert_eq!(label("pp7"), "p7");
        assert_eq!(label("top"), "top");
    }
}
