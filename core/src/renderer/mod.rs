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
        svg.push_str(&format!(
            "<svg width=\"{}\" height=\"{}\" viewBox=\"-105 -105 210 210\" xmlns=\"http://www.w3.org/2000/svg\">",
            self.width,
            self.height
        ));

        // Draw background/court
        svg.push_str("<rect x=\"-100\" y=\"-100\" width=\"200\" height=\"200\" fill=\"none\" stroke=\"#ccc\" stroke-width=\"1\" />");

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
