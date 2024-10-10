# Changeset

This is honestly just a fun side project I'm working on with no intentions of using it in any production-like setting.

It's based on the existing [changesets](https://github.com/changesets/changesets) tool, but I wanted to make it more generic and not tied to a specific project.

## Usage

### Adding a changeset

When you're ready to make a change you can run the `changeset add` command to produce a changeset file which will contain the type of change being made and a message describing the change.

The changeset can be edited to provide further details and documentation about the change. Anything added will be surfaced to the `CHANGELOG.md` file when the changeset is consumed.

```bash
changeset add --bump-type major --message "Added a new feature" # or simply `changeset add` to prompt for the bump type and message
```

| Argument      | Description                                     | Default |
| ------------- | ----------------------------------------------- | ------- |
| `--bump-type` | The type of bump to perform                     |         |
| `--message`   | The summary message to include in the changeset |         |

### Getting the current version

This is mainly a helper for CI or other scripts, but you can run the `changeset get-version` command to get the current version of the project.

```bash
changeset get-version
# 1.2.3
```

### Consuming changesets

```bash
changeset version
```

### Previewing the `CHANGELOG.md` file

```bash
changeset preview changelog
```

### Previewing the next version

```bash
changeset preview version
```

A dry run can be performed by passing the `--dry-run` flag.

This will output the highest version type found in the `.changeset` directory and the changesets that were found.

## Plugins

### VersionedFile

This plugin is used to read/write the version to/from a plain file.

The file must be a plain text file with the following format:

```text
1.2.3
```

Example config file:

```json
{
  "plugin": {
    "name": "versionfile",
    "sha256": "beef1de60035053ad01eff83875999dc9918a65e1cffc006fca95c3bfbe55d70",
    "url": "https://github.com/alex-way/changesets-go-versionfile-plugin/releases/download/0.0.2/versionfile.wasm",
    "versionedFile": ".changeset/version"
  }
}
```

## Implementing your own plugin
