use rand::seq::SliceRandom;
use semver::Version;
use std::fs::File;
use std::io::{self, Read, Seek};
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
const CHANGELOG_FILENAME: &str = "CHANGELOG.md";

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

    write!(
        file,
        "---\n{CHANGESET_FILE_KEY}: {bump_type}\n---\n\n# {message}\n",
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

pub fn determine_final_bump_type(changesets: &[Change]) -> anyhow::Result<Option<IncrementType>> {
    if changesets.is_empty() {
        return Ok(None);
    }
    let max_bump_type = changesets.iter().map(|c| &c.bump_type).max().cloned();

    Ok(max_bump_type)
}

pub fn determine_next_version(
    current_version: &Version,
    changesets: &[Change],
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

    changesets.iter().for_each(|c| {
        std::fs::remove_file(&c.file_path).unwrap();
    });

    Ok(new_version)
}

pub fn generate_changelog_contents(next_version: &Version, changesets: &[Change]) -> String {
    let mut contents = format!("## {next_version}\n\n", next_version = next_version);

    for _ in changesets {
        contents.push_str("- \n");
    }

    contents
}

/// Inserts the contents before the **first** line that starts with `search`
fn insert_before(contents: &str, search: &str, insertable: &str) -> String {
    let mut found = false;
    let mut result = String::new();

    for line in contents.lines() {
        if line.starts_with(search) && !found {
            found = true;
            result.push_str(insertable);
        }
        result.push_str(line);
        result.push('\n');
    }

    if !found {
        result.push_str(insertable);
        result.push('\n');
    }

    result
}

/// Creates, or updates a CHANGELOG.md file with the contents of the changesets
pub fn generate_changelog(next_version: &Version, changesets: &[Change]) -> anyhow::Result<()> {
    let file_exists = PathBuf::from(CHANGELOG_FILENAME).exists();
    let mut contents = String::new();
    if !file_exists {
        contents.push_str("# Changelog\n\n");
    }
    let mut file = File::options()
        .create(true)
        .read(true)
        .write(true)
        .truncate(false)
        .open(CHANGELOG_FILENAME)?;

    file.seek(io::SeekFrom::Start(0))?;

    file.read_to_string(&mut contents)?;

    let contents_to_insert = generate_changelog_contents(next_version, changesets);

    let new_contents = insert_before(&contents, "## ", &contents_to_insert);

    file.seek(io::SeekFrom::Start(0))?;
    write!(file, "{new_contents}")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case::major(vec![
        Change {
            bump_type: IncrementType::Major,
            file_path: PathBuf::new(),
        },
        Change {
            bump_type: IncrementType::Minor,
            file_path: PathBuf::new(),
        },
        Change {
            bump_type: IncrementType::Patch,
            file_path: PathBuf::new(),
        },
    ], Some(IncrementType::Major))]
    #[case::minor(vec![
        Change {
            bump_type: IncrementType::Minor,
            file_path: PathBuf::new(),
        },
        Change {
            bump_type: IncrementType::Minor,
            file_path: PathBuf::new(),
        },
        Change {
            bump_type: IncrementType::Patch,
            file_path: PathBuf::new(),
        },
    ], Some(IncrementType::Minor))]
    #[case::patch(vec![
        Change {
            bump_type: IncrementType::Patch,
            file_path: PathBuf::new(),
        },
        Change {
            bump_type: IncrementType::Patch,
            file_path: PathBuf::new(),
        },
    ], Some(IncrementType::Patch))]
    fn test_determine_final_bump_type_selects_correct_bump_type(
        #[case] input: Vec<Change>,
        #[case] expected: Option<IncrementType>,
    ) {
        let result = determine_final_bump_type(&input).unwrap();
        assert_eq!(result, expected);
    }

    #[rstest]
    fn test_insert_before_inserts_contents_before_search() {
        let contents = insert_before("line1\nline2\nline3\n", "line2", "new line\n");

        assert_eq!(contents, "line1\nnew line\nline2\nline3\n");
    }

    #[rstest]
    fn test_insert_before_inserts_contents_before_search_only_once() {
        let contents = insert_before("line1\nline2\nline3\nline2\n", "line2", "new line\n");

        assert_eq!(contents, "line1\nnew line\nline2\nline3\nline2\n");
    }

    #[rstest]
    fn test_insert_before_inserts_contents_before_search_partial_match() {
        let contents = insert_before("line1\nline2\nline3\n", "line", "new line\n");

        assert_eq!(contents, "new line\nline1\nline2\nline3\n");
    }
}
