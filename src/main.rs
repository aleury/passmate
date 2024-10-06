use clap::{Parser, Subcommand};
use passmate::{PassmateError, Vault};
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(visible_alias = "ls")]
    #[command(about = "List the entries stored in the vault")]
    List,

    #[command(about = "Get the value of an entry by name")]
    Get { name: String },

    #[command(about = "Add or update an entry")]
    Set { name: String, value: String },

    #[command(about = "Remove an entry")]
    Remove { name: String },
}

fn open_vault(path: PathBuf) -> Result<Vault, PassmateError> {
    let passphrase = rpassword::prompt_password("Enter password: ").map_err(PassmateError::IO)?;

    Vault::open(path, &passphrase)
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let dirs = xdg::BaseDirectories::with_prefix("passmate")?;
    let path = dirs.place_config_file("default.vault")?;
    let mut vault = open_vault(path)?;
    match args.command {
        Commands::List => {
            for entry in vault.entries() {
                println!("{entry}");
            }
        }
        Commands::Get { name } => {
            let Some(value) = vault.get(&name) else {
                eprintln!("{name} not found");
                std::process::exit(1);
            };
            println!("{value}");
        }
        Commands::Set { name, value } => {
            vault.set(name, value);
            vault.save()?;
        }
        Commands::Remove { name } => {
            vault.remove(&name);
            vault.save()?;
        }
    }
    Ok(())
}
