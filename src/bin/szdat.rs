use clap::{Parser, Subcommand};
use std::fs::File;
use std::path::PathBuf;
use szdat::szdat::{ARCHIVE_CONTENT_TYPE, Archive, Envelope, encode_base32, generate_secret_key};

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
    let mut body = Vec::new();
    archive
        .write_cbor_to(&mut body)
        .expect("Should be able to write body to vec");

    let envelope = Envelope::of_content_type(ARCHIVE_CONTENT_TYPE.to_string(), body);
    let output_path = dir.with_extension("szdat");
    let file = File::create(&output_path).expect("Should be able to create file");
    envelope
        .write_cbor_to(file)
        .expect("Should be able to write to file");
    println!("Archived: {:?}", output_path);
}

fn unarchive(file_path: PathBuf) {
    let file = File::open(&file_path).expect("Should be able to open file");
    let envelope = Envelope::read_cbor_from(file).expect("Should be able to read envelope");
    let archive: Archive = envelope
        .deserialize_body()
        .expect("Should be able to deserialize archive");

    let dir = file_path.with_extension("");
    archive
        .write_archive_contents(&dir)
        .expect("Should be able to write unarchived files");

    println!("Unarchived: {:?}", dir);
}

fn secret() {
    let key = generate_secret_key();
    let encoded_key = encode_base32(key);
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
