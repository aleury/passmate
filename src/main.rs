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
    }

    Ok(())
}
