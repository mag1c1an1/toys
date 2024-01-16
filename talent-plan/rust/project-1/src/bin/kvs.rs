use std::env::current_dir;
use clap::{Parser, Subcommand};
use kvs::{KvsError, KvStore};
use kvs::KvsError::{Io, KeyNotFound};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(disable_help_subcommand = true)]
#[command(arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Set the value of a string key to a string
    Set {
        /// A string key
        key: String,
        /// The string value of the key
        value: String,
    },
    /// Get the string value of a given string key
    Get {
        /// A string key
        key: String
    },
    /// Remove a given key
    Rm {
        /// A string key
        key: String
    },
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let mut kvs = KvStore::open(current_dir()?)?;
    match cli.command {
        Commands::Set { key, value } => {
            kvs.set(key, value)?;
        }
        Commands::Get { key } => {
            if let Some(ans) = kvs.get(key)? {
                println!("{ans}");
            }
        }
        Commands::Rm { key } => {
            kvs.remove(key)?
        }
    };
    Ok(())
}