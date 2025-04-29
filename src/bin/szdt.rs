use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use szdt::base58btc;
use szdt::car::{CarBlock, CarHeader, CarWriter};
use szdt::ed25519::generate_private_key;
use szdt::file::walk_files;

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

fn archive(dir: PathBuf, _private_key: String) {
    let filename = "archive.car";
    println!("Writing archive: {}", filename);
    let car_file = fs::File::create(filename).expect("Failed to create archive file");
    let meta: HashMap<String, String> = HashMap::new();
    let header = CarHeader::new_v1(Vec::new(), meta);
    let mut car = CarWriter::new(car_file, header).expect("Should be able to create CAR");
    for path in walk_files(&dir).expect("Directory should be readable") {
        let data = fs::read(&path).expect("Path should be readable");
        let block = CarBlock::from_raw(data);
        car.write_block(&block)
            .expect("Should be able to write block");
        println!("{} -> {}", &path.display(), block.cid());
    }
    println!("Archive created: {}", filename);
}

fn unarchive(_file_path: PathBuf) {
    println!("TODO");
}

fn genkey() {
    let key = generate_private_key();
    let encoded_key = base58btc::encode(key);
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
