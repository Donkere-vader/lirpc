use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    CodeGen {
        contract_path: String,
        language: String,
        output_path: String,
    },
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let command_result: Result<(), String> = match args.command {
        Commands::CodeGen {
            contract_path: _,
            language: _,
            output_path: _,
        } => {
            // TODO
            Ok(())
        }
    };

    if let Err(e) = command_result {
        eprintln!("Error during execution of command: {e}");
    }
}
