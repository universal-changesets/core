use crate::changelog;
use crate::changeset::{self, Bump, ChangeSetExt, IncrementType};
use crate::plugin;
use clap::{Args, Parser, Subcommand, ValueEnum};
use cliclack::{input, select};
use semver::Version;
use std::io::{Read, Write};
use std::{fs::File, path::PathBuf};

#[derive(Parser, Debug, PartialEq, Clone, ValueEnum, Eq)]
pub enum BumpType {
    Major,
    Minor,
    Patch,
}

impl BumpType {
    fn to_increment_type(&self) -> IncrementType {
        match self {
            BumpType::Major => IncrementType::Major,
            BumpType::Minor => IncrementType::Minor,
            BumpType::Patch => IncrementType::Patch,
        }
    }
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Args)]
pub struct AddCommand {
    /// The type of increment
    #[arg(short = 't', long = "type")]
    pub increment_type: Option<BumpType>,

    #[arg(short = 'm', long = "message")]
    pub message: Option<String>,
}

#[derive(Debug, Args)]
pub struct VersionCommand {}

#[derive(Debug, Args)]
pub struct GetCommand {}

#[derive(Parser)]
pub enum PreviewCommands {
    Version(GetCommand),
}
#[derive(Parser)]
pub struct Preview {
    #[structopt(subcommand)]
    pub preview_commands: PreviewCommands,
}

#[derive(Subcommand)]
pub enum Commands {
    Add(AddCommand),
    Version(VersionCommand),
    Get(GetCommand),
    Preview(Preview),
}

pub fn add_changeset(command: &AddCommand) {
    let increment_type = command.increment_type.as_ref().unwrap_or_else(|| {
        select("Which type of increment?")
            .items(&[
                (&BumpType::Major, "Major", ""),
                (&BumpType::Minor, "Minor", ""),
                (&BumpType::Patch, "Patch", ""),
            ])
            .interact()
            .unwrap()
    });

    let message = command.message.clone().unwrap_or_else(|| {
        input("Enter a message for the changeset:")
            .interact()
            .unwrap()
    });

    let change_file =
        changeset::create_change_file(increment_type.to_increment_type(), message.as_str())
            .unwrap();
    println!("Changeset created at: {}", change_file.display());
}

pub fn get_version() -> Version {
    let ver = plugin::get_version_via_plugin();
    match ver {
        Ok(v) => {
            println!("Version: {}", v);
            return v;
        }
        Err(e) => {
            println!("Error: {}", e);
            return Version::new(0, 0, 0);
        }
    }
}

pub fn version_command() {
    let current_version = plugin::get_version_via_plugin().unwrap();
    let changesets = changeset::get_changesets().unwrap();
    let bump_type = changesets.determine_final_bump_type().unwrap();
    let new_version = match bump_type {
        Some(bump_type) => {
            let new_version = current_version.bump(&bump_type);
            println!("Updating version from {current_version} to {new_version}");
            Some(new_version)
        }
        None => {
            println!("No changesets found");
            None
        }
    };
    if new_version.is_none() {
        return;
    }
    let existing_changelog_path = PathBuf::from(changelog::CHANGELOG_FILENAME);
    let mut file = File::options()
        .create(false)
        .read(true)
        .write(false)
        .truncate(false)
        .open(&existing_changelog_path)
        .unwrap();

    let mut existing_changelog = String::new();
    file.read_to_string(&mut existing_changelog).unwrap();

    let publish_date = chrono::Utc::now();

    let new_contents = changelog::generate_changelog(
        &existing_changelog,
        &new_version.unwrap(),
        &changesets,
        publish_date,
    )
    .unwrap();

    let mut file = File::options()
        .create(true)
        .read(true)
        .write(true)
        .truncate(true)
        .open(&existing_changelog_path)
        .unwrap();

    write!(file, "{}", new_contents).unwrap();
    changesets.consume(&current_version).unwrap();
}
