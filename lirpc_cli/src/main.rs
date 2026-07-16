use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::ExitCode,
};

use clap::{Parser, Subcommand, ValueEnum};
use lirpc::{api_spec::ApiSpec, codegen::CodeGen};
use lirpc_rs_codegen::RustCodeGen;

#[derive(Parser)]
#[command(name = "lirpc", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Generate a client package from an api spec file.
    Codegen {
        /// The target language to generate a client for.
        language: Language,
        /// Path to the api spec JSON file.
        api_spec_path: PathBuf,
        /// Folder the generated package is written into.
        output_folder: PathBuf,
        /// Overwrite the output folder without asking for confirmation.
        #[arg(long)]
        overwrite: bool,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Language {
    Rust,
}

#[derive(Debug, thiserror::Error)]
enum CliError {
    #[error("failed to read api spec file '{path}': {source}")]
    ReadApiSpec {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to parse api spec file '{path}': {source}")]
    ParseApiSpec {
        path: PathBuf,
        #[source]
        source: lirpc::api_spec::ApiSpecError,
    },
    #[error("failed to remove existing output folder '{path}': {source}")]
    RemoveOutputFolder {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to write generated file '{path}': {source}")]
    WriteGeneratedFile {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to read confirmation from stdin: {0}")]
    ReadConfirmation(#[source] io::Error),
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Command::Codegen {
            language,
            api_spec_path,
            output_folder,
            overwrite,
        } => match run_codegen(language, &api_spec_path, &output_folder, overwrite) {
            Ok(true) => ExitCode::SUCCESS,
            Ok(false) => {
                println!("Aborted.");
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("Error: {e}");
                ExitCode::FAILURE
            }
        },
    }
}

/// Returns `Ok(false)` if the user declined to overwrite an existing output folder.
fn run_codegen(
    language: Language,
    api_spec_path: &Path,
    output_folder: &Path,
    overwrite: bool,
) -> Result<bool, CliError> {
    let spec_json = fs::read_to_string(api_spec_path).map_err(|source| CliError::ReadApiSpec {
        path: api_spec_path.to_path_buf(),
        source,
    })?;

    let spec = ApiSpec::from_json(&spec_json).map_err(|source| CliError::ParseApiSpec {
        path: api_spec_path.to_path_buf(),
        source,
    })?;

    if output_folder.exists() {
        if !overwrite && !confirm_overwrite(output_folder)? {
            return Ok(false);
        }

        fs::remove_dir_all(output_folder).map_err(|source| CliError::RemoveOutputFolder {
            path: output_folder.to_path_buf(),
            source,
        })?;
    }

    let package = match language {
        Language::Rust => RustCodeGen::generate_package(&spec),
    };

    for (relative_path, contents) in package {
        let file_path = output_folder.join(&relative_path);

        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).map_err(|source| CliError::WriteGeneratedFile {
                path: file_path.clone(),
                source,
            })?;
        }

        fs::write(&file_path, contents).map_err(|source| CliError::WriteGeneratedFile {
            path: file_path.clone(),
            source,
        })?;
    }

    println!(
        "Generated client package into '{}'.",
        output_folder.display()
    );

    Ok(true)
}

fn confirm_overwrite(path: &Path) -> Result<bool, CliError> {
    print!(
        "Output folder '{}' already exists. Overwrite? [y/N] ",
        path.display()
    );
    io::stdout().flush().map_err(CliError::ReadConfirmation)?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(CliError::ReadConfirmation)?;

    Ok(matches!(input.trim().to_lowercase().as_str(), "y" | "yes"))
}
