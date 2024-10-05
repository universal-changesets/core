use crate::changeset::{Change, IncrementType};
use crate::utils;
use semver::Version;

pub const CHANGELOG_FILENAME: &str = "CHANGELOG.md";

pub fn generate_changelog_contents(next_version: &Version, changesets: &[Change]) -> String {
    if changesets.is_empty() {
        return String::new();
    }

    let mut contents = format!("## {next_version}\n\n");

    let breaking_changes = changesets
        .iter()
        .filter(|c| c.bump_type == IncrementType::Major)
        .collect::<Vec<_>>();
    let features = changesets
        .iter()
        .filter(|c| c.bump_type == IncrementType::Minor)
        .collect::<Vec<_>>();
    let patches = changesets
        .iter()
        .filter(|c| c.bump_type == IncrementType::Patch)
        .collect::<Vec<_>>();

    if let Some(section) = generate_section("Breaking Changes", breaking_changes) {
        contents.push_str(&section);
        if !features.is_empty() || !patches.is_empty() {
            contents.push_str("\n\n");
        }
    }

    if let Some(section) = generate_section("Features", features) {
        contents.push_str(&section);
        if !patches.is_empty() {
            contents.push_str("\n\n");
        }
    }

    if let Some(section) = generate_section("Patches", patches) {
        contents.push_str(&section);
    }

    contents
}

fn generate_section(title: &str, changes: Vec<&Change>) -> Option<String> {
    if changes.is_empty() {
        return None;
    }

    let mut section = format!("### {title}\n\n");
    let changes_contents = changes
        .iter()
        .map(|c| {
            let mut contents = format!("#### {}", c.summary);
            if !c.description.is_empty() {
                contents.push_str("\n\n");
                contents.push_str(&c.description);
            }
            contents
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    section.push_str(&changes_contents);
    Some(section)
}

/// Creates, or updates a CHANGELOG.md file with the contents of the changesets
pub fn generate_changelog(
    existing_changelog: &str,
    next_version: &Version,
    changesets: &[Change],
) -> anyhow::Result<String> {
    let mut contents = String::new();
    if existing_changelog.is_empty() {
        contents.push_str("# Changelog");
    } else {
        contents.push_str(existing_changelog.trim());
    }
    contents.push_str("\n\n");

    let mut contents_to_insert = generate_changelog_contents(next_version, changesets);
    contents_to_insert.push_str("\n\n");

    let mut new_contents = utils::insert_before(&contents, "## ", &contents_to_insert);
    new_contents = new_contents.trim_end().to_string();
    new_contents.push('\n');

    Ok(new_contents)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::changeset::IncrementType;
    use pretty_assertions::assert_eq;
    use rstest::rstest;
    use std::path::PathBuf;

    #[rstest]
    #[case(vec![], "")]
    #[case(vec![
        Change {
            bump_type: IncrementType::Major,
            summary: "Breaking change 1".to_string(),
            description: "".to_string(),
            file_path: PathBuf::new(),
        },
    ], "## 1.2.3\n\n### Breaking Changes\n\n#### Breaking change 1")]
    #[case(vec![
        Change {
            bump_type: IncrementType::Major,
            summary: "Breaking change 1".to_string(),
            description: "This is the text for the breaking change".to_string(),
            file_path: PathBuf::new(),
        },
    ], "## 1.2.3\n\n### Breaking Changes\n\n#### Breaking change 1\n\nThis is the text for the breaking change")]
    #[case(vec![
        Change {
            bump_type: IncrementType::Minor,
            summary: "Feature 1".to_string(),
            description: "feature description".to_string(),
            file_path: PathBuf::new(),
        },
    ], "## 1.2.3\n\n### Features\n\n#### Feature 1\n\nfeature description")]
    #[case(vec![
        Change {
            bump_type: IncrementType::Patch,
            summary: "Patch 1".to_string(),
            description: "patch description".to_string(),
            file_path: PathBuf::new(),
        },
    ], "## 1.2.3\n\n### Patches\n\n#### Patch 1\n\npatch description")]
    #[case(vec![
        Change {
            bump_type: IncrementType::Major,
            summary: "Breaking change 1".to_string(),
            description: "This is the text for the breaking change".to_string(),
            file_path: PathBuf::new(),
        },
        Change {
            bump_type: IncrementType::Patch,
            summary: "Patch 1".to_string(),
            description: "This is the text for the patch".to_string(),
            file_path: PathBuf::new(),
        },
    ], "## 1.2.3\n\n### Breaking Changes\n\n#### Breaking change 1\n\nThis is the text for the breaking change\n\n### Patches\n\n#### Patch 1\n\nThis is the text for the patch")]
    #[case(vec![
        Change {
            bump_type: IncrementType::Major,
            summary: "Breaking change 1".to_string(),
            description: "This is the text for the breaking change".to_string(),
            file_path: PathBuf::new(),
        },
        Change {
            bump_type: IncrementType::Minor,
            summary: "Feature 1".to_string(),
            description: "This is the text for the feature".to_string(),
            file_path: PathBuf::new(),
        },
        Change {
            bump_type: IncrementType::Patch,
            summary: "Patch 1".to_string(),
            description: "This is the text for the patch".to_string(),
            file_path: PathBuf::new(),
        },
    ], "## 1.2.3\n\n### Breaking Changes\n\n#### Breaking change 1\n\nThis is the text for the breaking change\n\n### Features\n\n#### Feature 1\n\nThis is the text for the feature\n\n### Patches\n\n#### Patch 1\n\nThis is the text for the patch")]
    #[case(vec![
        Change {
            bump_type: IncrementType::Major,
            summary: "Breaking change 1".to_string(),
            description: "This is the text for the breaking change".to_string(),
            file_path: PathBuf::new(),
        },
        Change {
            bump_type: IncrementType::Major,
            summary: "Breaking change 2".to_string(),
            description: "This is the text for the breaking change again".to_string(),
            file_path: PathBuf::new(),
        },
    ], "## 1.2.3\n\n### Breaking Changes\n\n#### Breaking change 1\n\nThis is the text for the breaking change\n\n#### Breaking change 2\n\nThis is the text for the breaking change again")]
    fn test_generate_changelog_contents(#[case] changes: Vec<Change>, #[case] expected: &str) {
        let version = Version::new(1, 2, 3);

        let changelog_contents = generate_changelog_contents(&version, &changes);

        assert_eq!(changelog_contents, expected);
    }

    #[rstest]
    #[case(vec![Change {
        bump_type: IncrementType::Major,
        summary: "test".to_string(),
        description: "".to_string(),
        file_path: PathBuf::new(),
    }], "# Changelog\n", "# Changelog\n\n## 1.2.3\n\n### Breaking Changes\n\n#### test\n")]
    #[case(vec![Change {
        bump_type: IncrementType::Major,
        summary: "test".to_string(),
        description: "".to_string(),
        file_path: PathBuf::new(),
    }], "# Changelog\n\n## 1.2.2\n\n### Breaking Changes\n\n#### test\n", "# Changelog\n\n## 1.2.3\n\n### Breaking Changes\n\n#### test\n\n## 1.2.2\n\n### Breaking Changes\n\n#### test\n")]
    fn test_generate_changelog_generates_correct_contents(
        #[case] changes: Vec<Change>,
        #[case] existing_changelog: &str,
        #[case] expected: &str,
    ) {
        let changelog =
            generate_changelog(existing_changelog, &Version::new(1, 2, 3), &changes).unwrap();

        assert_eq!(changelog, expected);
    }
}
