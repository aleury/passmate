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
            let config_dir = project_dirs.create_config_directory("")?;
            let vault_dir = config_dir.join("default.vault");
            let vault = Vault::open(&vault_dir)?;
            vault.save()?;
            println!("Initialized vault at {}", vault_dir.display());
        }
    }

    Ok(())
}
