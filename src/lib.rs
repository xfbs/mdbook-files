use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use mdbook::{
    book::{Book, Chapter},
    errors::Result as MdbookResult,
    preprocess::{Preprocessor, PreprocessorContext},
    BookItem,
};
use pulldown_cmark::{CodeBlockKind, CowStr, Event, HeadingLevel, Options, Parser, Tag};
use pulldown_cmark_to_cmark::cmark;
use serde::Deserialize;
use std::collections::BTreeMap;
use toml::value::Value;
use uuid::Uuid;

/// Configuration for an invocation of files
#[derive(Deserialize, Debug)]
pub struct Files {
    pub files: Vec<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
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

        for path in &data.files {
            let full_glob = self.prefix.join(path);
            let globs = glob::glob(full_glob.as_str()).context("Globbing files")?;
            for path in globs {
                let path: Utf8PathBuf = path?.try_into()?;
                let path = path.strip_prefix(&self.prefix)?;
                paths.insert(path.into(), Uuid::new_v4());
            }
        }

        let mut events = vec![];

        if let Some(title) = &data.title {
            events.push(Event::Start(Tag::Heading(HeadingLevel::H5, None, vec![])));
            events.push(Event::Text(CowStr::Boxed(title.to_string().into())));
            events.push(Event::End(Tag::Heading(HeadingLevel::H5, None, vec![])));
        }

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
            let contents = std::fs::read_to_string(self.prefix.join(path))?;
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
        let mut parser = Parser::new_ext(&markdown, Options::all());
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
