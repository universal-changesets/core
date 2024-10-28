use crate::changelog;
use crate::changeset::{self, Bump, ChangeSetExt, IncrementType};
use crate::plugin::{self, set_version_via_plugin};
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
    Changelog,
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
            println!("{}", v);
            return v;
        }
        Err(e) => {
            println!("Error: {}", e);
            return Version::new(0, 0, 0);
        }
    }
}

pub fn preview_version_command() {
    let current_version = plugin::get_version_via_plugin().unwrap();
    let changesets = changeset::get_changesets().unwrap();
    let bump_type = changesets.determine_final_bump_type().unwrap();
    let new_version = bump_type.map(|bump_type| current_version.bump(&bump_type));
    if new_version.is_none() {
        println!("There aren't any changes!");
        return;
    }

    let publish_date = chrono::Utc::now();

    let contents_to_insert =
        changelog::generate_changelog_contents(&new_version.unwrap(), &changesets, publish_date);

    println!("{}", contents_to_insert)
}

pub fn version_command() -> anyhow::Result<()> {
    let current_version = plugin::get_version_via_plugin()?;
    let changesets = changeset::get_changesets()?;
    let bump_type = changesets.determine_final_bump_type()?;
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
        return Ok(());
    }
    let new = new_version.unwrap();
    set_version_via_plugin(&new)?;

    write_changelog(&changesets, &new)?;

    changesets.consume(&current_version)?;
    return Ok(());
}

pub fn write_changelog(changesets: &[changeset::Change], new: &Version) -> anyhow::Result<()> {
    let existing_changelog_path = PathBuf::from(changelog::CHANGELOG_FILENAME);

    let mut existing_changelog = String::new();
    if !existing_changelog_path.exists() {
        existing_changelog.push_str("# Changelog\n\n");
    } else {
        let mut changelog_file = File::options()
            .create(false)
            .read(true)
            .write(false)
            .truncate(false)
            .open(&existing_changelog_path)
            .unwrap();
        changelog_file
            .read_to_string(&mut existing_changelog)
            .unwrap();
    }

    let today = chrono::Utc::now();

    let new_contents =
        changelog::generate_changelog(&existing_changelog, new, changesets, today).unwrap();

    let mut changelog_file = File::options()
        .create(true)
        .read(true)
        .write(true)
        .truncate(true)
        .open(&existing_changelog_path)
        .unwrap();

    write!(changelog_file, "{}", new_contents).unwrap();
    return Ok(());
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use rstest::rstest;
    use std::env::set_current_dir;
    use tempfile::tempdir;

    #[rstest]
    fn test_write_changelog_creates_changelog_if_not_exists() {
        let directory = tempdir().unwrap();
        // change to directory

        set_current_dir(&directory).unwrap();

        let changesets = vec![changeset::Change {
            summary: "Summary".to_string(),
            description: "Description".to_string(),
            bump_type: IncrementType::Major,
            file_path: PathBuf::new(),
        }];

        let new_version = Version::new(1, 0, 0);
        // Assert no changelog file exists before running the func
        assert!(!Path::new(changelog::CHANGELOG_FILENAME).exists());

        let result = write_changelog(&changesets, &new_version);
        assert!(result.is_ok());

        assert!(Path::new(changelog::CHANGELOG_FILENAME).exists());
    }
}
