use clap::{Parser, Subcommand};
use playbook_lang_core::Renderer;
use playbook_lang_formatter::format_checked;
use playbook_lang_linter::lint;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about = "Playbook Language CLI Tool", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Convert playbook-lang files to SVG
    Render {
        /// Input .playbook file
        input: PathBuf,

        /// Output .svg file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Format playbook-lang files (outputs to stdout)
    Fmt {
        /// Input .playbook file
        input: PathBuf,
    },
    /// Lint playbook-lang files (outputs to stdout)
    Lint {
        /// Input .playbook file
        input: PathBuf,
    },
    /// Get final player positions (outputs to stdout as JSON)
    Play {
        /// Input .playbook file
        input: PathBuf,
    },
}

fn main() {
    let args = Args::parse();

    match args.command {
        Commands::Render { input, output } => {
            let input_content = fs::read_to_string(&input).expect("Failed to read input file");
            let renderer = Renderer::new();
            let result = renderer.render(&input_content);

            match result {
                Ok(svg) => {
                    let output_path = output.unwrap_or_else(|| {
                        let mut path = input.clone();
                        path.set_extension("svg");
                        path
                    });

                    fs::write(&output_path, svg).expect("Failed to write output file");
                    println!("Successfully converted {:?} to {:?}", input, output_path);
                }
                Err(e) => {
                    eprintln!("Compile Error:\n{}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Fmt { input } => {
            let input_content = fs::read_to_string(&input).expect("Failed to read input file");
            match format_checked(&input_content) {
                Ok(formatted) => print!("{}", formatted),
                Err(errors) => {
                    for e in &errors {
                        eprintln!("{}: parse error: {}", input.display(), e);
                    }
                    std::process::exit(1);
                }
            }
        }
        Commands::Lint { input } => {
            let input_content = fs::read_to_string(&input).expect("Failed to read input file");
            let linter_outputs = lint(&input_content);
            for output in linter_outputs.iter() {
                println!("lint error [{}]: {}", output.severity, output.message);
                println!("line: {}, column: {}", output.line, output.column);
            }
        }
        Commands::Play { input } => {
            let input_content = fs::read_to_string(&input).expect("Failed to read input file");
            let renderer = Renderer::new();
            let result = renderer.play(&input_content);

            match result {
                Ok(json) => {
                    println!("{}", json);
                }
                Err(e) => {
                    eprintln!("Compile Error:\n{}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}
