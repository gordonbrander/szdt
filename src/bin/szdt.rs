use clap::{Parser, Subcommand};
use console::style;
use dialoguer::Confirm;
use std::ffi::OsStr;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::path::PathBuf;
use szdt::config;
use szdt::file::write_file_deep;
use szdt::key_storage::InsecureKeyStorage;
use szdt::link::ToLink;
use szdt::mnemonic::Mnemonic;
use szdt::szdt::{Unarchiver, archive};
use szdt::text::truncate_string_left;
use szdt::util::now;

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

    let archive_receipt = archive(&dir, &file_name, &key_material, Some(nickname.to_string()))
        .expect("Unable to create archive");

    println!("{:<12} {}", "Archive:", file_name.display());
    println!("{:<12} {}", "Nickname:", nickname);
    println!("{:<12} {}", "DID:", key_material.did());
    println!("");
    println!("{:<32} | {:<52}", "File", "Hash");
    for memo in &archive_receipt.manifest {
        let path = memo.protected.path.as_deref().unwrap_or("None");
        println!(
            "{:<32} | {:<52}",
            truncate_string_left(path, 32),
            memo.protected.src
        );
    }
    println!("");
    println!("Archived {} files", &archive_receipt.manifest.len());
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

    let file_bufreader = BufReader::new(File::open(&file_path).expect("Unable to open file"));

    let now_time = now();

    let mut count = 0;
    for result in Unarchiver::new(file_bufreader) {
        let (memo, bytes) = result.expect("Unable to read archive blocks");

        let confirmation = Confirm::new()
            .with_prompt(format!(
                "Unknown issuer {}. Do you want to trust this key?",
                memo.protected
                    .iss
                    .as_ref()
                    .map(|did| did.to_string())
                    .unwrap_or("None".to_string())
            ))
            .default(true)
            .show_default(true)
            .interact()
            .expect("Could not interact with terminal");

        if !confirmation {
            println!("Stopping");
            break;
        }

        // Check sig and expiries
        memo.validate(Some(now_time))
            .expect("Invalid memo signature");

        // Check checksum
        let hash = bytes.to_link().expect("Unable to hash body bytes");
        memo.checksum(&hash)
            .expect("Body bytes don't match checksum");

        // Use the path in the headers, or else the hash if no path given
        let file_path = memo.protected.path.clone().unwrap_or(hash.to_string());
        let path = archive_dir.join(&file_path);
        let bytes = bytes.into_inner();
        write_file_deep(&path, &bytes).expect("Unable to write file");

        println!("Path: {}", style(&file_path).bold());
        println!("Hash: {}", style(memo.protected.src.to_string()).green());
        println!(
            "From: {}",
            style(
                memo.protected
                    .iss
                    .as_ref()
                    .map(|iss| iss.to_string())
                    .unwrap_or("None".to_string())
            )
            .cyan()
        );
        println!("");
        count += 1;
    }

    println!("Unarchived {} files to {}", count, archive_dir.display());
}

fn create_key_cmd(config: &mut Config, nickname: &str) {
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
    println!("{:<1} | {:<16} | {:<56}", "🔒", "Nickname", "DID");

    for (nickname, key_material) in config.key_storage.keys().expect("Unable to read keys") {
        let has_private_key = if key_material.private_key().is_some() {
            "🔑"
        } else {
            " "
        };
        println!(
            "{:<1} | {:<16} | {:<56}",
            has_private_key,
            nickname,
            key_material.did()
        );
    }
}

fn delete_key_cmd(config: &mut Config, nickname: &str) {
    config
        .key_storage
        .delete_key(nickname)
        .expect("Unable to delete key");
}

fn main() {
    let contacts_file = config::contacts_file().expect("Unable to locate key storage directory");
    let key_storage =
        InsecureKeyStorage::new(&contacts_file).expect("Unable to initialize key storage");
    let mut config = Config { key_storage };

    let cli = Cli::parse();
    match cli.command {
        Commands::Archive { dir, sign } => archive_cmd(&config, &dir, &sign),
        Commands::Unarchive { file, dir } => unarchive_cmd(dir, file),
        Commands::Key { command } => match command {
            KeyCommands::Create { nickname } => create_key_cmd(&mut config, &nickname),
            KeyCommands::List {} => list_keys_cmd(&config),
            KeyCommands::Delete { nickname } => delete_key_cmd(&mut config, &nickname),
        },
    }
}
