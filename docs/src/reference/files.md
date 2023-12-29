# Files

To render files, a pseudo-code block needs to be added to your markdown source
that looks like this:

~~~markdown
```files
path = "path/to/folder"
# other config
```
~~~

The `mkdbook-files` plugin will pick up on these and replace them with file widgets.

This section explains the options available for every files instance.

```toml
# path to folder to select files to show
path = "path/to/folder"

# Add a glob to the set of overrides.
#
# Globs provided here have precisely the same semantics as a single line in a gitignore file,
# where the meaning of `!` is inverted: namely, `!` at the beginning of a glob will ignore a
# file. Without `!`, all matches of the glob provided are treated as whitelist matches.
ignore = ["*.png", "!*.md"]

# When set, is the default file to show.
default_file = "README.md"

# Process ignores case insensitively
ignore_case_insensitive = false

# Do not cross file system boundaries.
#
# When this option is enabled, directory traversal will not descend into directories that are
# on a different file system from the root path.
same_file_system = false

# Select the file type given by name.
types = ["png", "rust"]

# Enables ignoring hidden files.
hidden = false

# Whether to follow symbolic links or not.
follow_links = false

# Enables reading `.ignore` files.
#
# `.ignore` files have the same semantics as gitignore files and are supported by search
# tools such as ripgrep and The Silver Searcher.
dot_ignore = false

# Enables reading a global `gitignore` file, whose path is specified in git’s `core.excludesFile`
# config option.
git_global = false

# Enables reading `.git/info/exclude` files.
git_exclude = false

# Enables reading `.gitignore` files.
git_ignore = false

# Whether a git repository is required to apply git-related ignore rules (global rules,
# .gitignore and local exclude rules).
require_git = false

# Enables reading ignore files from parent directories.
git_ignore_parents = false

# Maximum depth to recurse.
#max_depth = 1234

# Ignore files above the specified limit.
#max_filesize = 10000
```
