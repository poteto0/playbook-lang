use playbook_lang_core::ast::{
    Action, ActionSection, CurveDirection, DefenseAction, DefenseTarget, MoveAction, PassAction,
    PathType, Playbook, ScreenAction, ScreenTarget, State, Timing,
};
use playbook_lang_core::lexer::{Lexer, Span};
use playbook_lang_core::parser::{ParseError, Parser};
use std::collections::HashMap;

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

    Ok(Formatter::format_with_comments(&playbook))
}

/// Pseudo anchor for comments that follow every formatted node; they are
/// written at the very end of the output.
const TRAILING: usize = usize::MAX;

#[derive(Default)]
struct Formatter {
    /// Comments keyed by the start of the node they precede.
    comments: HashMap<usize, Vec<String>>,
    /// Starts of every node that can carry leading comments, in visit order.
    anchors: Vec<usize>,
    output: String,
    indent_level: usize,
}

impl Formatter {
    /// Attaches each comment to the first node that follows it in the
    /// source, so comments move together with their node however the output
    /// reorders sections and entries. The anchors are taken from a dry run
    /// of the formatter itself, so they always match what it writes.
    fn format_with_comments(playbook: &Playbook) -> String {
        let mut dry_run = Formatter::default();
        let output = dry_run.format(playbook);
        if playbook.comments.is_empty() {
            return output;
        }

        let mut anchors = dry_run.anchors;
        anchors.sort_unstable();
        let mut comments: Vec<_> = playbook.comments.iter().collect();
        comments.sort_by_key(|(span, _)| span.start);

        let mut formatter = Formatter::default();
        for (span, comment) in comments {
            let next = anchors.partition_point(|&anchor| anchor <= span.start);
            let anchor = anchors.get(next).copied().unwrap_or(TRAILING);
            formatter
                .comments
                .entry(anchor)
                .or_default()
                .push(comment.clone());
        }
        formatter.format(playbook)
    }

    fn format(&mut self, playbook: &Playbook) -> String {
        let spans = playbook.section_spans;

        self.format_identifier_list("players", spans.players, &playbook.players);
        self.format_identifier_list("defenders", spans.defenders, &playbook.defenders);
        self.format_state(spans.state, &playbook.state);
        self.format_actions(&playbook.action_sections, &playbook.actions);

        self.write_comments_at(TRAILING);
        std::mem::take(&mut self.output)
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

    /// Registers `span` as an anchor and writes the comments attached to it;
    /// a no-op when the node was absent from the source (`None`).
    fn write_leading_comments(&mut self, span: impl Into<Option<Span>>) {
        if let Some(span) = span.into() {
            self.write_comments_at(span.start);
        }
    }

    fn write_comments_at(&mut self, anchor: usize) {
        self.anchors.push(anchor);
        for comment in self.comments.remove(&anchor).unwrap_or_default() {
            self.push_str(&self.indent());
            self.push_str("// ");
            self.push_str(&comment);
            self.newline();
        }
    }

    /// Formats a `players` / `defenders` section.
    fn format_identifier_list(&mut self, keyword: &str, span: Option<Span>, ids: &[String]) {
        if ids.is_empty() {
            return;
        }

        self.write_leading_comments(span);
        self.push_str(keyword);
        self.push_str(" = { ");
        self.push_str(&ids.join(", "));
        self.push_str(" }\n\n");
    }

    /// Formats a single `defense` entry's target: a fixed position (using
    /// `coord_op`, which differs between `state` (`=`) and `action` (`->`))
    /// or a player mark, always spelled out explicitly (`-[offset]>`).
    fn format_defense_target(&self, target: &DefenseTarget, coord_op: &str) -> String {
        match target {
            DefenseTarget::Position(x, y) => format!("{} ({}, {})", coord_op, x, y),
            DefenseTarget::Mark {
                player,
                offset,
                timing,
            } => format!("-[{}]> {}{}", offset, player, self.format_timing(timing)),
        }
    }

    fn format_state(&mut self, span: Option<Span>, state: &State) {
        self.write_leading_comments(span);
        self.push_str("state = {\n");
        self.indent_level += 1;

        if let Some(ref baller) = state.baller {
            self.write_leading_comments(state.baller_span);
            self.push_str(&self.indent());
            self.push_str("baller = ");
            self.push_str(baller);
            self.push_str(",\n");
        }

        let mut positions: Vec<_> = state.positions.iter().collect();
        positions.sort_by_key(|(player, _)| *player);
        if !positions.is_empty() {
            self.write_leading_comments(state.position_span);
        }
        self.format_block("position", &positions, |this, (player, ((x, y), span))| {
            this.write_leading_comments(*span);
            this.push_str(&this.indent());
            this.push_str(&format!("{} = ({}, {}),\n", player, x, y));
        });

        let mut defense: Vec<_> = state.defense.iter().collect();
        defense.sort_by_key(|(defender, _)| *defender);
        if !defense.is_empty() {
            self.write_leading_comments(state.defense_span);
        }
        self.format_block("defense", &defense, |this, (defender, (target, span))| {
            this.write_leading_comments(*span);
            this.push_str(&this.indent());
            this.push_str(defender);
            this.push_str(" ");
            let target_str = this.format_defense_target(target, "=");
            this.push_str(&target_str);
            this.push_str(",\n");
        });

        self.indent_level -= 1;
        self.push_str("}\n\n");
    }

    /// Writes each `action` / `actions` section back in its original form,
    /// rather than merging them, which could change their meaning.
    fn format_actions(&mut self, sections: &[ActionSection], actions: &[Action]) {
        // A hand-built `Playbook` may carry actions without sections.
        let fallback;
        let sections = if sections.is_empty() && !actions.is_empty() {
            fallback = [ActionSection {
                span: actions[0].span,
                list: actions.len() > 1,
                range: 0..actions.len(),
            }];
            &fallback[..]
        } else {
            sections
        };

        for (i, section) in sections.iter().enumerate() {
            let phases = &actions[section.range.clone()];
            if i > 0 {
                self.newline();
            }
            self.write_leading_comments(section.span);
            if section.list {
                self.push_str("actions = [\n");
                self.indent_level += 1;
                for (j, action) in phases.iter().enumerate() {
                    self.write_leading_comments(action.span);
                    self.format_phase(action);
                    if j < phases.len() - 1 {
                        self.push_str(",");
                    }
                    self.newline();
                }
                self.indent_level -= 1;
                self.push_str("]\n");
            } else {
                self.format_phase(&phases[0]);
                self.newline();
            }
        }
    }

    /// Writes one `action = { ... }`, without a trailing newline.
    fn format_phase(&mut self, action: &Action) {
        self.push_str(&self.indent());
        self.push_str("action = {\n");
        self.indent_level += 1;
        self.format_action_block(action);
        self.indent_level -= 1;
        self.push_str(&self.indent());
        self.push_str("}");
    }

    fn format_action_block(&mut self, action: &Action) {
        self.format_moves(&action.moves);
        self.format_screens(&action.screens);
        self.format_passes(&action.passes);
        self.format_defenses(&action.defenses);
    }

    /// Shared scaffold for a braced property block (`move`, `screen`,
    /// `pass`, `defense`, and `state`'s `position` / `defense`): an empty check, the `<header> = {` line, one
    /// indented line per item (each preceded by any comments that belong
    /// before it and followed by a trailing comma), and the closing `},`.
    /// `format_entry` writes just the entry's own line content (the part
    /// between `self.indent()` and the trailing `,\n`, exclusive).
    fn format_block<T>(
        &mut self,
        header: &str,
        items: &[T],
        mut format_entry: impl FnMut(&mut Self, &T),
    ) {
        if items.is_empty() {
            return;
        }

        self.push_str(&self.indent());
        self.push_str(header);
        self.push_str(" = {\n");
        self.indent_level += 1;
        for item in items {
            format_entry(self, item);
        }
        self.indent_level -= 1;
        self.push_str(&self.indent());
        self.push_str("},\n");
    }

    fn format_moves(&mut self, moves: &[MoveAction]) {
        self.format_block("move", moves, |this, m| {
            this.write_leading_comments(m.span);
            this.push_str(&this.indent());
            this.push_str(&m.player);
            this.push_str(" ");
            let path_str = this.format_path_type(&m.path_type);
            this.push_str(&path_str);
            this.push_str(&format!(" ({}, {}),\n", m.target.0, m.target.1));
        });
    }

    fn format_screens(&mut self, screens: &[ScreenAction]) {
        self.format_block("screen", screens, |this, s| {
            this.write_leading_comments(s.span);
            this.push_str(&this.indent());
            this.push_str(&s.player);
            this.push_str(" ");
            let path_str = this.format_path_type(&s.path_type);
            this.push_str(&path_str);
            this.push_str(" ");
            match &s.target {
                ScreenTarget::Player(p) => this.push_str(p),
                ScreenTarget::Coordinate(x, y) => this.push_str(&format!("({}, {})", x, y)),
            }
            let timing_str = this.format_timing(&s.timing);
            this.push_str(&timing_str);
            this.push_str(",\n");
        });
    }

    fn format_passes(&mut self, passes: &[PassAction]) {
        self.format_block("pass", passes, |this, p| {
            this.write_leading_comments(p.span);
            this.push_str(&this.indent());
            this.push_str(&p.from);
            this.push_str(" -> ");
            this.push_str(&p.to);
            let timing_str = this.format_timing(&p.timing);
            this.push_str(&timing_str);
            this.push_str(",\n");
        });
    }

    fn format_defenses(&mut self, defenses: &[DefenseAction]) {
        self.format_block("defense", defenses, |this, d| {
            this.write_leading_comments(d.span);
            this.push_str(&this.indent());
            this.push_str(&d.defender);
            this.push_str(" ");
            let target_str = this.format_defense_target(&d.target, "->");
            this.push_str(&target_str);
            this.push_str(",\n");
        });
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

// Middle
action = {
  move = {
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
    d1 -[20]> p1,
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
    fn test_format_defenders_flushes_leading_comment() {
        // With no `players` section, `format_defenders` must flush a
        // top-of-file comment itself, the same way `format_players` does,
        // instead of leaving it queued until the end of the output.
        let input = "// setup notes\ndefenders = { d1 }\nstate = { defense = { d1 = (0, 0) } }";
        let expected = r#"// setup notes
defenders = { d1 }

state = {
  defense = {
    d1 = (0, 0),
  },
}

"#;
        assert_eq!(format(input), expected);
    }

    #[test]
    fn test_format_comment_before_action_stays_in_place() {
        // Regression for #86: a comment right before `action = {` used to be
        // hoisted above `players`.
        let input = "players = { p1 }\n// before action\naction = { move = { p1 -> (0,0) } }";
        let expected = r#"players = { p1 }

state = {
}

// before action
action = {
  move = {
    p1 -> (0, 0),
  },
}
"#;
        assert_eq!(format(input), expected);
    }

    #[test]
    fn test_format_state_comments_are_kept() {
        // Regression for #86: comments inside `state` used to be dropped from
        // their place (flushed later or hoisted). Entries are sorted by name,
        // and each comment must move together with its entry.
        let input = r#"players = { p1, p2 }
defenders = { d1, d2 }
// initial setup
state = {
  // who has the ball
  baller = p1,
  // spacing
  position = {
    // wing
    p2 = (10, 10),
    // top
    p1 = (0, 0),
  },
  defense = {
    // zone
    d2 = (5, 5),
    // man
    d1 -> p1,
  },
}
action = { move = { p1 -> (1, 1) } }"#;
        let expected = r#"players = { p1, p2 }

defenders = { d1, d2 }

// initial setup
state = {
  // who has the ball
  baller = p1,
  // spacing
  position = {
    // top
    p1 = (0, 0),
    // wing
    p2 = (10, 10),
  },
  defense = {
    // man
    d1 -[20]> p1,
    // zone
    d2 = (5, 5),
  },
}

action = {
  move = {
    p1 -> (1, 1),
  },
}
"#;
        let formatted = format(input);
        assert_eq!(formatted, expected);
        assert_eq!(
            format(&formatted),
            formatted,
            "Formatting should be idempotent"
        );
    }

    #[test]
    fn test_format_comment_before_each_action_in_list() {
        let input = "// phases\nactions = [\n// first\naction = { move = { p1 -> (0,0) } },\n// second\naction = { pass = { p1 -> p2 } }\n]";
        let expected = r#"state = {
}

// phases
actions = [
  // first
  action = {
    move = {
      p1 -> (0, 0),
    },
  },
  // second
  action = {
    pass = {
      p1 -> p2,
    },
  }
]
"#;
        assert_eq!(format(input), expected);
    }

    #[test]
    fn test_format_comments_follow_reordered_action_blocks() {
        // `pass` is written after `move`, so its comment must move with it.
        let input = "action = {\n  pass = {\n    // hand off\n    p1 -> p2,\n  },\n  move = {\n    // cut\n    p2 -> (0,0),\n  },\n}\n// end";
        let expected = r#"state = {
}

action = {
  move = {
    // cut
    p2 -> (0, 0),
  },
  pass = {
    // hand off
    p1 -> p2,
  },
}
// end
"#;
        assert_eq!(format(input), expected);
    }

    #[test]
    fn test_format_keeps_action_sections_separate() {
        // Regression for #87: separate `action` sections used to be merged
        // into one `actions = [...]` list.
        let input = "action={move={p1->(1,1)}}\n// second\naction={move={p1->(2,2)}}";
        let expected = r#"state = {
}

action = {
  move = {
    p1 -> (1, 1),
  },
}

// second
action = {
  move = {
    p1 -> (2, 2),
  },
}
"#;
        let formatted = format(input);
        assert_eq!(formatted, expected);
        assert_eq!(format(&formatted), formatted);
    }

    #[test]
    fn test_format_keeps_mixed_action_sections() {
        let input = "actions=[action={move={p1->(1,1)}}]action={pass={p1->p2}}";
        let expected = r#"state = {
}

actions = [
  action = {
    move = {
      p1 -> (1, 1),
    },
  }
]

action = {
  pass = {
    p1 -> p2,
  },
}
"#;
        let formatted = format(input);
        assert_eq!(formatted, expected);
        assert!(format_checked(&formatted).is_ok());
    }

    #[test]
    fn test_format_defense_idempotency() {
        let input = "defenders={d1}state={defense={d1->p1}}action={defense={d1-[3]>p1,d2->(1,1)}}";
        let first_pass = format(input);
        let second_pass = format(&first_pass);
        assert_eq!(first_pass, second_pass, "Formatting should be idempotent");
    }
}
