use clap::{Parser, Subcommand};
use std::path::PathBuf;
use szdat::Archive;

#[derive(Parser)]
#[command(version = "0.0.1")]
#[command(author = "szdat")]
#[command(about = "Simple censorship-resistant publishing and archiving")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Unpack a szdat archive")]
    Unarchive {
        #[arg(help = "Archive file")]
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },

    #[command(about = "Create a szdat archive from a folder full of files")]
    Archive {
        #[arg(help = "Directory to archive")]
        #[arg(value_name = "DIR")]
        dir: PathBuf,
    },
}

fn archive(dir: PathBuf) {
    let archive = Archive::from_dir(&dir).expect("Read archive files");
    let output_path = dir.with_extension("szdat");
    archive
        .write_cbor(&output_path)
        .expect("Wrote archive file");
}

fn unarchive(_file: PathBuf) {}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Archive { dir } => archive(dir),
        Commands::Unarchive { file } => unarchive(file),
    }
}
