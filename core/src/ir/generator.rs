use crate::ast::{Action, DefenseTarget, PathType, Playbook, ScreenTarget, Timing};
use crate::geometry::normalize;
use crate::ir::*;
use crate::lexer::Span;
use std::collections::{HashMap, HashSet};

pub struct IRGenerator;

/// Offsets `pos` towards the center of the court (the origin) by `distance`.
/// If `pos` is already (effectively) at the center, no direction exists to
/// offset towards, so the position is returned unchanged. `distance` is
/// clamped to `pos`'s distance from center so the offset can never overshoot
/// past the center to the far side of the tracked position.
fn offset_towards_center(pos: (f64, f64), distance: f64) -> (f64, f64) {
    let (x, y) = pos;
    let Some((ux, uy, len)) = normalize(x, y, 1e-9) else {
        return pos;
    };
    let distance = distance.min(len);
    (x - ux * distance, y - uy * distance)
}

/// Looks up a player's position for a given `timing` relative to a phase:
/// `Before` uses the phase's start positions, `After`/`None` its end
/// positions, and `Middle` the average of the two. Returns `None` if the
/// player is absent from a map the timing needs.
fn timed_position(
    timing: &Timing,
    player_id: &str,
    start_positions: &HashMap<String, (f64, f64)>,
    end_positions: &HashMap<String, (f64, f64)>,
) -> Option<(f64, f64)> {
    match timing {
        Timing::Before => start_positions.get(player_id).copied(),
        Timing::After | Timing::None => end_positions.get(player_id).copied(),
        Timing::Middle => {
            let start = start_positions.get(player_id).copied()?;
            let end = end_positions.get(player_id).copied()?;
            Some(((start.0 + end.0) / 2.0, (start.1 + end.1) / 2.0))
        }
    }
}

/// Resolves what a defender's target position is, given the player positions
/// at the phase's start and end. A fixed `Position` always resolves; a `Mark`
/// tracks the player at its `timing` position, offset towards the center, and
/// fails with the marked player's id if that player can't be found.
fn resolve_defense_target<'a>(
    target: &'a DefenseTarget,
    start_positions: &HashMap<String, (f64, f64)>,
    end_positions: &HashMap<String, (f64, f64)>,
) -> Result<(f64, f64), &'a str> {
    match target {
        DefenseTarget::Position(x, y) => Ok((*x, *y)),
        DefenseTarget::Mark {
            player,
            offset,
            timing,
        } => timed_position(timing, player, start_positions, end_positions)
            .map(|p| offset_towards_center(p, *offset))
            .ok_or(player),
    }
}

/// Resolves the initial (state) position of every defender listed in
/// `defense`. A Mark referencing a player not present in `positions` fails
/// with `IRError::UnexpectedPlayer`, matching the strict behavior of
/// `action.defense` marks.
fn resolve_initial_defense_positions(
    defense: &HashMap<String, (DefenseTarget, Span)>,
    positions: &HashMap<String, (f64, f64)>,
) -> Result<HashMap<String, (f64, f64)>, IRError> {
    defense
        .iter()
        .map(|(defender, (target, span))| {
            // In `state`, players are static, so start and end positions are
            // the same map; any Mark timing resolves to that fixed position.
            let pos = resolve_defense_target(target, positions, positions)
                .map_err(|player| IRError::UnexpectedPlayer(*span, player.to_string()))?;
            Ok((defender.clone(), pos))
        })
        .collect()
}

/// Derives an entity's display label from its player id by stripping a single
/// leading `p` (e.g. `p1` -> `1`). Ids without the prefix are kept verbatim.
fn player_label(player_id: &str) -> &str {
    player_id.strip_prefix('p').unwrap_or(player_id)
}

/// [`timed_position`] wrapper that fails with `IRError::UnexpectedPlayer`
/// (carrying `span`) when the player can't be resolved.
fn resolve_timed_position(
    timing: Timing,
    player_id: &str,
    span: Span,
    start_positions: &HashMap<String, (f64, f64)>,
    end_positions: &HashMap<String, (f64, f64)>,
) -> Result<(f64, f64), IRError> {
    timed_position(&timing, player_id, start_positions, end_positions)
        .ok_or_else(|| IRError::UnexpectedPlayer(span, player_id.to_string()))
}

/// Checks that no entity is given the same action `kind` (e.g. `move`) twice
/// inside one `action` block. Two entries of the same kind for the same id
/// are processed sequentially over the same phase, rendering contradictory
/// overlapping lines from the same start position, so the duplicate (second)
/// entry is an error instead. `span_of` yields the source span of each entry.
fn validate_unique<'a, E>(
    kind: &'static str,
    entries: impl IntoIterator<Item = E>,
    span_of: impl Fn(&E) -> Span,
    id_of: impl Fn(&E) -> &'a str,
) -> Result<(), IRError> {
    let mut seen = HashSet::new();
    for entry in entries {
        let id = id_of(&entry);
        if !seen.insert(id) {
            let span = span_of(&entry);
            return Err(IRError::DuplicateAction(span, kind, id.to_string()));
        }
    }
    Ok(())
}

/// Validates one `action` block's duplicate-entry rules (see
/// [`validate_unique`]): `moves` keyed by `player`, `screens` keyed by
/// `screener`, `defenses` keyed by `defender`.
fn validate_action(action: &Action) -> Result<(), IRError> {
    validate_unique("move", &action.moves, |m| m.span, |m| m.player.as_str())?;
    validate_unique("screen", &action.screens, |s| s.span, |s| s.player.as_str())?;
    validate_unique(
        "defense",
        &action.defenses,
        |d| d.span,
        |d| d.defender.as_str(),
    )
}

impl IRGenerator {
    pub fn generate(playbook: Playbook) -> Result<Scene, IRError> {
        let mut entities = Vec::new();
        let mut interactions = Vec::new();

        let initial_positions: HashMap<_, _> = playbook
            .state
            .positions
            .iter()
            .map(|(id, (pos, _))| (id.clone(), *pos))
            .collect();
        let mut current_positions = initial_positions.clone();
        let mut current_baller = playbook.state.baller.clone();

        let initial_defense_positions =
            resolve_initial_defense_positions(&playbook.state.defense, &initial_positions)?;
        let mut current_defense_positions = initial_defense_positions.clone();

        // Every player listed in `players`, and every defender listed in
        // `defenders`, must have a position in `state` (`state.position` and
        // `state.defense` respectively); without one it would be drawn at
        // (0, 0), hiding a real mistake from the author. Fail with an error
        // that points at the corresponding section keyword. `section_spans`
        // is always populated by the parser for these sections;
        // `Span::default()` only covers direct API users.
        for player_id in &playbook.players {
            if !initial_positions.contains_key(player_id) {
                return Err(IRError::MissingPosition(
                    playbook.section_spans.players.unwrap_or_default(),
                    player_id.clone(),
                ));
            }
        }
        for defender_id in &playbook.defenders {
            // The initial map is byte-identical here (it is cloned once);
            // read it so the pre-action-loop invariant stays clear even if
            // the code below is reordered.
            if !initial_defense_positions.contains_key(defender_id) {
                return Err(IRError::MissingDefensePosition(
                    playbook.section_spans.defenders.unwrap_or_default(),
                    defender_id.clone(),
                ));
            }
        }

        for action in playbook.actions {
            validate_action(&action)?;
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

            // Defenders: move to a fixed position, or mark (track) a player,
            // offset towards the center of the court. `current_defense_positions`
            // is only read here (it's reassigned once, below), so `from` can be
            // read directly from it without a separate frozen snapshot.
            let mut phase_end_defense_positions = current_defense_positions.clone();
            for defense_action in action.defenses {
                let defender = &defense_action.defender;
                let from = *current_defense_positions.get(defender).ok_or_else(|| {
                    IRError::UnexpectedPlayer(defense_action.span, defender.clone())
                })?;

                let to = resolve_defense_target(
                    &defense_action.target,
                    &phase_start_positions,
                    &phase_end_positions,
                )
                .map_err(|player| {
                    IRError::UnexpectedPlayer(defense_action.span, player.to_string())
                })?;

                phase_end_defense_positions.insert(defense_action.defender.clone(), to);
                interactions.push(Interaction::Defense(DefenseLine {
                    defender_id: defense_action.defender,
                    from,
                    to,
                }));
            }

            // Update current positions for the next phase
            current_positions = phase_end_positions;
            current_defense_positions = phase_end_defense_positions;
        }

        // 3. Create Entities with final state
        // entities for drawing
        let initial_baller = playbook.state.baller.as_ref();
        for player_id in playbook.players {
            // Positions were validated above: every listed player is present
            // in both maps, so indexing is total.
            let start_pos = initial_positions[&player_id];
            let end_pos = current_positions[&player_id];
            let is_baller = initial_baller == Some(&player_id);

            entities.push(Entity {
                id: player_id.clone(),
                kind: EntityKind::Player,
                label: player_label(&player_id).to_string(),
                start_pos,
                end_pos,
                is_baller,
            });
        }

        // Defenders are drawn with a fixed "x" label and never carry the ball.
        for defender_id in playbook.defenders {
            let start_pos = initial_defense_positions[&defender_id];
            let end_pos = current_defense_positions[&defender_id];

            entities.push(Entity {
                id: defender_id,
                kind: EntityKind::Defender,
                label: "x".to_string(),
                start_pos,
                end_pos,
                is_baller: false,
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
    /// Builds a `state.defense` map placing each of `ids` at the fixed `pos`.
    fn defense_at(pos: (f64, f64), ids: &[&str]) -> HashMap<String, (DefenseTarget, Span)> {
        ids.iter()
            .map(|id| {
                (
                    id.to_string(),
                    (DefenseTarget::Position(pos.0, pos.1), dummy_span()),
                )
            })
            .collect()
    }

    fn dummy_span() -> Span {
        Span {
            start: 0,
            end: 0,
            line: 0,
            column: 0,
        }
    }

    /// Span whose line/column match the assertion variables of the
    /// missing-position tests.
    fn span_at(line: usize, column: usize) -> Span {
        Span {
            start: 10,
            end: 17,
            line,
            column,
        }
    }

    #[test]
    fn test_ir_generation() {
        let mut positions = HashMap::new();
        positions.insert("p1".to_string(), ((0.0, 0.0), dummy_span()));
        positions.insert("p2".to_string(), ((10.0, 10.0), dummy_span()));

        let playbook = Playbook {
            players: vec!["p1".to_string(), "p2".to_string()],
            defenders: vec![],
            state: State {
                baller: Some("p1".to_string()),
                positions,
                ..Default::default()
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
            ..Default::default()
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
        positions.insert("p1".to_string(), ((0.0, 0.0), dummy_span()));
        positions.insert("p2".to_string(), ((10.0, 10.0), dummy_span()));

        let playbook = Playbook {
            players: vec!["p1".to_string(), "p2".to_string()],
            defenders: vec![],
            state: State {
                baller: Some("p2".to_string()),
                positions,
                ..Default::default()
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
            ..Default::default()
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
        positions.insert("p1".to_string(), ((0.0, 0.0), dummy_span()));

        let playbook = Playbook {
            players: vec!["p1".to_string()],
            defenders: vec![],
            state: State {
                baller: Some("p1".to_string()),
                positions,
                ..Default::default()
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
            ..Default::default()
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
        positions.insert("p1".to_string(), ((0.0, 0.0), dummy_span()));
        positions.insert("player3".to_string(), ((1.0, 1.0), dummy_span()));
        positions.insert("pp7".to_string(), ((2.0, 2.0), dummy_span()));
        positions.insert("top".to_string(), ((3.0, 3.0), dummy_span()));

        let playbook = Playbook {
            players: vec![
                "p1".to_string(),
                "player3".to_string(),
                "pp7".to_string(),
                "top".to_string(),
            ],
            defenders: vec![],
            state: State {
                baller: Some("p1".to_string()),
                positions,
                ..Default::default()
            },
            actions: vec![],
            ..Default::default()
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

    #[test]
    fn test_defender_initial_position_and_mark() {
        let mut positions = HashMap::new();
        positions.insert("p1".to_string(), ((0.0, 60.0), dummy_span()));

        let mut defense = HashMap::new();
        defense.insert(
            "d1".to_string(),
            (
                DefenseTarget::Mark {
                    player: "p1".to_string(),
                    offset: 10.0,
                    timing: Timing::None,
                },
                dummy_span(),
            ),
        );
        defense.insert(
            "d2".to_string(),
            (DefenseTarget::Position(-90.0, -80.0), dummy_span()),
        );

        let playbook = Playbook {
            players: vec!["p1".to_string()],
            defenders: vec!["d1".to_string(), "d2".to_string()],
            state: State {
                baller: None,
                positions,
                defense,
                ..Default::default()
            },
            actions: vec![],
            ..Default::default()
        };

        let scene = IRGenerator::generate(playbook).unwrap();

        let d1 = scene.entities.iter().find(|e| e.id == "d1").unwrap();
        assert_eq!(d1.kind, EntityKind::Defender);
        assert_eq!(d1.label, "x");
        assert!(!d1.is_baller);
        // p1 at (0, 60), offset 10 towards the center (0, 0) => (0, 50).
        assert_eq!(d1.start_pos, (0.0, 50.0));
        assert_eq!(d1.end_pos, (0.0, 50.0));

        let d2 = scene.entities.iter().find(|e| e.id == "d2").unwrap();
        assert_eq!(d2.start_pos, (-90.0, -80.0));
        assert_eq!(d2.end_pos, (-90.0, -80.0));
    }

    #[test]
    fn test_defender_tracks_player_after_move() {
        let mut positions = HashMap::new();
        positions.insert("p1".to_string(), ((0.0, 60.0), dummy_span()));
        let defense = defense_at((0.0, -50.0), &["d1"]);

        let playbook = Playbook {
            players: vec!["p1".to_string()],
            defenders: vec!["d1".to_string()],
            state: State {
                baller: None,
                positions,
                defense,
                ..Default::default()
            },
            actions: vec![Action {
                moves: vec![MoveAction {
                    player: "p1".to_string(),
                    target: (0.0, 100.0),
                    path_type: PathType::Straight,
                    span: dummy_span(),
                }],
                defenses: vec![DefenseAction {
                    defender: "d1".to_string(),
                    target: DefenseTarget::Mark {
                        player: "p1".to_string(),
                        offset: 5.0,
                        timing: Timing::None,
                    },
                    span: dummy_span(),
                }],
                ..Default::default()
            }],
            ..Default::default()
        };

        let scene = IRGenerator::generate(playbook).unwrap();

        let d1 = scene.entities.iter().find(|e| e.id == "d1").unwrap();
        assert_eq!(d1.start_pos, (0.0, -50.0));
        // p1 ends at (0, 100); offset 5 towards center (0, 0) => (0, 95).
        assert_eq!(d1.end_pos, (0.0, 95.0));

        let defense_interaction = scene
            .interactions
            .iter()
            .find_map(|i| match i {
                Interaction::Defense(d) => Some(d),
                _ => None,
            })
            .expect("expected a Defense interaction");
        assert_eq!(defense_interaction.defender_id, "d1");
        assert_eq!(defense_interaction.from, (0.0, -50.0));
        assert_eq!(defense_interaction.to, (0.0, 95.0));
    }

    #[test]
    fn test_defender_mark_timing_tracks_start_middle_and_end() {
        let mut positions = HashMap::new();
        positions.insert("p1".to_string(), ((0.0, 40.0), dummy_span()));

        // p1 moves from (0, 40) to (0, 80) in this phase. A defender marking
        // p1 with `:before` tracks the start, `:after` the end, and `:middle`
        // the midpoint (all offset 0 to isolate the timing behavior).
        let mark = |defender: &str, timing: Timing| DefenseAction {
            defender: defender.to_string(),
            target: DefenseTarget::Mark {
                player: "p1".to_string(),
                offset: 0.0,
                timing,
            },
            span: dummy_span(),
        };

        let defense = defense_at((0.0, 0.0), &["d1", "d2", "d3"]);

        let playbook = Playbook {
            players: vec!["p1".to_string()],
            defenders: vec!["d1".to_string(), "d2".to_string(), "d3".to_string()],
            state: State {
                positions,
                defense,
                ..Default::default()
            },
            actions: vec![Action {
                moves: vec![MoveAction {
                    player: "p1".to_string(),
                    target: (0.0, 80.0),
                    path_type: PathType::Straight,
                    span: dummy_span(),
                }],
                defenses: vec![
                    mark("d1", Timing::Before),
                    mark("d2", Timing::After),
                    mark("d3", Timing::Middle),
                ],
                ..Default::default()
            }],
            ..Default::default()
        };

        let scene = IRGenerator::generate(playbook).unwrap();
        let end_of = |id: &str| scene.entities.iter().find(|e| e.id == id).unwrap().end_pos;
        assert_eq!(end_of("d1"), (0.0, 40.0)); // before -> p1 start
        assert_eq!(end_of("d2"), (0.0, 80.0)); // after  -> p1 end
        assert_eq!(end_of("d3"), (0.0, 60.0)); // middle -> midpoint
    }

    #[test]
    fn test_defense_mark_unknown_player_errors() {
        let defense = defense_at((0.0, 0.0), &["d1"]);

        let playbook = Playbook {
            players: vec![],
            defenders: vec!["d1".to_string()],
            state: State {
                defense,
                ..Default::default()
            },
            actions: vec![Action {
                defenses: vec![DefenseAction {
                    defender: "d1".to_string(),
                    target: DefenseTarget::Mark {
                        player: "p99".to_string(),
                        offset: 10.0,
                        timing: Timing::None,
                    },
                    span: dummy_span(),
                }],
                ..Default::default()
            }],
            ..Default::default()
        };

        let result = IRGenerator::generate(playbook);
        assert!(result.is_err());
        match result.unwrap_err() {
            IRError::UnexpectedPlayer(_, name) => assert_eq!(name, "p99"),
            other => panic!("Expected UnexpectedPlayer, got {:?}", other),
        }
    }

    #[test]
    fn test_defense_unknown_defender_errors() {
        // A defense entry for a defender with no known position must error
        // like a move for an unknown player. `d1` gets a state position so
        // the error is about the action entry `x9`, not the missing state
        // position (see test_missing_player_position_errors).
        let mut defense = HashMap::new();
        defense.insert(
            "d1".to_string(),
            (DefenseTarget::Position(0.0, 0.0), dummy_span()),
        );
        let playbook = Playbook {
            defenders: vec!["d1".to_string()],
            state: State {
                defense,
                ..Default::default()
            },
            actions: vec![Action {
                defenses: vec![DefenseAction {
                    defender: "x9".to_string(),
                    target: DefenseTarget::Position(10.0, 0.0),
                    span: dummy_span(),
                }],
                ..Default::default()
            }],
            ..Default::default()
        };

        match IRGenerator::generate(playbook).unwrap_err() {
            IRError::UnexpectedPlayer(_, name) => assert_eq!(name, "x9"),
            other => panic!("Expected UnexpectedPlayer, got {:?}", other),
        }
    }

    #[test]
    fn test_state_defense_mark_unknown_player_errors() {
        let mut defense = HashMap::new();
        defense.insert(
            "d1".to_string(),
            (
                DefenseTarget::Mark {
                    player: "p99".to_string(),
                    offset: 10.0,
                    timing: Timing::None,
                },
                dummy_span(),
            ),
        );

        let playbook = Playbook {
            players: vec![],
            defenders: vec!["d1".to_string()],
            state: State {
                defense,
                ..Default::default()
            },
            actions: vec![],
            ..Default::default()
        };

        // A state.defense Mark referencing an unknown player must error the
        // same way an action.defense Mark does, instead of silently
        // defaulting the defender to the court origin.
        let result = IRGenerator::generate(playbook);
        assert!(result.is_err());
        match result.unwrap_err() {
            IRError::UnexpectedPlayer(_, name) => assert_eq!(name, "p99"),
            other => panic!("Expected UnexpectedPlayer, got {:?}", other),
        }
    }

    #[test]
    fn test_missing_position_errors() {
        // A player listed in `players` without a `state.position` entry must
        // error instead of being silently drawn at (0, 0).
        let mut positions = HashMap::new();
        positions.insert("p1".to_string(), ((0.0, 0.0), dummy_span()));

        let section_spans = SectionSpans {
            players: Some(span_at(1, 1)),
            ..Default::default()
        };

        let playbook = Playbook {
            players: vec!["p1".to_string(), "p2".to_string()],
            state: State {
                positions,
                ..Default::default()
            },
            section_spans,
            ..Default::default()
        };

        let result = IRGenerator::generate(playbook);
        assert!(result.is_err());
        match result.unwrap_err() {
            IRError::MissingPosition(span, name) => {
                assert_eq!(name, "p2");
                assert_eq!(span.line, 1);
                assert_eq!(span.column, 1);
            }
            other => panic!("Expected MissingPosition, got {:?}", other),
        }
    }

    #[test]
    fn test_missing_defender_position_errors() {
        // A defender listed in `defenders` without a `state.defense` entry
        // (and without any position from a defense action) must error,
        // pointing at the `defenders` section keyword.
        let section_spans = SectionSpans {
            defenders: Some(span_at(2, 1)),
            ..Default::default()
        };

        let playbook = Playbook {
            players: vec!["p1".to_string()],
            defenders: vec!["d1".to_string()],
            state: State {
                positions: {
                    let mut positions = HashMap::new();
                    positions.insert("p1".to_string(), ((0.0, 0.0), dummy_span()));
                    positions
                },
                ..Default::default()
            },
            section_spans,
            ..Default::default()
        };

        let result = IRGenerator::generate(playbook);
        assert!(result.is_err());
        match result.unwrap_err() {
            IRError::MissingDefensePosition(span, name) => {
                assert_eq!(name, "d1");
                assert_eq!(span.line, 2);
                assert_eq!(span.column, 1);
            }
            other => panic!("Expected MissingDefensePosition, got {:?}", other),
        }
    }

    #[test]
    fn test_duplicate_move_errors() {
        // Two `move` entries for the same player in one `action` block must
        // error, pointing at the duplicate (second) entry, instead of
        // rendering two overlapping move lines from the same start position.
        let mut positions = HashMap::new();
        positions.insert("p1".to_string(), ((0.0, 0.0), dummy_span()));

        let playbook = Playbook {
            players: vec!["p1".to_string()],
            state: State {
                positions,
                ..Default::default()
            },
            actions: vec![Action {
                moves: vec![
                    MoveAction {
                        player: "p1".to_string(),
                        target: (0.0, 40.0),
                        path_type: PathType::Straight,
                        span: span_at(5, 13),
                    },
                    MoveAction {
                        player: "p1".to_string(),
                        target: (50.0, 0.0),
                        path_type: PathType::Straight,
                        span: span_at(6, 13),
                    },
                ],
                ..Default::default()
            }],
            ..Default::default()
        };

        let result = IRGenerator::generate(playbook);
        match result.unwrap_err() {
            IRError::DuplicateAction(span, kind, name) => {
                assert_eq!(kind, "move");
                assert_eq!(name, "p1");
                // The error points at the second entry.
                assert_eq!((span.line, span.column), (6, 13));
            }
            other => panic!("Expected DuplicateAction, got {:?}", other),
        }
    }

    #[test]
    fn test_duplicate_screen_and_defense_errors() {
        // The same rule applies per kind: two `screen` entries by the same
        // screener, and two `defense` entries for the same defender, must
        // both fail with their own kind labels.
        let make = |screens: Vec<ScreenAction>, defenses: Vec<DefenseAction>| Playbook {
            players: vec!["p1".to_string()],
            defenders: vec!["d1".to_string()],
            state: State {
                positions: {
                    let mut positions = HashMap::new();
                    positions.insert("p1".to_string(), ((0.0, 0.0), dummy_span()));
                    positions
                },
                defense: defense_at((0.0, -50.0), &["d1"]),
                ..Default::default()
            },
            actions: vec![Action {
                screens,
                defenses,
                ..Default::default()
            }],
            ..Default::default()
        };

        let dup_screens = make(
            vec![
                ScreenAction {
                    player: "p1".to_string(),
                    target: ScreenTarget::Coordinate(25.0, 25.0),
                    timing: Timing::None,
                    path_type: PathType::Straight,
                    span: dummy_span(),
                },
                ScreenAction {
                    player: "p1".to_string(),
                    target: ScreenTarget::Coordinate(50.0, 50.0),
                    timing: Timing::None,
                    path_type: PathType::Straight,
                    span: span_at(7, 21),
                },
            ],
            vec![],
        );
        match IRGenerator::generate(dup_screens).unwrap_err() {
            IRError::DuplicateAction(span, kind, name) => {
                assert_eq!(kind, "screen");
                assert_eq!(name, "p1");
                assert_eq!((span.line, span.column), (7, 21));
            }
            other => panic!("Expected DuplicateAction, got {:?}", other),
        }

        let dup_defenses = make(
            vec![],
            vec![
                DefenseAction {
                    defender: "d1".to_string(),
                    target: DefenseTarget::Position(10.0, 0.0),
                    span: dummy_span(),
                },
                DefenseAction {
                    defender: "d1".to_string(),
                    target: DefenseTarget::Position(20.0, 0.0),
                    span: span_at(8, 5),
                },
            ],
        );
        match IRGenerator::generate(dup_defenses).unwrap_err() {
            IRError::DuplicateAction(span, kind, name) => {
                assert_eq!(kind, "defense");
                assert_eq!(name, "d1");
                assert_eq!((span.line, span.column), (8, 5));
            }
            other => panic!("Expected DuplicateAction, got {:?}", other),
        }
    }

    #[test]
    fn test_same_id_in_separate_actions_is_not_a_duplicate() {
        // The check is per action block: reusing the same player (and
        // defender) across phases is the normal multi-action flow.
        let mut positions = HashMap::new();
        positions.insert("p1".to_string(), ((0.0, 0.0), dummy_span()));

        let move_to = |target: (f64, f64)| MoveAction {
            player: "p1".to_string(),
            target,
            path_type: PathType::Straight,
            span: dummy_span(),
        };
        let defense = || DefenseAction {
            defender: "d1".to_string(),
            target: DefenseTarget::Mark {
                player: "p1".to_string(),
                offset: 0.0,
                timing: Timing::None,
            },
            span: dummy_span(),
        };

        let playbook = Playbook {
            players: vec!["p1".to_string()],
            defenders: vec!["d1".to_string()],
            state: State {
                positions,
                defense: defense_at((0.0, -50.0), &["d1"]),
                ..Default::default()
            },
            actions: vec![
                Action {
                    moves: vec![move_to((0.0, 40.0))],
                    defenses: vec![defense()],
                    ..Default::default()
                },
                Action {
                    moves: vec![move_to((0.0, 80.0))],
                    defenses: vec![defense()],
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let scene = IRGenerator::generate(playbook).unwrap();
        assert_eq!(
            scene
                .entities
                .iter()
                .find(|e| e.id == "p1")
                .unwrap()
                .end_pos,
            (0.0, 80.0)
        );
    }

    #[test]
    fn test_duplicate_action_message() {
        let err = IRError::DuplicateAction(dummy_span(), "move", "p1".to_string());
        assert_eq!(
            err.message(),
            "Duplicate action 'move' for 'p1' in the same action block"
        );
    }

    #[test]
    fn test_missing_position_message() {
        let player = IRError::MissingPosition(dummy_span(), "p2".to_string());
        assert_eq!(player.message(), "Player 'p2' has no position in state");
        let defender = IRError::MissingDefensePosition(dummy_span(), "d2".to_string());
        assert_eq!(
            defender.message(),
            "Defender 'd2' has no defense entry in state"
        );
    }

    #[test]
    fn test_missing_position_defaults_span_for_direct_api_users() {
        // Without `section_spans` (only possible for callers that build a
        // `Playbook` directly, bypassing the parser) the error still fires,
        // with the default span.
        let playbook = Playbook {
            players: vec!["p1".to_string()],
            ..Default::default()
        };
        let result = IRGenerator::generate(playbook);
        match result.unwrap_err() {
            IRError::MissingPosition(span, name) => {
                assert_eq!(name, "p1");
                assert_eq!((span.line, span.column), (0, 0));
            }
            other => panic!("Expected MissingPosition, got {:?}", other),
        }
    }

    #[test]
    fn test_defense_mark_offset_clamped_to_not_overshoot_center() {
        let mut positions = HashMap::new();
        positions.insert("p1".to_string(), ((0.0, 5.0), dummy_span()));

        let mut defense = HashMap::new();
        defense.insert(
            "d1".to_string(),
            (
                DefenseTarget::Mark {
                    player: "p1".to_string(),
                    offset: 10.0,
                    timing: Timing::None,
                },
                dummy_span(),
            ),
        );

        let playbook = Playbook {
            players: vec!["p1".to_string()],
            defenders: vec!["d1".to_string()],
            state: State {
                positions,
                defense,
                ..Default::default()
            },
            actions: vec![],
            ..Default::default()
        };

        let scene = IRGenerator::generate(playbook).unwrap();
        let d1 = scene.entities.iter().find(|e| e.id == "d1").unwrap();
        // p1 is only 5 units from center; an offset of 10 must clamp to 5
        // rather than overshoot to the far side of center at (0, -5).
        assert_eq!(d1.start_pos, (0.0, 0.0));
    }
}
