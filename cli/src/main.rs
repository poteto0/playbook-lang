use clap::Parser;
use playbook_lang_core::Renderer;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about = "Convert playbook-lang files to SVG", long_about = None)]
struct Args {
    /// Input .playbook file
    input: PathBuf,

    /// Output .svg file
    #[arg(short, long)]
    output: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();

    let input_content = fs::read_to_string(&args.input).expect("Failed to read input file");
    let renderer = Renderer::new();
    let result = renderer.render(&input_content);

    match result {
        Ok(svg) => {
            let output_path = args.output.unwrap_or_else(|| {
                let mut path = args.input.clone();
                path.set_extension("svg");
                path
            });

            fs::write(&output_path, svg).expect("Failed to write output file");
            println!(
                "Successfully converted {:?} to {:?}",
                args.input, output_path
            );
        }
        Err(e) => {
            eprintln!("Compile Error:\n{}", e);
            std::process::exit(1);
        }
    }
}
