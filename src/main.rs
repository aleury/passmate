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
    Init,
    Get { name: String },
    Set { name: String, value: String },
    Remove { name: String },
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let project_dirs = xdg::BaseDirectories::with_prefix("passmate")?;
    match args.command {
        Commands::Init => {
            let path = project_dirs.place_config_file("default.vault")?;
            let vault = Vault::open(&path)?;
            vault.save()?;
            println!("Initialized vault at {}", path.display());
        }
        Commands::Get { name } => {
            let path = project_dirs.place_config_file("default.vault")?;
            let vault = Vault::open(&path)?;
            let Some(value) = vault.get(&name) else {
                eprintln!("{name} not found");
                std::process::exit(1);
            };
            println!("{value}");
        }
        Commands::Set { name, value } => {
            let path = project_dirs.place_config_file("default.vault")?;
            let mut vault = Vault::open(&path)?;
            vault.set(name, value);
            vault.save()?;
        }
        Commands::Remove { name } => {
            let path = project_dirs.place_config_file("default.vault")?;
            let mut vault = Vault::open(&path)?;
            vault.remove(&name);
            vault.save()?;
        }
    }
    Ok(())
}
