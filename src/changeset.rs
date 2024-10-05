use anyhow::Result;
use rand::seq::SliceRandom;
use semver::Version;
use std::io::Read;
use std::{fmt::Display, io::Write, path::PathBuf};

const NAMES: [&str; 39] = [
    "dog",
    "arnold",
    "cat",
    "kitten",
    "puppy",
    "armadillo",
    "giraffe",
    "happy",
    "sad",
    "emotional",
    "earth",
    "mars",
    "car",
    "robot",
    "whale",
    "python",
    "snake",
    "lizard",
    "bird",
    "eagle",
    "hawk",
    "falcon",
    "owl",
    "parrot",
    "penguin",
    "dolphin",
    "shark",
    "fish",
    "whale",
    "octopus",
    "squid",
    "jellyfish",
    "starfish",
    "seahorse",
    "seal",
    "otter",
    "beaver",
    "squirrel",
    "chipmunk",
];

const CHANGE_NAME_PARTS: i8 = 3;
const CHANGESET_DIRECTORY: &str = ".changeset";
const CHANGESET_FILE_KEY: &str = "changeset/type";

#[derive(Debug, PartialEq, Clone, Eq)]
pub enum IncrementType {
    Major,
    Minor,
    Patch,
}

pub trait ParseBumpType {
    fn parse_bump_type(&self) -> anyhow::Result<IncrementType>;
}

impl ParseBumpType for str {
    fn parse_bump_type(&self) -> anyhow::Result<IncrementType> {
        match self {
            "major" => Ok(IncrementType::Major),
            "minor" => Ok(IncrementType::Minor),
            "patch" => Ok(IncrementType::Patch),
            _ => Err(anyhow::anyhow!("invalid bump type")),
        }
    }
}

impl Display for IncrementType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IncrementType::Major => write!(f, "major"),
            IncrementType::Minor => write!(f, "minor"),
            IncrementType::Patch => write!(f, "patch"),
        }
    }
}

impl PartialOrd for IncrementType {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for IncrementType {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (IncrementType::Major, IncrementType::Major) => std::cmp::Ordering::Equal,
            (IncrementType::Major, _) => std::cmp::Ordering::Greater,
            (IncrementType::Minor, IncrementType::Minor) => std::cmp::Ordering::Equal,
            (IncrementType::Minor, IncrementType::Major) => std::cmp::Ordering::Less,
            (IncrementType::Minor, _) => std::cmp::Ordering::Greater,
            (IncrementType::Patch, IncrementType::Patch) => std::cmp::Ordering::Equal,
            (IncrementType::Patch, _) => std::cmp::Ordering::Less,
        }
    }
}

pub trait Bump {
    fn bump_major(&self) -> Version;
    fn bump_minor(&self) -> Version;
    fn bump_patch(&self) -> Version;
    fn bump(&self, bump_type: &IncrementType) -> Version {
        match bump_type {
            IncrementType::Major => self.bump_major(),
            IncrementType::Minor => self.bump_minor(),
            IncrementType::Patch => self.bump_patch(),
        }
    }
}

impl Bump for Version {
    fn bump_major(&self) -> Version {
        Version {
            major: self.major + 1,
            minor: 0,
            patch: 0,
            pre: self.pre.clone(),
            build: self.build.clone(),
        }
    }
    fn bump_minor(&self) -> Version {
        Version {
            major: self.major,
            minor: self.minor + 1,
            patch: 0,
            pre: self.pre.clone(),
            build: self.build.clone(),
        }
    }
    fn bump_patch(&self) -> Version {
        Version {
            major: self.major,
            minor: self.minor,
            patch: self.patch + 1,
            pre: self.pre.clone(),
            build: self.build.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Change {
    pub file_path: PathBuf,
    pub bump_type: IncrementType,
    pub summary: String,
    pub description: String,
}

impl TryFrom<PathBuf> for Change {
    type Error = &'static str;

    fn try_from(val: PathBuf) -> Result<Self, Self::Error> {
        if val.is_file() {
            let file = std::fs::File::open(&val);
            let file = match file {
                Ok(file) => file,
                Err(_) => return Err("Failed to open file"),
            };
            let mut reader = std::io::BufReader::new(file);
            let mut contents = String::new();
            let read_result = reader.read_to_string(&mut contents);
            if read_result.is_err() {
                return Err("Failed to read file");
            }

            let metadata = contents
                .lines()
                .find(|line| line.starts_with(CHANGESET_FILE_KEY));

            match metadata {
                Some(metadata) => {
                    let metadata = metadata.split(":").collect::<Vec<&str>>();
                    let bump_type = metadata[1].trim();
                    let parsed_bump_type = bump_type.parse_bump_type();
                    if parsed_bump_type.is_err() {
                        return Err("Invalid bump type found in file");
                    }
                    let parsed_bump_type = parsed_bump_type.unwrap();

                    let summary = contents
                        .lines()
                        .find(|line| line.starts_with("# "))
                        .unwrap()
                        .replace("# ", "");

                    let description = contents
                        .split("\n---\n")
                        .skip(2)
                        .skip_while(|line| line.starts_with("# "))
                        .collect::<Vec<&str>>()
                        .join("\n");

                    return Ok(Change {
                        bump_type: parsed_bump_type,
                        summary,
                        description,
                        file_path: val.clone(),
                    });
                }
                None => return Err("No metadata found in file"),
            }
        }
        return Err("Path is not a file");
    }
}

pub trait ChangeSetExt {
    fn determine_next_version(&self, current_version: &Version) -> Result<Version>;
    fn determine_final_bump_type(&self) -> Result<Option<IncrementType>>;
    fn consume(self, current_version: &Version) -> Result<Version>;
}

impl ChangeSetExt for Vec<Change> {
    fn determine_next_version(&self, current_version: &Version) -> Result<Version> {
        let bump_type = self.determine_final_bump_type()?;
        match bump_type {
            Some(bump_type) => Ok(current_version.bump(&bump_type)),
            None => Ok(current_version.clone()),
        }
    }
    fn determine_final_bump_type(&self) -> Result<Option<IncrementType>> {
        if self.is_empty() {
            return Ok(None);
        }
        let max_bump_type = self.iter().map(|c| &c.bump_type).max().cloned();

        Ok(max_bump_type)
    }
    fn consume(self, current_version: &Version) -> Result<Version> {
        let new_version = self.determine_next_version(current_version)?;
        self.iter().for_each(|c| {
            std::fs::remove_file(&c.file_path).unwrap();
        });

        Ok(new_version)
    }
}

pub fn generate_change_name() -> String {
    let parts: Vec<_> = NAMES
        .choose_multiple(&mut rand::thread_rng(), CHANGE_NAME_PARTS as usize)
        .cloned()
        .collect();
    return parts.join("-");
}

pub fn create_change_file(bump_type: IncrementType, message: &str) -> anyhow::Result<PathBuf> {
    let filename = generate_change_name();

    if !PathBuf::from(CHANGESET_DIRECTORY).exists() {
        std::fs::create_dir(CHANGESET_DIRECTORY)?;
    }

    let filepath = PathBuf::from(CHANGESET_DIRECTORY).join(format!("{}.md", filename));

    let mut file = std::fs::File::create(filepath.clone())?;

    write!(
        file,
        "---\n{CHANGESET_FILE_KEY}: {bump_type}\n---\n\n# {message}\n",
    )?;

    Ok(filepath)
}

/// Retrieves all changesets from the changeset directory
pub fn get_changesets() -> anyhow::Result<Vec<Change>> {
    let mut changesets: Vec<Change> = Vec::new();

    for entry in std::fs::read_dir(CHANGESET_DIRECTORY)? {
        let entry = entry?;
        let path = entry.path();
        let extension = path.extension();
        if path.is_file() && extension == Some(std::ffi::OsStr::new("md")) {
            let change = Change::try_from(path).unwrap();
            changesets.push(change);
        }
    }

    Ok(changesets)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    #[rstest]
    #[case::major(vec![
        Change {
            bump_type: IncrementType::Major,
            summary: "".to_string(),
            description: "".to_string(),
            file_path: PathBuf::new(),
        },
        Change {
            bump_type: IncrementType::Minor,
            summary: "".to_string(),
            description: "".to_string(),
            file_path: PathBuf::new(),
        },
        Change {
            bump_type: IncrementType::Patch,
            summary: "".to_string(),
            description: "".to_string(),
            file_path: PathBuf::new(),
        },
    ], Some(IncrementType::Major))]
    #[case::minor(vec![
        Change {
            bump_type: IncrementType::Minor,
            summary: "".to_string(),
            description: "".to_string(),
            file_path: PathBuf::new(),
        },
        Change {
            bump_type: IncrementType::Minor,
            summary: "".to_string(),
            description: "".to_string(),
            file_path: PathBuf::new(),
        },
        Change {
            bump_type: IncrementType::Patch,
            summary: "".to_string(),
            description: "".to_string(),
            file_path: PathBuf::new(),
        },
    ], Some(IncrementType::Minor))]
    #[case::patch(vec![
        Change {
            bump_type: IncrementType::Patch,
            summary: "".to_string(),
            description: "".to_string(),
            file_path: PathBuf::new(),
        },
        Change {
            bump_type: IncrementType::Patch,
            summary: "".to_string(),
            description: "".to_string(),
            file_path: PathBuf::new(),
        },
    ], Some(IncrementType::Patch))]
    fn test_determine_final_bump_type_selects_correct_bump_type(
        #[case] input: Vec<Change>,
        #[case] expected: Option<IncrementType>,
    ) {
        let result = input.determine_final_bump_type().unwrap();
        assert_eq!(result, expected);
    }
}
