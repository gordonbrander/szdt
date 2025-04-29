use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use szdt::base58btc;
use szdt::car::{CarBlock, CarHeader, CarReader, CarWriter};
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
    let file_name = "archive.car";
    println!("Writing archive: {}", file_name);
    let car_file = fs::File::create(file_name).expect("Failed to create archive file");
    let header = CarHeader::new_v1();
    let mut car = CarWriter::new(car_file, &header).expect("Should be able to create CAR");
    for path in walk_files(&dir).expect("Directory should be readable") {
        let body = fs::read(&path).expect("Path should be readable");
        let block = CarBlock::from_raw(body);
        car.write_block(&block)
            .expect("Should be able to write block");
        println!("{} -> {}", &path.display(), block.cid());
    }
    println!("Archive created: {}", file_name);
}

fn unarchive(file_path: PathBuf) {
    let file = fs::File::open(&file_path).expect("Should be able to open file");
    let reader: CarReader<fs::File, HashMap<String, String>> =
        CarReader::read_from(file).expect("Should be able to read CAR file");

    // Create a folder named after the file path
    let archive_dir: PathBuf = file_path
        .file_stem()
        .map(|p| p.into())
        .unwrap_or("archive".into());

    fs::create_dir_all(&archive_dir).expect("Should be able to create directory");

    for block in reader {
        let block = block.expect("Should be able to read block");
        let path = archive_dir.join(block.cid().to_string());
        let body = block.body();
        fs::write(&path, body).expect("Should be able to write file");
        println!("{} -> {}", block.cid(), &path.display());
    }
    println!("Unpacked archive");
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
