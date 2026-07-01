use playbook_lang_core::ast::{
    Action, CurveDirection, DefenseAction, DefenseTarget, MoveAction, PassAction, PathType,
    Playbook, ScreenAction, ScreenTarget, State, Timing,
};
use playbook_lang_core::lexer::{Lexer, Span};
use playbook_lang_core::parser::{ParseError, Parser};
use std::collections::VecDeque;

use wasm_bindgen::prelude::*;

/// Formats the input. Returns the original input unchanged if there are parse errors.
#[wasm_bindgen]
pub fn format(input: &str) -> String {
    format_checked(input).unwrap_or_else(|_| input.to_string())
}

/// Formats the input, returning parse errors if any are found.
pub fn format_checked(input: &str) -> Result<String, Vec<ParseError>> {
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (playbook, errors) = parser.parse();

    if !errors.is_empty() {
        return Err(errors);
    }

    let mut formatter = Formatter::new(playbook);
    Ok(formatter.format())
}

struct Formatter {
    playbook: Playbook,
    comments: VecDeque<(Span, String)>,
    output: String,
    indent_level: usize,
}

impl Formatter {
    fn new(playbook: Playbook) -> Self {
        let mut comments: Vec<_> = playbook.comments.clone();
        comments.sort_by_key(|(span, _)| span.start);
        Self {
            playbook,
            comments: VecDeque::from(comments),
            output: String::new(),
            indent_level: 0,
        }
    }

    fn format(&mut self) -> String {
        let players = self.playbook.players.clone();
        let defenders = self.playbook.defenders.clone();
        let state = self.playbook.state.clone();
        let actions = self.playbook.actions.clone();

        self.format_players(players);
        self.format_defenders(defenders);
        self.format_state(state);
        self.format_actions(actions);

        self.flush_comments(usize::MAX);
        self.output.clone()
    }

    fn indent(&self) -> String {
        "  ".repeat(self.indent_level)
    }

    fn push_str(&mut self, s: &str) {
        self.output.push_str(s);
    }

    fn newline(&mut self) {
        self.output.push('\n');
    }

    fn flush_comments(&mut self, before_pos: usize) {
        while let Some((span, _)) = self.comments.front() {
            if span.start < before_pos {
                let (_, comment) = self.comments.pop_front().unwrap();
                self.push_str(&self.indent());
                self.push_str("// ");
                self.push_str(&comment);
                self.newline();
            } else {
                break;
            }
        }
    }

    fn format_players(&mut self, players: Vec<String>) {
        if players.is_empty() {
            return;
        }

        if let Some((span, _)) = self.comments.front() {
            self.flush_comments(span.start + 1);
        }

        self.push_str("players = { ");
        self.push_str(&players.join(", "));
        self.push_str(" }\n\n");
    }

    fn format_defenders(&mut self, defenders: Vec<String>) {
        if defenders.is_empty() {
            return;
        }

        self.push_str("defenders = { ");
        self.push_str(&defenders.join(", "));
        self.push_str(" }\n\n");
    }

    /// Formats a single `defense` entry's target: a fixed position (using
    /// `coord_op`, which differs between `state` (`=`) and `action` (`->`))
    /// or a player mark, always spelled out explicitly (`-[offset]>`).
    fn format_defense_target(&self, target: &DefenseTarget, coord_op: &str) -> String {
        match target {
            DefenseTarget::Position(x, y) => format!("{} ({}, {})", coord_op, x, y),
            DefenseTarget::Mark { player, offset } => format!("-[{}]> {}", offset, player),
        }
    }

    fn format_state(&mut self, state: State) {
        self.push_str("state = {\n");
        self.indent_level += 1;

        if let Some(ref baller) = state.baller {
            self.push_str(&self.indent());
            self.push_str("baller = ");
            self.push_str(baller);
            self.push_str(",\n");
        }

        if !state.positions.is_empty() {
            self.push_str(&self.indent());
            self.push_str("position = {\n");
            self.indent_level += 1;

            let mut sorted_players: Vec<_> = state.positions.keys().collect();
            sorted_players.sort();
            for player in sorted_players {
                let (x, y) = state.positions.get(player).unwrap();
                self.push_str(&self.indent());
                self.push_str(&format!("{} = ({}, {}),\n", player, x, y));
            }

            self.indent_level -= 1;
            self.push_str(&self.indent());
            self.push_str("},\n");
        }

        if !state.defense.is_empty() {
            self.push_str(&self.indent());
            self.push_str("defense = {\n");
            self.indent_level += 1;

            let mut sorted_defenders: Vec<_> = state.defense.keys().collect();
            sorted_defenders.sort();
            for defender in sorted_defenders {
                let target = state.defense.get(defender).unwrap();
                self.push_str(&self.indent());
                self.push_str(defender);
                self.push_str(" ");
                self.push_str(&self.format_defense_target(target, "="));
                self.push_str(",\n");
            }

            self.indent_level -= 1;
            self.push_str(&self.indent());
            self.push_str("},\n");
        }

        self.indent_level -= 1;
        self.push_str("}\n\n");
    }

    fn format_actions(&mut self, actions: Vec<Action>) {
        if actions.is_empty() {
            return;
        }

        if actions.len() == 1 {
            self.push_str("action = {\n");
            self.indent_level += 1;
            self.format_action_block(&actions[0]);
            self.indent_level -= 1;
            self.push_str("}\n");
        } else {
            self.push_str("actions = [\n");
            self.indent_level += 1;
            let len = actions.len();
            for (i, action) in actions.iter().enumerate() {
                self.push_str(&self.indent());
                self.push_str("action = {\n");
                self.indent_level += 1;
                self.format_action_block(action);
                self.indent_level -= 1;
                self.push_str(&self.indent());
                self.push_str("}");
                if i < len - 1 {
                    self.push_str(",");
                }
                self.newline();
            }
            self.indent_level -= 1;
            self.push_str("]\n");
        }
    }

    fn format_action_block(&mut self, action: &Action) {
        self.format_moves(&action.moves);
        self.format_screens(&action.screens);
        self.format_passes(&action.passes);
        self.format_defenses(&action.defenses);
    }

    fn format_moves(&mut self, moves: &[MoveAction]) {
        if moves.is_empty() {
            return;
        }

        self.push_str(&self.indent());
        self.push_str("move = {\n");
        self.indent_level += 1;
        for m in moves {
            self.flush_comments(m.span.start);
            self.push_str(&self.indent());
            self.push_str(&m.player);
            self.push_str(" ");
            let path_str = self.format_path_type(&m.path_type);
            self.push_str(&path_str);
            self.push_str(&format!(" ({}, {}),\n", m.target.0, m.target.1));
        }
        self.indent_level -= 1;
        self.push_str(&self.indent());
        self.push_str("},\n");
    }

    fn format_screens(&mut self, screens: &[ScreenAction]) {
        if screens.is_empty() {
            return;
        }

        self.push_str(&self.indent());
        self.push_str("screen = {\n");
        self.indent_level += 1;
        for s in screens {
            self.flush_comments(s.span.start);
            self.push_str(&self.indent());
            self.push_str(&s.player);
            self.push_str(" ");
            let path_str = self.format_path_type(&s.path_type);
            self.push_str(&path_str);
            self.push_str(" ");
            match &s.target {
                ScreenTarget::Player(p) => self.push_str(p),
                ScreenTarget::Coordinate(x, y) => self.push_str(&format!("({}, {})", x, y)),
            }
            let timing_str = self.format_timing(&s.timing);
            self.push_str(&timing_str);
            self.push_str(",\n");
        }
        self.indent_level -= 1;
        self.push_str(&self.indent());
        self.push_str("},\n");
    }

    fn format_passes(&mut self, passes: &[PassAction]) {
        if passes.is_empty() {
            return;
        }

        self.push_str(&self.indent());
        self.push_str("pass = {\n");
        self.indent_level += 1;
        for p in passes {
            self.flush_comments(p.span.start);
            self.push_str(&self.indent());
            self.push_str(&p.from);
            self.push_str(" -> ");
            self.push_str(&p.to);
            let timing_str = self.format_timing(&p.timing);
            self.push_str(&timing_str);
            self.push_str(",\n");
        }
        self.indent_level -= 1;
        self.push_str(&self.indent());
        self.push_str("},\n");
    }

    fn format_defenses(&mut self, defenses: &[DefenseAction]) {
        if defenses.is_empty() {
            return;
        }

        self.push_str(&self.indent());
        self.push_str("defense = {\n");
        self.indent_level += 1;
        for d in defenses {
            self.flush_comments(d.span.start);
            self.push_str(&self.indent());
            self.push_str(&d.defender);
            self.push_str(" ");
            self.push_str(&self.format_defense_target(&d.target, "->"));
            self.push_str(",\n");
        }
        self.indent_level -= 1;
        self.push_str(&self.indent());
        self.push_str("},\n");
    }

    fn format_path_type(&self, path_type: &PathType) -> String {
        match path_type {
            PathType::Straight => "->".to_string(),
            PathType::Curve(dir) => match dir {
                CurveDirection::Left(f) => format!("~[l:{}]>", f),
                CurveDirection::Right(f) => format!("~[r:{}]>", f),
            },
        }
    }

    fn format_timing(&self, timing: &Timing) -> String {
        match timing {
            Timing::Before => ":before".to_string(),
            Timing::After => ":after".to_string(),
            Timing::Middle => ":middle".to_string(),
            Timing::None => "".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_basic() {
        let input = "players={p1,p2}state={baller=p1,position={p1=(0,0),p2=(10,10)}}";
        let expected = r#"players = { p1, p2 }

state = {
  baller = p1,
  position = {
    p1 = (0, 0),
    p2 = (10, 10),
  },
}

"#;
        assert_eq!(format(input), expected);
    }

    #[test]
    fn test_format_single_action() {
        let input = "action={move={p1->(10,10)}}";
        let expected = r#"state = {
}

action = {
  move = {
    p1 -> (10, 10),
  },
}
"#;
        assert_eq!(format(input), expected);
    }

    #[test]
    fn test_format_multiple_actions() {
        let input = "actions=[action={move={p1->(10,10)}},action={pass={p1->p2:after}}]";
        let expected = r#"state = {
}

actions = [
  action = {
    move = {
      p1 -> (10, 10),
    },
  },
  action = {
    pass = {
      p1 -> p2:after,
    },
  }
]
"#;
        assert_eq!(format(input), expected);
    }

    #[test]
    fn test_format_comments() {
        let input = "// Header\nplayers = { p1 }\n// Middle\naction = { move = { p1 -> (0,0) } }";
        let expected = r#"// Header
players = { p1 }

state = {
}

action = {
  move = {
    // Middle
    p1 -> (0, 0),
  },
}
"#;
        assert_eq!(format(input), expected);
    }

    #[test]
    fn test_idempotency() {
        let input = "players={p1}state={baller=p1}action={move={p1->(1,1)}}";
        let first_pass = format(input);
        let second_pass = format(&first_pass);
        assert_eq!(first_pass, second_pass, "Formatting should be idempotent");
    }

    #[test]
    fn test_parse_error_handling() {
        let invalid = "players = { !!!invalid";
        assert_eq!(format(invalid), invalid);
        assert!(format_checked(invalid).is_err());
    }

    #[test]
    fn test_format_checked_ok_on_valid_input() {
        let input = "action={move={p1->(10,10)}}";
        assert!(format_checked(input).is_ok());
    }

    #[test]
    fn test_format_defenders_and_defense() {
        let input = "players={p1}defenders={d1,d2}state={position={p1=(0,60)}defense={d1->p1,d2=(-90,-80)}}action={defense={d1-[5]>p1,d2->(70,20)}}";
        let expected = r#"players = { p1 }

defenders = { d1, d2 }

state = {
  position = {
    p1 = (0, 60),
  },
  defense = {
    d1 -[10]> p1,
    d2 = (-90, -80),
  },
}

action = {
  defense = {
    d1 -[5]> p1,
    d2 -> (70, 20),
  },
}
"#;
        assert_eq!(format(input), expected);
    }

    #[test]
    fn test_format_defense_idempotency() {
        let input = "defenders={d1}state={defense={d1->p1}}action={defense={d1-[3]>p1,d2->(1,1)}}";
        let first_pass = format(input);
        let second_pass = format(&first_pass);
        assert_eq!(first_pass, second_pass, "Formatting should be idempotent");
    }
}
