use clap::{Parser, Subcommand};
use std::fs::File;
use std::path::PathBuf;
use szdat::archive::{ARCHIVE_CONTENT_TYPE, Archive};
use szdat::envelope::{Envelope, decode_base32, encode_base32, generate_secret_key};

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

        #[arg(help = "Secret key to sign archive with")]
        #[arg(
            long_help = "Secret key to sign archive with. The secret key should be a Base-32 encoded Ed25519 key. You can generate a key using the `secret` command.)"
        )]
        #[arg(short, long)]
        #[arg(value_name = "SECRET")]
        secret: String,
    },

    #[command(about = "Generate secret key")]
    Secret {},
}

fn archive(dir: PathBuf, secret_key: String) {
    let archive = Archive::from_dir(&dir).expect("Should be able to read directory");
    let mut body = Vec::new();
    archive
        .write_cbor_to(&mut body)
        .expect("Should be able to write body to vec");

    let secret_key_bytes = decode_base32(&secret_key).expect("Invalid secret key");

    let envelope = Envelope::of_content_type(ARCHIVE_CONTENT_TYPE.to_string(), body)
        .sign(&secret_key_bytes)
        .expect("Unable to sign envelope");

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

    // Check signature
    envelope.verify().expect("Signature verification failed.");

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
        Commands::Archive {
            dir,
            secret: secret_key,
        } => archive(dir, secret_key),
        Commands::Unarchive { file } => unarchive(file),
        Commands::Secret {} => secret(),
    }
}
