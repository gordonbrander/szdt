use clap::{Parser, Subcommand};
use std::ffi::OsStr;
use std::path::PathBuf;
use szdt::base58btc;
use szdt::config;
use szdt::ed25519_key_material::Ed25519KeyMaterial;
use szdt::key_storage::InsecureKeyStorage;
use szdt::mnemonic::Mnemonic;
use szdt::szdt::{archive, unarchive};

/// Shared CLI configuration
struct Config {
    pub key_storage: InsecureKeyStorage,
}

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
        privkey: String,
    },

    #[command(about = "Generate a private key")]
    Genkey {},

    #[command(about = "Create and manage signing keys")]
    Key {
        #[command(subcommand)]
        command: KeyCommands,
    },
}

#[derive(Subcommand)]
enum KeyCommands {
    #[command(about = "Create a new keypair")]
    Create {
        #[arg(help = "Nickname for key")]
        #[arg(value_name = "NAME")]
        #[arg(default_value = "default")]
        name: String,
    },

    #[command(about = "List all signing keys")]
    List {},

    #[command(about = "Delete a signing key")]
    Delete {
        #[arg(help = "Key nickname")]
        #[arg(value_name = "NAME")]
        name: String,
    },
}

fn archive_cmd(dir: PathBuf, private_key_base58: String) {
    let default_file_name = OsStr::new("archive");

    let file_name =
        PathBuf::from(dir.file_stem().unwrap_or(default_file_name)).with_extension("szdt");

    let private_key_bytes =
        base58btc::decode(&private_key_base58).expect("Secret key base encoding is invalid");

    let key_material = Ed25519KeyMaterial::try_from_private_key(&private_key_bytes)
        .expect("Private key is not valid");

    let archive_receipt =
        archive(&dir, &file_name, &key_material).expect("Unable to create archive");

    println!("Archive created: {}", file_name.display());
    println!("Issuer: {}", key_material.did());
    println!("Manifest:");
    for resource in archive_receipt.manifest.resources {
        println!("{}\t{}", resource.path.to_string_lossy(), resource.src);
    }
}

fn unarchive_cmd(dir: Option<PathBuf>, file_path: PathBuf) {
    // Create a folder named after the file path
    let archive_dir = match dir {
        Some(dir) => dir,
        None => file_path
            .file_stem()
            .map(|p| p.into())
            .unwrap_or("archive".into()),
    };

    unarchive(&archive_dir, &file_path).expect("Unable to unpack archive");

    println!("Unpacked archive");
}

fn genkey() {
    let key_material = Ed25519KeyMaterial::generate();
    let mnemonic = Mnemonic::try_from(&key_material).expect("Unable to generate mnemonic");
    println!("{}", mnemonic);
}

fn create_key_cmd(config: &Config, nickname: &str) {
    let key_material = config
        .key_storage
        .create_key(&nickname)
        .expect("Unable to create key");
    let mnemonic = Mnemonic::try_from(&key_material).expect("Unable to generate mnemonic");
    println!("Nickname: {}", nickname);
    println!("DID: {}", key_material.did());
    println!("");
    println!("Recovery phrase:");
    println!("{}", mnemonic);
}

fn list_keys_cmd(config: &Config) {
    println!("{:<16} | {:<56}", "Nickname", "DID");

    for (nickname, did) in config.key_storage.keys().expect("Unable to read keys") {
        println!("{:<16} | {:<56}", nickname, did);
    }
}

fn delete_key_cmd(config: &Config, nickname: &str) {
    config
        .key_storage
        .delete_key(nickname)
        .expect("Unable to delete key");
}

fn main() {
    let keys_dir = config::keys_dir().expect("Unable to locate key storage directory");
    let key_storage = InsecureKeyStorage::new(keys_dir).expect("Unable to initialize key storage");
    let config = Config { key_storage };

    let cli = Cli::parse();
    match cli.command {
        Commands::Archive { dir, privkey } => archive_cmd(dir, privkey),
        Commands::Unarchive { file, dir } => unarchive_cmd(dir, file),
        Commands::Genkey {} => genkey(),
        Commands::Key { command } => match command {
            KeyCommands::Create { name } => create_key_cmd(&config, &name),
            KeyCommands::List {} => list_keys_cmd(&config),
            KeyCommands::Delete { name } => delete_key_cmd(&config, &name),
        },
    }
}
