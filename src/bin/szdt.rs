use clap::{Parser, Subcommand};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use szdt::archive::{ARCHIVE_CONTENT_TYPE, Archive};
use szdt::cose::CoseEnvelope;
use szdt::did::{decode_base58btc, encode_base58btc};
use szdt::ed25519::{generate_private_key, vec_to_private_key};

#[derive(Parser)]
#[command(version = "0.0.1")]
#[command(author = "szdt")]
#[command(about = "Censorship-resistant publishing and archiving")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Unpack a szdt archive")]
    Unarchive {
        #[arg(help = "Archive file")]
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },

    #[command(about = "Create a szdt archive from a folder full of files")]
    Archive {
        #[arg(help = "Directory to archive")]
        #[arg(value_name = "DIR")]
        dir: PathBuf,

        #[arg(help = "Private key to sign archive with")]
        #[arg(
            long_help = "Private key to sign archive with. The private key should be a Base-32 encoded Ed25519 key. You can generate a key using the `genkey` command.)"
        )]
        #[arg(short, long)]
        #[arg(value_name = "KEY")]
        privkey: String,
    },

    #[command(about = "Generate a private key")]
    Genkey {},
}

fn archive(dir: PathBuf, private_key: String) {
    let archive = Archive::from_dir(&dir).expect("Should be able to read directory");
    let mut body = Vec::new();
    archive
        .write_cbor_to(&mut body)
        .expect("Should be able to write body to vec");

    let private_key_bytes =
        decode_base58btc(&private_key).expect("Invalid private key (could not decode base58btc)");
    let private_key = vec_to_private_key(&private_key_bytes)
        .expect("Invalid private key (wrong number of bytes)");

    let signed_cbor_bytes = CoseEnvelope::of_content_type(ARCHIVE_CONTENT_TYPE.to_string(), body)
        .to_cose_sign1_ed25519(&private_key)
        .expect("Unable to sign envelope");

    let output_path = dir.with_extension("szdt");
    let mut file = File::create(&output_path).expect("Should be able to create file");
    file.write_all(&signed_cbor_bytes)
        .expect("Should be able to write to file");

    println!("Archived: {:?}", output_path);
}

fn unarchive(file_path: PathBuf) {
    let bytes = std::fs::read(&file_path).expect("Should be able to read file");

    let envelope =
        CoseEnvelope::from_cose_sign1(&bytes).expect("Must be valid COSE_Sign1 structure");

    let archive: Archive = envelope
        .deserialize_payload()
        .expect("Should be able to deserialize archive");

    let dir = file_path.with_extension("");
    archive
        .write_archive_contents(&dir)
        .expect("Should be able to write unarchived files");

    println!("Unarchived: {:?}", dir);
}

fn genkey() {
    let key = generate_private_key();
    let encoded_key = encode_base58btc(key);
    println!("{}", encoded_key);
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Archive { dir, privkey } => archive(dir, privkey),
        Commands::Unarchive { file } => unarchive(file),
        Commands::Genkey {} => genkey(),
    }
}
