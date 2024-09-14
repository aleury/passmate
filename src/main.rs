use clap::{Parser, Subcommand};
use passmate::Vault;

#[derive(Parser)]
#[command(version, about)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Get { name: String },
    Set { name: String, value: String },
    Remove { name: String },
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let dirs = xdg::BaseDirectories::with_prefix("passmate")?;
    let path = dirs.place_config_file("default.vault")?;
    let mut vault = Vault::open(&path)?;
    match args.command {
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
