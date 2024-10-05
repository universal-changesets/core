/// Inserts the contents before the **first** line that starts with `search`
pub fn insert_before(contents: &str, search: &str, insertable: &str) -> String {
    if let Some(pos) = contents.find(search) {
        let (before, after) = contents.split_at(pos);
        return format!("{}{}{}", before, insertable, after);
    } else {
        return format!("{}{}", contents, insertable);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use rstest::rstest;

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
