use rand::seq::SliceRandom;
use semver::Version;
use std::io::Read;
use std::{fmt::Display, io::Write, path::PathBuf};

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

const NAMES: [&str; 16] = [
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
];

const CHANGE_NAME_PARTS: i8 = 3;
const CHANGESET_DIRECTORY: &str = ".changeset";
const CHANGESET_FILE_KEY: &str = "changeset/type";

#[derive(Debug, Clone)]
pub struct Change {
    pub bump_type: IncrementType,
    pub file_path: PathBuf,
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

    file.write_all(
        format!("---\n{CHANGESET_FILE_KEY}: {bump_type}\n---\n\n# {message}\n").as_bytes(),
    )?;

    Ok(filepath)
}

pub fn get_changesets() -> anyhow::Result<Vec<Change>> {
    let mut changesets: Vec<Change> = Vec::new();

    for entry in std::fs::read_dir(CHANGESET_DIRECTORY)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let file = std::fs::File::open(path)?;
            let mut reader = std::io::BufReader::new(file);
            let mut contents = String::new();
            reader.read_to_string(&mut contents)?;

            let metadata = contents
                .lines()
                .find(|line| line.starts_with(CHANGESET_FILE_KEY));

            if let Some(metadata) = metadata {
                let metadata = metadata.split(":").collect::<Vec<&str>>();
                let bump_type = metadata[1].trim();
                changesets.push(Change {
                    bump_type: bump_type.parse_bump_type()?,
                    file_path: entry.path(),
                });
            }
        }
    }

    Ok(changesets)
}

pub fn determine_final_bump_type(
    changesets: &Vec<Change>,
) -> anyhow::Result<Option<IncrementType>> {
    // if any changeset has a major bump, return major
    for changeset in changesets {
        if changeset.bump_type == IncrementType::Major {
            return Ok(Some(IncrementType::Major));
        }
    }
    for changeset in changesets {
        if changeset.bump_type == IncrementType::Minor {
            return Ok(Some(IncrementType::Minor));
        }
    }
    for changeset in changesets {
        if changeset.bump_type == IncrementType::Patch {
            return Ok(Some(IncrementType::Patch));
        }
    }
    return Ok(None);
}

pub fn determine_next_version(
    current_version: &Version,
    changesets: &Vec<Change>,
) -> anyhow::Result<Version> {
    let bump_type = determine_final_bump_type(changesets)?;
    match bump_type {
        Some(bump_type) => Ok(current_version.bump(&bump_type)),
        None => Ok(current_version.clone()),
    }
}

pub fn consume_changesets(
    current_version: &Version,
    changesets: Vec<Change>,
) -> anyhow::Result<Version> {
    let new_version = determine_next_version(current_version, &changesets)?;
    for changeset in changesets {
        std::fs::remove_file(changeset.file_path)?;
    }
    Ok(new_version)
}
