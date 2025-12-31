use crate::ir::*;

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
        for interaction in &scene.interactions {
            match interaction {
                Interaction::Move(m) => {
                    svg.push_str(&self.render_move(m));
                }
                Interaction::Pass(p) => {
                    svg.push_str(&self.render_pass(p));
                }
                Interaction::Screen(s) => {
                    svg.push_str(&self.render_screen(s));
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
        court.push_str(&format!(
            "<svg width=\"{}\" height=\"{}\" viewBox=\"-105 -105 210 210\" xmlns=\"http://www.w3.org/2000/svg\">",
            self.width,
            self.height
        ));

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

    fn render_move(&self, m: &MoveLine) -> String {
        format!(
            "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"black\" stroke-width=\"2\" marker-end=\"url(#arrowhead)\" />",
            m.from.0, m.from.1, m.to.0, m.to.1
        )
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
        let len = (dx * dx + dy * dy).sqrt();

        // Normalized direction. Default to (0, 1) (downward) if stationary.
        let (nx, ny) = if len > 0.001 {
            (dx / len, dy / len)
        } else {
            (0.0, 1.0)
        };

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
        // Draw the movement line (stem) to the shifted center
        svg.push_str(&format!(
            "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"black\" stroke-width=\"2\" />",
            s.from.0, s.from.1, cx, cy
        ));

        // Draw the perpendicular bar
        svg.push_str(&format!(
            "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"black\" stroke-width=\"2\" />",
            bx1, by1, bx2, by2
        ));

        svg
    }

    fn render_player(&self, entity: &Entity) -> String {
        let mut player = String::new();
        player.push_str(&format!(
                "<circle cx=\"{}\" cy=\"{}\" r=\"8\" fill=\"white\" stroke=\"gray\" stroke-width=\"1\" opacity=\"0.3\" />",
                entity.start_pos.0, entity.start_pos.1
            ));
        player.push_str(&format!(
                "<circle cx=\"{}\" cy=\"{}\" r=\"10\" fill=\"white\" stroke=\"black\" stroke-width=\"2\" />",
                entity.end_pos.0, entity.end_pos.1
            ));
        player.push_str(&format!(
                "<text x=\"{}\" y=\"{}\" font-size=\"12\" text-anchor=\"middle\" dominant-baseline=\"central\" font-family=\"Arial\">{}</text>",
                entity.end_pos.0, entity.end_pos.1, entity.label
            ));

        if entity.is_baller {
            player.push_str(&format!(
                    "<circle cx=\"{}\" cy=\"{}\" r=\"4\" fill=\"orange\" stroke=\"black\" stroke-width=\"1\" transform=\"translate(10, -10)\" />",
                    entity.end_pos.0, entity.end_pos.1
                ));
        }

        player
    }

    pub fn render(&self, input: &str) -> Result<String, String> {
        use crate::ir::IRGenerator;
        use crate::lexer::Lexer;
        use crate::parser::{ParseError, Parser};

        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);

        match parser.parse() {
            Ok(playbook) => {
                let scene = IRGenerator::generate(playbook);
                Ok(self.render_scene(&scene))
            }
            Err(e) => {
                let error_msg = match e {
                    ParseError::UnexpectedToken(token, msg) => {
                        format!(
                            "Error at line {}, column {}: {} (found {:?})",
                            token.span.line, token.span.column, msg, token.kind
                        )
                    }
                    ParseError::UnexpectedEOF => "Error: Unexpected End of File".to_string(),
                    ParseError::InvalidSyntax(msg) => format!("Error: {}", msg),
                };
                Err(error_msg)
            }
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
            action = { move = { p2 -> (0, 50) } }
        "#;
        let output = renderer.render(input).expect("Failed to render");
        assert!(output.contains("<svg"));
        assert!(output.contains("circle"));
        assert!(output.contains(">1<"));
        assert!(output.contains(">2<"));
    }

    #[test]
    fn test_error_reporting() {
        let renderer = Renderer::new();
        let input = "players = { "; // Missing closing brace
        let output = renderer.render(input).unwrap_err();
        assert!(output.contains("Error"));
        // EOF handling is tricky to test specific line without knowing where EOF span lands,
        // but it should contain "Error" and likely "Expected RBrace".
        assert!(output.contains("Expected RBrace"));
    }

    #[test]
    fn test_typo_suggestion() {
        let renderer = Renderer::new();
        let input = "aciton = { }"; // typo: action
        let output = renderer.render(input).unwrap_err();
        assert!(output.contains("Did you mean 'action'?"));
    }
}
