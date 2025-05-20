use clap::{Parser, Subcommand};
use std::ffi::OsStr;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use szdt::base58btc;
use szdt::car::{CarBlock, CarReader, CarWriter};
use szdt::car_claim_header::CarClaimHeader;
use szdt::claim::{self, Assertion, Claim, WitnessAssertion};
use szdt::ed25519::generate_signing_key;
use szdt::file::walk_files;
use szdt::manifest::Manifest;

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
    #[command(about = "Unpack a .car archive")]
    Unarchive {
        #[arg(help = "Archive file")]
        #[arg(value_name = "FILE")]
        file: PathBuf,
        #[arg(
            value_name = "DIR",
            short,
            long,
            help = "Directory to unpack archive into. Defaults to archive file name."
        )]
        dir: Option<PathBuf>,
    },

    #[command(about = "Create a .car archive from a folder full of files")]
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
        privkey: Option<String>,
    },

    #[command(about = "Generate a private key")]
    Genkey {},
}

fn archive(dir: PathBuf, secret_key: Option<String>) {
    let default_file_name = OsStr::new("archive");

    let file_name =
        PathBuf::from(dir.file_stem().unwrap_or(default_file_name)).with_extension("car");

    println!("Writing archive: {}", file_name.display());

    let manifest = Manifest::from_dir(&dir).expect("Unable to create manifest");

    // Create a new CarBlock for the manifest.
    // We'll sign over its CID.
    let manifest_block =
        CarBlock::from_serializable(&manifest).expect("Unable to generate CID from manifest");

    println!("manifest -> {}", manifest_block.cid());

    let claims = match secret_key {
        Some(secret_key) => {
            let secret_key_bytes =
                base58btc::decode(&secret_key).expect("Secret key base encoding is invalid");

            let witness_claim = claim::Builder::new(&secret_key_bytes)
                .expect("Unable to build claim")
                .add_ast(Assertion::Witness(WitnessAssertion {
                    cid: manifest_block.cid().clone(),
                }))
                .sign()
                .expect("Unable to sign claim");

            vec![witness_claim]
        }
        None => vec![],
    };

    let header = CarClaimHeader::new(vec![manifest_block.cid().clone()], claims);

    let car_file = fs::File::create(&file_name).expect("Failed to create archive file");
    let mut car = CarWriter::new(car_file, &header).expect("Should be able to create car");

    // Write manifest block first
    manifest_block
        .write_into(&mut car)
        .expect("Unable to write manifest to CAR");

    for path in walk_files(&dir).expect("Directory should be readable") {
        let body = fs::read(&path).expect("Path should be readable");
        let block = CarBlock::from_raw(body);
        block
            .write_into(&mut car)
            .expect("Should be able to write block to car file");
        println!("{} -> {}", &path.display(), block.cid());
    }

    car.flush()
        .expect("Should be able to flush all writes to car file");

    println!("Archive created: {}", file_name.display());
}

fn unarchive(file_path: PathBuf, dir: Option<PathBuf>) {
    let file = fs::File::open(&file_path).expect("Should be able to open file");

    let reader: CarReader<_, CarClaimHeader> =
        CarReader::read_from(file).expect("Should be able to read car file");

    let header = reader.header();

    let validated_claims: Vec<Claim> = header
        .claims
        .iter()
        .filter(|claim| {
            let result = claim.validate(None);
            if let Err(err) = &result {
                eprintln!("Claim error: {}", err);
            }
            result.is_ok()
        })
        .map(|claim| claim.clone())
        .collect();

    println!(
        "Validated {} of {} claims",
        validated_claims.len(),
        header.claims.len()
    );
    for claim in validated_claims {
        println!("{}", claim.payload().iss);
    }

    // Create a folder named after the file path
    let archive_dir = match dir {
        Some(dir) => dir,
        None => file_path
            .file_stem()
            .map(|p| p.into())
            .unwrap_or("archive".into()),
    };

    fs::create_dir(&archive_dir).expect("Should be able to create directory");

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
    let secret_key = generate_signing_key().to_bytes();
    let encoded_key = base58btc::encode(secret_key);
    println!("{}", encoded_key);
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Archive { dir, privkey } => archive(dir, privkey),
        Commands::Unarchive { file, dir } => unarchive(file, dir),
        Commands::Genkey {} => genkey(),
    }
}
