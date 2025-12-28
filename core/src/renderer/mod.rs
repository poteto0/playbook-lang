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
        court.push_str("<rect x=\"-105\" y=\"-105\" width=\"210\" height=\"210\" fill=\"white\" />");

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
        format!(
            "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"blue\" stroke-width=\"4\" />",
            s.from.0, s.from.1, s.to.0, s.to.1
        )
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

    pub fn render(&self, input: &str) -> String {
        use crate::ir::IRGenerator;
        use crate::lexer::Lexer;
        use crate::parser::Parser;

        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);

        match parser.parse() {
            Ok(playbook) => {
                let scene = IRGenerator::generate(playbook);
                self.render_scene(&scene)
            }
            Err(e) => format!("<svg><text>Parse Error: {:?}</text></svg>", e),
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
        let output = renderer.render(input);
        assert!(output.contains("<svg"));
        assert!(output.contains("circle"));
        assert!(output.contains(">1<"));
        assert!(output.contains(">2<"));
    }
}
