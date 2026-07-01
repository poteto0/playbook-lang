use crate::geometry::normalize;
use crate::ir::*;
use crate::parser::ParseError;
use std::fmt::Write;

/// Build a single error object as a JSON string using `serde_json` so that
/// `message` (which may contain `"`, `\`, etc.) is always safely escaped.
fn error_json(line: usize, column: usize, length: usize, message: &str) -> String {
    serde_json::json!({
        "line": line,
        "column": column,
        "length": length,
        "message": message,
    })
    .to_string()
}

/// Format parser errors into the `[Error]:[...]` envelope expected by the frontend.
fn format_parse_errors(errors: &[ParseError]) -> String {
    let error_jsons: Vec<String> = errors
        .iter()
        .map(|e| match e {
            ParseError::UnexpectedToken(token, msg) | ParseError::InvalidSyntax(token, msg) => {
                error_json(token.span.line, token.span.column, token.span.len(), msg)
            }
            ParseError::UnexpectedEOF => error_json(0, 0, 0, "Unexpected end of file"),
        })
        .collect();

    format!("[Error]:[{}]", error_jsons.join(", "))
}

/// Format an IR generation error into the `[Error]:[...]` envelope.
fn format_ir_error(e: &IRError) -> String {
    let json = match e {
        IRError::UnexpectedPlayer(span, name) => error_json(
            span.line,
            span.column,
            span.len(),
            &format!("Player '{}' not found in state", name),
        ),
        IRError::PlayerNotBaller(span, name) => error_json(
            span.line,
            span.column,
            span.len(),
            &format!("Player '{}' does not have the ball", name),
        ),
    };
    format!("[Error]:[{}]", json)
}

/// Escape characters that are unsafe in XML/SVG text and attribute values.
fn escape_xml(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(c),
        }
    }
    out
}

pub struct Renderer {
    width: u32,
    height: u32,
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer {
    pub fn play(&self, input: &str) -> Result<String, String> {
        use crate::ir::IRGenerator;
        use crate::lexer::Lexer;
        use crate::parser::Parser;

        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);

        let (playbook, errors) = parser.parse();

        if !errors.is_empty() {
            return Err(format_parse_errors(&errors));
        }

        match IRGenerator::generate(playbook) {
            Ok(scene) => {
                let mut output = String::new();

                let (players, defenders): (Vec<&Entity>, Vec<&Entity>) = scene
                    .entities
                    .iter()
                    .partition(|e| e.kind == EntityKind::Player);

                // players
                let player_ids: Vec<&str> = players.iter().map(|e| e.id.as_str()).collect();
                output.push_str(&format!("players = {{ {} }}\n\n", player_ids.join(", ")));

                if !defenders.is_empty() {
                    let defender_ids: Vec<&str> = defenders.iter().map(|e| e.id.as_str()).collect();
                    output.push_str(&format!(
                        "defenders = {{ {} }}\n\n",
                        defender_ids.join(", ")
                    ));
                }

                // state
                output.push_str("state = {\n");
                if let Some(baller) = &scene.final_baller {
                    output.push_str(&format!("  baller = {},\n", baller));
                }

                output.push_str("  position = {\n");
                for entity in &players {
                    output.push_str(&format!(
                        "    {} = ({}, {}),\n",
                        entity.id, entity.end_pos.0, entity.end_pos.1
                    ));
                }
                output.push_str("  },\n");

                if !defenders.is_empty() {
                    output.push_str("  defense = {\n");
                    for entity in &defenders {
                        output.push_str(&format!(
                            "    {} = ({}, {}),\n",
                            entity.id, entity.end_pos.0, entity.end_pos.1
                        ));
                    }
                    output.push_str("  },\n");
                }
                output.push_str("}\n");

                Ok(output)
            }
            Err(e) => Err(format_ir_error(&e)),
        }
    }

    pub fn new() -> Self {
        Self {
            width: 500,
            height: 500,
        }
    }

    pub fn render_scene(&self, scene: &Scene) -> String {
        let mut svg = String::new();
        svg.push_str(&self.render_court());

        // 1. Draw Interactions
        for (i, interaction) in scene.interactions.iter().enumerate() {
            match interaction {
                Interaction::Move(m) => {
                    let is_last = self.is_last_move(scene, i, &m.player_id);
                    svg.push_str(&self.render_move(m, is_last));
                }
                Interaction::Pass(p) => {
                    svg.push_str(&self.render_pass(p));
                }
                Interaction::Screen(s) => {
                    svg.push_str(&self.render_screen(s));
                }
                Interaction::Defense(d) => {
                    let is_last = self.is_last_move(scene, i, &d.defender_id);
                    svg.push_str(&self.render_defense(d, is_last));
                }
            }
        }

        // 2. Draw Entities
        for entity in &scene.entities {
            svg.push_str(&self.render_player(entity));
        }

        svg.push_str("<defs><marker id=\"arrowhead\" markerWidth=\"10\" markerHeight=\"7\" refX=\"10\" refY=\"3.5\" orient=\"auto\"><polygon points=\"0 0, 10 3.5, 0 7\" fill=\"black\" /></marker></defs>");
        svg.push_str("</svg>");
        svg
    }

    fn render_court(&self) -> String {
        let mut court = String::new();
        let _ = write!(
            &mut court,
            "<svg width=\"{}\" height=\"{}\" viewBox=\"-105 -105 210 210\" xmlns=\"http://www.w3.org/2000/svg\">",
            self.width, self.height
        );

        // 0. Global Background (White fill for everything)
        court
            .push_str("<rect x=\"-105\" y=\"-105\" width=\"210\" height=\"210\" fill=\"white\" />");

        // 1. Court Boundary (Half court)
        // Black border. Covers the half court area. Fill is already white from background, but keeping fill=\"white\" ensures opacity if layers change.
        court.push_str("<rect x=\"-100\" y=\"-90\" width=\"200\" height=\"180\" fill=\"white\" stroke=\"black\" stroke-width=\"2\" />");

        // 2. Key area (Rectangle)
        court.push_str("<rect x=\"-20\" y=\"-90\" width=\"40\" height=\"65\" fill=\"none\" stroke=\"black\" stroke-width=\"1\" />");

        // 3. Free-throw circle
        court.push_str("<circle cx=\"0\" cy=\"-25\" r=\"20\" fill=\"none\" stroke=\"black\" stroke-width=\"1\" />");

        // 4. 3-point line (Straight lines + Arc)
        // Straight lines from baseline (y=-90) to y=-35 at x=+/-80.
        // Arc connects (-80, -35) to (80, -35). Sweep-flag=0 makes it curve downwards (towards Y+).
        court.push_str("<path d=\"M -80 -90 L -80 -35 A 80 80 0 0 0 80 -35 L 80 -90\" fill=\"none\" stroke=\"black\" stroke-width=\"1\" />");

        // 5. Center Circle (Half) at the opposite side (y=90)
        court.push_str("<path d=\"M -20 90 A 20 20 0 0 1 20 90\" fill=\"none\" stroke=\"black\" stroke-width=\"1\" />");

        // 6. Backboard
        court.push_str("<line x1=\"-12\" y1=\"-88\" x2=\"12\" y2=\"-88\" stroke=\"black\" stroke-width=\"1\" />");

        // 7. Hoop (Red)
        court.push_str("<circle cx=\"0\" cy=\"-84\" r=\"5\" stroke=\"red\" stroke-width=\"1\" fill=\"none\" />");

        court
    }

    fn render_move(&self, m: &MoveLine, draw_arrow: bool) -> String {
        let marker = if draw_arrow {
            " marker-end=\"url(#arrowhead)\""
        } else {
            ""
        };

        if m.is_dribble && m.curve.is_none() {
            // Only support straight dribble for now
            return self.render_dribble(m, draw_arrow);
        }

        match &m.curve {
            Some(dir) => {
                let (cx, cy) = self.calculate_control_point(m.from, m.to, dir);
                format!(
                    "<path d=\"M {} {} Q {} {} {} {}\" stroke=\"black\" stroke-width=\"2\" fill=\"none\"{} />",
                    m.from.0, m.from.1, cx, cy, m.to.0, m.to.1, marker
                )
            }
            None => self.render_straight_line(m.from, m.to, draw_arrow),
        }
    }

    /// A plain straight `<line>` between two points, with an optional
    /// arrowhead marker at the end. Shared by straight moves and defense
    /// movement lines.
    fn render_straight_line(&self, from: (f64, f64), to: (f64, f64), draw_arrow: bool) -> String {
        let marker = if draw_arrow {
            " marker-end=\"url(#arrowhead)\""
        } else {
            ""
        };
        format!(
            "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"black\" stroke-width=\"2\"{} />",
            from.0, from.1, to.0, to.1, marker
        )
    }

    fn render_dribble(&self, m: &MoveLine, draw_arrow: bool) -> String {
        let dx = m.to.0 - m.from.0;
        let dy = m.to.1 - m.from.1;

        let Some((ux, uy, dist)) = normalize(dx, dy, 1.0) else {
            return "".to_string(); // Too short
        };

        // Perpendicular vector
        let px = -uy;
        let py = ux;

        // Configuration
        let margin_ratio = 0.1; // 10% linear segment at each end
        let amplitude = 5.0; // Wider waves
        let step_size = 5.0; // Smaller period (finer waves)

        let start_wavy_dist = dist * margin_ratio;
        let end_wavy_dist = dist * (1.0 - margin_ratio);
        let wavy_dist = end_wavy_dist - start_wavy_dist;

        let steps = (wavy_dist / step_size).ceil().max(2.0) as usize;

        // Start with linear segment
        let wavy_start_x = m.from.0 + ux * start_wavy_dist;
        let wavy_start_y = m.from.1 + uy * start_wavy_dist;
        let mut path_data = format!(
            "M {} {} L {} {}",
            m.from.0, m.from.1, wavy_start_x, wavy_start_y
        );

        // Wavy middle part
        for i in 0..steps {
            let t2 = (i as f64 + 0.5) / steps as f64;
            let t3 = (i + 1) as f64 / steps as f64;

            let mid_dist = start_wavy_dist + wavy_dist * t2;
            let mid_x = m.from.0 + ux * mid_dist;
            let mid_y = m.from.1 + uy * mid_dist;

            // Wavy effect
            let offset = if i % 2 == 0 { amplitude } else { -amplitude };
            let cp_x = mid_x + px * offset;
            let cp_y = mid_y + py * offset;

            let end_dist = start_wavy_dist + wavy_dist * t3;
            let end_x = m.from.0 + ux * end_dist;
            let end_y = m.from.1 + uy * end_dist;

            let _ = write!(&mut path_data, " Q {} {} {} {}", cp_x, cp_y, end_x, end_y);
        }

        // End with linear segment
        let _ = write!(&mut path_data, " L {} {}", m.to.0, m.to.1);

        let marker = if draw_arrow {
            " marker-end=\"url(#arrowhead)\""
        } else {
            ""
        };

        format!(
            "<path d=\"{}\" stroke=\"black\" stroke-width=\"2\" fill=\"none\"{} />",
            path_data, marker
        )
    }

    /// Bezier curve
    fn calculate_control_point(
        &self,
        from: (f64, f64),
        to: (f64, f64),
        dir: &crate::ast::CurveDirection,
    ) -> (f64, f64) {
        let dx = to.0 - from.0;
        let dy = to.1 - from.1;
        let mid_x = (from.0 + to.0) / 2.0;
        let mid_y = (from.1 + to.1) / 2.0;

        let (nx, ny, factor) = match dir {
            crate::ast::CurveDirection::Left(f) => (dy, -dx, *f),
            crate::ast::CurveDirection::Right(f) => (-dy, dx, *f),
        };

        (mid_x + nx * factor, mid_y + ny * factor)
    }

    fn render_defense(&self, d: &DefenseLine, draw_arrow: bool) -> String {
        self.render_straight_line(d.from, d.to, draw_arrow)
    }

    fn render_pass(&self, p: &PassLine) -> String {
        format!(
            "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"black\" stroke-width=\"2\" stroke-dasharray=\"4\" marker-end=\"url(#arrowhead)\" />",
            p.from.0, p.from.1, p.to.0, p.to.1
        )
    }

    fn render_screen(&self, s: &ScreenLine) -> String {
        let dx = s.to.0 - s.from.0;
        let dy = s.to.1 - s.from.1;

        // Normalized direction. Default to (0, 1) (downward) if stationary.
        let (nx, ny) = normalize(dx, dy, 0.001)
            .map(|(ux, uy, _)| (ux, uy))
            .unwrap_or((0.0, 1.0));

        // Offset the screen position slightly towards the screener (from)
        let shift_amount = 5.0;
        let cx = s.to.0 - nx * shift_amount;
        let cy = s.to.1 - ny * shift_amount;

        // Perpendicular vector (-y, x)
        let px = -ny;
        let py = nx;

        let bar_len = 15.0;
        let half_bar = bar_len / 2.0;

        // Coordinates for the perpendicular bar centered at (cx, cy)
        let bx1 = cx - px * half_bar;
        let by1 = cy - py * half_bar;
        let bx2 = cx + px * half_bar;
        let by2 = cy + py * half_bar;

        let mut svg = String::new();

        match &s.curve {
            Some(dir) => {
                let (cpx, cpy) = self.calculate_control_point(s.from, (cx, cy), dir);
                let _ = write!(
                    &mut svg,
                    "<path d=\"M {} {} Q {} {} {} {}\" stroke=\"black\" stroke-width=\"2\" fill=\"none\" />",
                    s.from.0, s.from.1, cpx, cpy, cx, cy
                );
            }
            None => {
                let _ = write!(
                    &mut svg,
                    "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"black\" stroke-width=\"2\" />",
                    s.from.0, s.from.1, cx, cy
                );
            }
        }

        // Draw the perpendicular bar
        let _ = write!(
            &mut svg,
            "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"black\" stroke-width=\"2\" />",
            bx1, by1, bx2, by2
        );

        svg
    }

    fn is_last_move(&self, scene: &Scene, current_idx: usize, player_id: &str) -> bool {
        for i in (current_idx + 1)..scene.interactions.len() {
            match &scene.interactions[i] {
                Interaction::Move(m) if m.player_id == player_id => return false,
                Interaction::Screen(s) if s.screener_id == player_id => return false,
                Interaction::Defense(d) if d.defender_id == player_id => return false,
                _ => {}
            }
        }
        true
    }

    fn render_player(&self, entity: &Entity) -> String {
        let mut player = String::new();
        let _ = write!(
            &mut player,
            "<circle cx=\"{}\" cy=\"{}\" r=\"10\" fill=\"white\" stroke=\"black\" stroke-width=\"2\" />",
            entity.start_pos.0, entity.start_pos.1
        );
        let _ = write!(
            &mut player,
            "<text x=\"{}\" y=\"{}\" font-size=\"12\" text-anchor=\"middle\" dominant-baseline=\"central\" font-family=\"Arial\">{}</text>",
            entity.start_pos.0, entity.start_pos.1, escape_xml(&entity.label)
        );

        if entity.is_baller {
            let _ = write!(
                &mut player,
                "<circle cx=\"{}\" cy=\"{}\" r=\"4\" fill=\"orange\" stroke=\"black\" stroke-width=\"1\" transform=\"translate(10, -10)\" />",
                entity.start_pos.0, entity.start_pos.1
            );
        }

        player
    }

    pub fn render(&self, input: &str) -> Result<String, String> {
        use crate::ir::IRGenerator;
        use crate::lexer::Lexer;
        use crate::parser::Parser;

        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);

        let (playbook, errors) = parser.parse();

        if !errors.is_empty() {
            return Err(format_parse_errors(&errors));
        }

        match IRGenerator::generate(playbook) {
            Ok(scene) => Ok(self.render_scene(&scene)),
            Err(e) => Err(format_ir_error(&e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_pipeline() {
        let renderer = Renderer::new();
        let input = r#"            players = { p1, p2 }
            state = { baller = p1, position = { p1 = (0, 0), p2 = (50, 50) } }
            actions = [
                action = { 
                    move = { p2 -> (0, 50) },
                    screen = { p1 -> (25, 25) }
                }
            ]
        "#;
        let output = renderer.render(input).expect("Failed to render");
        assert!(output.contains("<svg"));
        assert!(output.contains("circle"));
        // Player 2 is at (50, 50) initially, moved to (0, 50).
        // Label should be at start_pos (50, 50) now.
        assert!(output.contains("x=\"50\" y=\"50\""));
        assert!(output.contains(">2<"));
        // Player 1 is at (0, 0) and is baller.
        assert!(output.contains("x=\"0\" y=\"0\""));
        assert!(output.contains(">1<"));
        // Ball (orange circle) should be at p1's start_pos (0, 0)
        assert!(output.contains("fill=\"orange\""));
        // Check for screen rendering elements (perpendicular bar)
        // Screen rendering uses black stroke width 2 lines
        assert!(output.contains("stroke=\"black\" stroke-width=\"2\""));
    }

    #[test]
    fn test_render_defender() {
        let renderer = Renderer::new();
        let input = r#"
            players = { p1 }
            defenders = { d1 }
            state = {
                position = { p1 = (0, 60) },
                defense = { d1 -> p1 },
            }
            action = {
                defense = { d1 -[5]> p1 },
            }
        "#;
        let output = renderer.render(input).expect("Failed to render");
        // Defender label "x" is drawn, without a ball marker.
        assert!(output.contains(">x<"));
        // A defense movement line is drawn between the two marked positions.
        assert!(output.contains(
            "<line x1=\"0\" y1=\"50\" x2=\"0\" y2=\"55\" stroke=\"black\" stroke-width=\"2\" marker-end=\"url(#arrowhead)\" />"
        ));
    }

    #[test]
    fn test_play_reports_defenders_separately_from_players() {
        let renderer = Renderer::new();
        let input = r#"
            players = { p1 }
            defenders = { d1 }
            state = {
                position = { p1 = (0, 60) },
                defense = { d1 -> p1 },
            }
        "#;
        let output = renderer.play(input).expect("Failed to play");
        assert!(output.contains("players = { p1 }"));
        assert!(output.contains("defenders = { d1 }"));
        assert!(!output.contains("players = { p1, d1 }"));
        assert!(output.contains("defense = {"));
    }

    #[test]
    fn test_error_reporting() {
        let renderer = Renderer::new();
        let input = "players = { "; // Missing closing brace
        let output = renderer.render(input).unwrap_err();
        assert!(output.contains("[Error]:["));
        // EOF handling is tricky to test specific line without knowing where EOF span lands,
        // but it should contain "Error" and likely "Expected `}`".
        assert!(output.contains("Expected `}`"));
    }

    #[test]
    fn test_typo_suggestion() {
        let renderer = Renderer::new();
        let input = "aciton = { }"; // typo: action
        let output = renderer.render(input).unwrap_err();
        assert!(output.contains("Did you mean 'action'?"));
    }

    #[test]
    fn test_escape_xml_escapes_unsafe_chars() {
        assert_eq!(
            escape_xml(r#"<script>&"'"#),
            "&lt;script&gt;&amp;&quot;&apos;"
        );
        assert_eq!(escape_xml("plain"), "plain");
    }

    #[test]
    fn test_render_player_escapes_label() {
        let renderer = Renderer::new();
        let entity = Entity {
            id: "p1".to_string(),
            kind: EntityKind::Player,
            label: "<tspan onload=\"x\">".to_string(),
            start_pos: (0.0, 0.0),
            end_pos: (0.0, 0.0),
            is_baller: false,
        };
        let svg = renderer.render_player(&entity);
        // The raw injection must not appear; it must be escaped instead.
        assert!(!svg.contains("<tspan"));
        assert!(svg.contains("&lt;tspan onload=&quot;x&quot;&gt;"));
    }

    #[test]
    fn test_error_message_is_valid_json() {
        // A lexer error message containing a `"` must be escaped so the
        // resulting `[Error]:[...]` payload stays parseable JSON.
        let renderer = Renderer::new();
        let input = r#"players = { ~[a"b] }"#;
        let err = renderer.render(input).unwrap_err();
        let json = err
            .strip_prefix("[Error]:[")
            .and_then(|s| s.strip_suffix("]"))
            .expect("error envelope");
        let parsed: serde_json::Value = serde_json::from_str(json).expect("valid JSON object");
        assert!(parsed.get("message").is_some());
    }
}
