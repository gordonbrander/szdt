use clap::{Parser, Subcommand};
use std::ffi::OsStr;
use std::path::Path;
use std::path::PathBuf;
use szdt::config;
use szdt::key_storage::InsecureKeyStorage;
use szdt::mnemonic::Mnemonic;
use szdt::szdt::{archive, unarchive};
use szdt::text::truncate_string_left;

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
    #[command(about = "Unpack an .szdt archive")]
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

    #[command(about = "Create an .szdt archive from a folder")]
    Archive {
        #[arg(help = "Folder to archive")]
        #[arg(value_name = "DIR")]
        dir: PathBuf,

        #[arg(help = "Key to sign archive with")]
        #[arg(
            long_help = "Nickname of the key to sign the archive with. You can generate a signing key with `szdt key create`."
        )]
        #[arg(short, long)]
        #[arg(value_name = "NICKNAME")]
        #[arg(default_value = "default")]
        sign: String,
    },

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
        #[arg(value_name = "NICKNAME")]
        #[arg(default_value = "default")]
        nickname: String,
    },

    #[command(about = "List all signing keys")]
    List {},

    #[command(about = "Delete a signing key")]
    Delete {
        #[arg(help = "Key nickname")]
        #[arg(value_name = "NAME")]
        nickname: String,
    },
}

fn archive_cmd(config: &Config, dir: &Path, nickname: &str) {
    let default_file_name = OsStr::new("archive");

    let file_name =
        PathBuf::from(dir.file_stem().unwrap_or(default_file_name)).with_extension("szdt");

    let key_material = config
        .key_storage
        .key(nickname)
        .expect("Unable to access key")
        .expect("No key with that nickname. Tip: create a key using `szdt key create`.");

    let archive_receipt =
        archive(&dir, &file_name, &key_material).expect("Unable to create archive");

    println!("{:<12} {}", "Archive:", file_name.display());
    println!("{:<12} {} ({})", "Issuer:", key_material.did(), nickname);
    println!("");
    println!("{:<32} | {:<52}", "File", "Hash");
    for resource in &archive_receipt.manifest.resources {
        println!(
            "{:<32} | {:<52}",
            truncate_string_left(&resource.path.to_string_lossy(), 32),
            resource.src
        );
    }
    println!("");
    println!(
        "Archived {} files",
        &archive_receipt.manifest.resources.len()
    );
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
        Commands::Archive { dir, sign } => archive_cmd(&config, &dir, &sign),
        Commands::Unarchive { file, dir } => unarchive_cmd(dir, file),
        Commands::Key { command } => match command {
            KeyCommands::Create { nickname } => create_key_cmd(&config, &nickname),
            KeyCommands::List {} => list_keys_cmd(&config),
            KeyCommands::Delete { nickname } => delete_key_cmd(&config, &nickname),
        },
    }
}
