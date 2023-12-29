use anyhow::{bail, Context, Result};
use camino::Utf8PathBuf;
use ignore::{overrides::OverrideBuilder, WalkBuilder};
use log::*;
use mdbook::{
    book::{Book, Chapter},
    errors::Result as MdbookResult,
    preprocess::{Preprocessor, PreprocessorContext},
    BookItem,
};
use pulldown_cmark::{CodeBlockKind, CowStr, Event, Options, Parser, Tag};
use pulldown_cmark_to_cmark::cmark;
use serde::Deserialize;
use std::collections::BTreeMap;
use toml::value::Value;
use uuid::Uuid;

/// Configuration for an invocation of files
#[derive(Deserialize, Debug)]
pub struct Files {
    /// Path to files
    pub path: Utf8PathBuf,

    /// Add a glob to the set of overrides.
    ///
    /// Globs provided here have precisely the same semantics as a single line in a gitignore file,
    /// where the meaning of `!` is inverted: namely, `!` at the beginning of a glob will ignore a
    /// file. Without `!`, all matches of the glob provided are treated as whitelist matches.
    #[serde(default)]
    pub ignore: Vec<String>,

    /// Process ignores case insensitively
    #[serde(default)]
    pub ignore_case_insensitive: bool,

    /// Do not cross file system boundaries.
    ///
    /// When this option is enabled, directory traversal will not descend into directories that are
    /// on a different file system from the root path.
    #[serde(default)]
    pub same_file_system: bool,

    /// Select the file type given by name.
    #[serde(default)]
    pub types: Vec<String>,

    /// Enables ignoring hidden files.
    #[serde(default)]
    pub hidden: bool,

    /// Whether to follow symbolic links or not.
    #[serde(default)]
    pub follow_links: bool,

    /// Enables reading `.ignore` files.
    ///
    /// `.ignore` files have the same semantics as gitignore files and are supported by search
    /// tools such as ripgrep and The Silver Searcher.
    #[serde(default)]
    pub dot_ignore: bool,

    /// Enables reading a global `gitignore` file, whose path is specified in git’s `core.excludesFile`
    /// config option.
    #[serde(default)]
    pub git_global: bool,

    /// Enables reading `.git/info/exclude` files.
    #[serde(default)]
    pub git_exclude: bool,

    /// Enables reading `.gitignore` files.
    #[serde(default)]
    pub git_ignore: bool,

    /// Whether a git repository is required to apply git-related ignore rules (global rules,
    /// .gitignore and local exclude rules).
    #[serde(default)]
    pub require_git: bool,

    /// Enables reading ignore files from parent directories.
    #[serde(default)]
    pub git_ignore_parents: bool,

    /// The maximum depth to recurse.
    #[serde(default)]
    pub max_depth: Option<usize>,

    /// Whether to ignore files above the specified limit.
    #[serde(default)]
    pub max_filesize: Option<u64>,

    #[serde(default)]
    pub height: Option<String>,
}

/// Configuration for the plugin
#[derive(Deserialize)]
pub struct Config {
    pub prefix: Utf8PathBuf,
}

impl Config {
    fn map(&self, book: Book) -> Result<Book> {
        let mut book = book;
        book.sections = std::mem::take(&mut book.sections)
            .into_iter()
            .map(|section| self.map_book_item(section))
            .collect::<Result<_, _>>()?;
        Ok(book)
    }

    fn map_book_item(&self, item: BookItem) -> Result<BookItem> {
        let result = match item {
            BookItem::Chapter(chapter) => BookItem::Chapter(self.map_chapter(chapter)?),
            other => other,
        };

        Ok(result)
    }

    fn map_code<'a>(&self, code: CowStr<'a>) -> Result<Vec<Event<'a>>> {
        let data: Files = toml::from_str(&code).unwrap();
        let uuid = Uuid::new_v4();

        let mut paths: BTreeMap<Utf8PathBuf, Uuid> = Default::default();

        let parent = self.prefix.join(&data.path);
        let mut overrides = OverrideBuilder::new(&parent);
        for item in &data.ignore {
            overrides.add(item)?;
        }
        let overrides = overrides.build()?;
        let mut walker = WalkBuilder::new(&parent);
        walker
            .standard_filters(false)
            .ignore_case_insensitive(data.ignore_case_insensitive)
            .same_file_system(data.same_file_system)
            .require_git(data.require_git)
            .hidden(data.hidden)
            .ignore(data.dot_ignore)
            .git_ignore(data.git_ignore)
            .git_exclude(data.git_exclude)
            .git_global(data.git_global)
            .parents(data.git_ignore_parents)
            .follow_links(data.follow_links)
            .max_depth(data.max_depth)
            .overrides(overrides)
            .max_filesize(data.max_filesize);

        let walker = walker.build();

        for path in walker {
            let path = path?;
            if path.file_type().unwrap().is_file() {
                paths.insert(path.path().to_path_buf().try_into()?, Uuid::new_v4());
            }
        }

        info!("Found {} matching files", paths.len());
        if paths.is_empty() {
            bail!("No files matched");
        }

        let mut events = vec![];

        let height = data.height.as_deref().unwrap_or("300px");
        events.push(Event::Html(CowStr::Boxed(
            format!(r#"<div id="files-{uuid}" class="mdbook-files" style="height: {height};">"#)
                .into(),
        )));

        events.push(Event::Html(CowStr::Boxed(
            r#"<div class="mdbook-files-left">"#.to_string().into(),
        )));

        events.push(Event::Html(CowStr::Boxed(r#"<ul>"#.to_string().into())));
        for (path, uuid) in &paths {
            let path = path.strip_prefix(&parent)?;
            events.push(Event::Html(CowStr::Boxed(
                format!(r#"<li id="button-{uuid}">{path}</li>"#).into(),
            )));
        }
        events.push(Event::Html(CowStr::Boxed(r#"</ul>"#.to_string().into())));

        events.push(Event::Html(CowStr::Boxed("</div>".to_string().into())));

        events.push(Event::Html(CowStr::Boxed(
            r#"<div class="mdbook-files-right">"#.to_string().into(),
        )));

        for (path, uuid) in &paths {
            info!("Reading {path}");
            let contents = std::fs::read_to_string(path)?;
            let extension = path.extension().unwrap_or("");
            let tag = Tag::CodeBlock(CodeBlockKind::Fenced(CowStr::Boxed(extension.into())));

            events.push(Event::Html(CowStr::Boxed(
                format!(r#"<div id="file-{uuid}" class="mdbook-file visible">"#).into(),
            )));

            events.push(Event::Start(tag.clone()));
            events.push(Event::Text(CowStr::Boxed(contents.into())));
            events.push(Event::End(tag));

            events.push(Event::Html(CowStr::Boxed("</div>".to_string().into())));
        }

        events.push(Event::Html(CowStr::Boxed("</div>".to_string().into())));
        events.push(Event::Html(CowStr::Boxed("</div>".to_string().into())));

        let uuids: Vec<String> = paths.values().map(|uuid| uuid.to_string()).collect();
        events.push(Event::Html(CowStr::Boxed(format!(r#"<script>
            window.addEventListener("load", (event) => {{
                const uuids = {uuids:?};
                function set_visible(uuid) {{
                    uuids.forEach((uuid) => {{
                        document.getElementById(`button-${{uuid}}`).classList.remove("active");
                        document.getElementById(`file-${{uuid}}`).classList.remove("visible");
                    }});
                    const button = document.getElementById(`button-${{uuid}}`).classList.add("active");
                    const file = document.getElementById(`file-${{uuid}}`).classList.add("visible");
                }}
                function add_hook(uuid) {{
                    const button = document.getElementById(`button-${{uuid}}`);
                    button.addEventListener("click", (event) => set_visible(uuid));
                }}
                uuids.forEach((uuid) => add_hook(uuid));
                set_visible(uuids[0]);
            }});
        </script>"#).into())));

        events.push(Event::HardBreak);
        Ok(events)
    }

    fn label(&self) -> &str {
        "files"
    }

    fn map_chapter(&self, mut chapter: Chapter) -> Result<Chapter> {
        chapter.content = self.map_markdown(&chapter.content)?;
        chapter.sub_items = std::mem::take(&mut chapter.sub_items)
            .into_iter()
            .map(|item| self.map_book_item(item))
            .collect::<Result<_, _>>()?;
        Ok(chapter)
    }

    fn map_markdown(&self, markdown: &str) -> Result<String> {
        let mut parser = Parser::new_ext(markdown, Options::all());
        let mut events = vec![];

        loop {
            let next = parser.next();
            match next {
                None => break,
                Some(Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(label))))
                    if &*label == self.label() =>
                {
                    let mapped = match parser.next() {
                        Some(Event::Text(code)) => self.map_code(code).context("Mapping code")?,
                        other => unreachable!("Got {other:?}"),
                    };

                    for event in mapped.into_iter() {
                        events.push(event);
                    }

                    parser.next();
                }
                Some(event) => events.push(event),
            }
        }

        let mut buf = String::with_capacity(markdown.len());
        let output = cmark(events.iter(), &mut buf).map(|_| buf)?;
        Ok(output)
    }
}

pub struct FilesPreprocessor;

impl Preprocessor for FilesPreprocessor {
    fn name(&self) -> &str {
        "files"
    }

    fn run(&self, ctx: &PreprocessorContext, book: Book) -> MdbookResult<Book> {
        let config = ctx.config.get_preprocessor(self.name()).unwrap();
        let config: Config = Value::Table(config.clone()).try_into().unwrap();
        config.map(book)
    }
}
