use mdbook::{
    book::{Book, Chapter},
    errors::{Error, Result},
    preprocess::{Preprocessor, PreprocessorContext},
    BookItem,
};
use pulldown_cmark::{CodeBlockKind, CowStr, Event, Parser, Tag};
use pulldown_cmark_to_cmark::cmark;
use serde::Deserialize;
use std::path::PathBuf;
use toml::value::Value;

/// Configuration for an invocation of files
#[derive(Deserialize, Debug)]
pub struct Files {
    pub path: PathBuf,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub include: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
}

/// Configuration for the plugin
#[derive(Deserialize)]
pub struct Config {
    pub prefix: PathBuf,
}

impl Config {
    fn map(&self, book: Book) -> Book {
        let mut book = book;
        book.for_each_mut(|chapter| {
            self.map_book_item(chapter);
        });
        book
    }

    fn map_book_item(&self, item: &mut BookItem) {
        match item {
            BookItem::Chapter(chapter) => {
                *chapter = self.map_chapter(std::mem::take(chapter));
            }
            _ => {}
        }
    }

    fn map_code<'a>(&self, code: CowStr<'a>) -> Event<'a> {
        let files: Files = toml::from_str(&*code).unwrap();
        let string = format!("<pre><code>{files:?}</code></pre>");
        Event::Html(CowStr::Boxed(string.into()))
    }

    fn label(&self) -> &str {
        "files"
    }

    fn map_chapter(&self, chapter: Chapter) -> Chapter {
        let mut parser = Parser::new(&chapter.content);
        let mut events = vec![];

        loop {
            match parser.next() {
                None => break,
                Some(Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(label))))
                    if &*label == self.label() =>
                {
                    let html = match parser.next() {
                        Some(Event::Text(code)) => self.map_code(code),
                        other => unreachable!("Got {other:?}"),
                    };
                    events.push(html);
                    parser.next();
                }
                Some(event) => events.push(event),
            }
        }

        let mut buf = String::with_capacity(chapter.content.len());
        let output = cmark(events.iter(), &mut buf).map(|_| buf).unwrap();
        let mut chapter = chapter;
        chapter.content = output;
        chapter
    }
}

pub struct FilesPreprocessor;

impl Preprocessor for FilesPreprocessor {
    fn name(&self) -> &str {
        "files"
    }

    fn run(&self, ctx: &PreprocessorContext, book: Book) -> Result<Book> {
        let config = ctx.config.get_preprocessor(self.name()).unwrap();
        let config: Config = Value::Table(config.clone()).try_into().unwrap();
        Ok(config.map(book))
    }
}
