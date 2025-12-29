use clap::{Parser, Subcommand};
use lirpc::contracts::compile::compile;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Compile {
        crate_name: String,
        target_name: String,
        contract_path: String,
        version: String,
        #[clap(long, default_value_t = false)]
        release: bool,
        #[clap(long, default_value_t = false)]
        minimal: bool,
    },
    CodeGen {
        contract_path: String,
        language: String,
        output_path: String,
    },
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let command_result = match args.command {
        Commands::Compile {
            crate_name,
            target_name,
            contract_path,
            version,
            release,
            minimal,
        } => {
            compile(
                &crate_name,
                &target_name,
                &contract_path,
                version,
                release,
                minimal,
            )
            .await
        }
        Commands::CodeGen {
            contract_path: _,
            language: _,
            output_path: _,
        } => {
            todo!()
        }
    };

    if let Err(e) = command_result {
        eprintln!("Error during execution of command: {e}");
    }
}
