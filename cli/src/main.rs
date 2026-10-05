use clap::{Parser, Subcommand};
use playbook_lang_core::Renderer;
use playbook_lang_formatter::format_checked;
use playbook_lang_linter::lint;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

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

fn main() -> ExitCode {
    match run(Args::parse().command) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{}", e);
            ExitCode::FAILURE
        }
    }
}

fn read_input(path: &Path) -> Result<String, String> {
    fs::read_to_string(path)
        .map_err(|e| format!("{}: failed to read input file: {}", path.display(), e))
}

fn run(command: Commands) -> Result<(), String> {
    match command {
        Commands::Render { input, output } => {
            let svg = Renderer::new()
                .render(&read_input(&input)?)
                .map_err(|e| format!("Compile Error:\n{}", e))?;
            let output_path = output.unwrap_or_else(|| input.with_extension("svg"));
            fs::write(&output_path, svg).map_err(|e| {
                format!(
                    "{}: failed to write output file: {}",
                    output_path.display(),
                    e
                )
            })?;
            println!("Successfully converted {:?} to {:?}", input, output_path);
        }
        Commands::Fmt { input } => {
            let formatted = format_checked(&read_input(&input)?).map_err(|errors| {
                errors
                    .iter()
                    .map(|e| format!("{}: parse error: {}", input.display(), e))
                    .collect::<Vec<_>>()
                    .join("\n")
            })?;
            print!("{}", formatted);
        }
        Commands::Lint { input } => {
            for output in &lint(&read_input(&input)?) {
                println!("lint error [{}]: {}", output.severity, output.message);
                println!("line: {}, column: {}", output.line, output.column);
            }
        }
        Commands::Play { input } => {
            let json = Renderer::new()
                .play(&read_input(&input)?)
                .map_err(|e| format!("Compile Error:\n{}", e))?;
            println!("{}", json);
        }
    }
    Ok(())
}
