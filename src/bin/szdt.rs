use clap::{Parser, Subcommand};
use std::path::PathBuf;
use szdt::did::encode_base58btc;
use szdt::ed25519::generate_private_key;

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

fn archive(_dir: PathBuf, _private_key: String) {
    println!("TODO");
}

fn unarchive(_file_path: PathBuf) {
    println!("TODO");
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
