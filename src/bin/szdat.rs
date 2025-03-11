use clap::{Parser, Subcommand};
use std::path::PathBuf;
use szdat::{Archive, format_key_base32, generate_private_key};

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

    #[command(about = "Generate secret key")]
    Secret {},
}

fn archive(dir: PathBuf) {
    let archive = Archive::from_dir(&dir).expect("Should be able to read directory");
    let output_path = dir.with_extension("szdat");
    archive
        .write_archive(&output_path)
        .expect("Should be able to write archive file");
    println!("Archived: {:?}", output_path);
}

fn unarchive(file: PathBuf) {
    let archive = Archive::read_archive(&file).expect("Should be able to read archive");
    let dir = file.with_extension("");
    archive
        .write_archive_contents(&dir)
        .expect("Should be able to write unarchived files");

    println!("Unarchived: {:?}", dir);
}

fn secret() {
    let key = generate_private_key();
    let encoded_key = format_key_base32(key);
    println!("{}", encoded_key);
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Archive { dir } => archive(dir),
        Commands::Unarchive { file } => unarchive(file),
        Commands::Secret {} => secret(),
    }
}
