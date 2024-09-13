use clap::{Parser, Subcommand};
use directories::ProjectDirs;
use passmate::Vault;

#[derive(Parser)]
#[command(version, about)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let project_dirs =
        ProjectDirs::from("", "", "passmate").expect("passmate directories not found");
    match args.command {
        Commands::Init => {
            let config_dir = project_dirs.config_dir();
            if !config_dir.exists() {
                std::fs::create_dir_all(config_dir)?;
            }
            let vault_dir = config_dir.join("default.vault");
            let vault = Vault::open(&vault_dir)?;
            vault.save()?;
            println!("Initialized vault at {}", vault_dir.display());
        }
    }

    Ok(())
}
