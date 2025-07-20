use clap::{Parser, Subcommand};
use console::style;
use dialoguer::Confirm;
use std::ffi::OsStr;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::path::PathBuf;
use szdt_cli::config;
use szdt_cli::file::write_file_deep;
use szdt_cli::key_storage::InsecureKeyStorage;
use szdt_cli::szdt::{Unarchiver, archive};
use szdt_core::contact::Contact;
use szdt_core::ed25519_key_material::Ed25519KeyMaterial;
use szdt_core::link::ToLink;
use szdt_core::mnemonic::Mnemonic;
use szdt_core::nickname::Nickname;
use szdt_core::text::{ELLIPSIS, truncate};
use szdt_core::time::now;

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

    let nickname = Nickname::parse(nickname).expect("Invalid nickname");

    let contact = config
        .key_storage
        .contact(&nickname)
        .expect("Unable to access contacts")
        .expect("No contact with that nickname. Tip: create a key using `szdt key create`.");

    let archive_receipt = archive(dir, &file_name, &contact).expect("Unable to create archive");

    println!("{:<12} {}", "Archive:", file_name.display());
    println!(
        "{:<12} {} {}",
        "Issuer:",
        style(contact.nickname).bold().cyan(),
        style(format!("<{}>", contact.did)).cyan()
    );
    println!();
    println!("{:<32} | {:<52}", "File", "Hash");
    for memo in &archive_receipt.manifest {
        let path = memo.protected.path.as_deref().unwrap_or("None");
        println!(
            "{:<32} | {:<52}",
            truncate(path, 32, ELLIPSIS),
            style(memo.protected.src).green()
        );
    }
    println!();
    println!("Archived {} files", &archive_receipt.manifest.len());
}

fn unarchive_cmd(config: &mut Config, dir: Option<PathBuf>, file_path: PathBuf) {
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

        let Some(iss) = memo.protected.iss.as_ref() else {
            println!("Unsigned memo. Skipping");
            continue;
        };

        let contact: Contact = match config
            .key_storage
            .contact_for_did(iss)
            .expect("Unable to get key for did")
        {
            Some(contact) => contact,
            None => {
                let iss_nickname: &str = memo.protected.iss_nickname.as_deref().unwrap_or("anon");
                let iss_key_material =
                    Ed25519KeyMaterial::try_from(iss).expect("Unable to get public key from did");

                let confirmation = Confirm::new()
                    .with_prompt(format!(
                        "Unknown issuer {} {}. Do you want to add to trusted contacts?",
                        style(format!("~{iss_nickname}")).italic().bold().cyan(),
                        style(format!("<{iss}>")).cyan()
                    ))
                    .default(true)
                    .show_default(true)
                    .interact()
                    .expect("Could not interact with terminal");

                if confirmation {
                    let unique_nickname = config
                        .key_storage
                        .unique_nickname(iss_nickname)
                        .expect("Nickname is not valid");

                    let contact = Contact::new(
                        unique_nickname.clone(),
                        iss_key_material.did(),
                        iss_key_material.private_key(),
                    );

                    config
                        .key_storage
                        .create_contact(&contact)
                        .expect("Couldn't save key");

                    println!(
                        "Saved to contacts as {}",
                        style(unique_nickname).bold().cyan()
                    );
                    println!();

                    contact
                } else {
                    println!("Skipping...");
                    continue;
                }
            }
        };

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
            "Issuer: {} {}",
            style(contact.nickname).bold().cyan(),
            style(format!("<{}>", contact.did)).cyan()
        );
        println!();
        count += 1;
    }

    println!("Unarchived {} files to {}", count, archive_dir.display());
}

fn create_key_cmd(config: &mut Config, nickname: &str) {
    let unique_nickname = config
        .key_storage
        .unique_nickname(nickname)
        .expect("Unable to generate unique nickname");

    if unique_nickname.as_str() != nickname {
        println!(
            "Nickname {} already exists, using {}",
            nickname, &unique_nickname
        );
        println!();
    }

    let key_material = Ed25519KeyMaterial::generate();

    let contact = Contact::new(
        unique_nickname.clone(),
        key_material.did(),
        key_material.private_key(),
    );

    config
        .key_storage
        .create_contact(&contact)
        .expect("Unable to create key");

    let mnemonic = Mnemonic::try_from(&key_material).expect("Unable to generate mnemonic");

    println!("Key created:");
    println!(
        "{} {}",
        style(&unique_nickname).bold().cyan(),
        style(format!("<{}>", key_material.did())).cyan()
    );
    println!();
    println!("Recovery phrase:");
    println!("{mnemonic}");
}

fn list_keys_cmd(config: &Config) {
    println!("{:<2} | {:<24} | {:<56}", "🔒", "Nickname", "DID");

    for contact in config
        .key_storage
        .contacts()
        .expect("Unable to read contacts")
    {
        let has_private_key = if contact.private_key.is_some() {
            "🔑"
        } else {
            " "
        };
        println!(
            "{:<2} | {:<24} | {:<56}",
            has_private_key, contact.nickname, contact.did
        );
    }
}

fn delete_key_cmd(config: &mut Config, nickname: &str) {
    config
        .key_storage
        .delete_contact(nickname)
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
        Commands::Unarchive { file, dir } => unarchive_cmd(&mut config, dir, file),
        Commands::Key { command } => match command {
            KeyCommands::Create { nickname } => create_key_cmd(&mut config, &nickname),
            KeyCommands::List {} => list_keys_cmd(&config),
            KeyCommands::Delete { nickname } => delete_key_cmd(&mut config, &nickname),
        },
    }
}
