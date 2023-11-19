use mdbook::{
    preprocess::{CmdPreprocessor, Preprocessor, PreprocessorContext},
    book::Book,
    errors::Result,
};

pub struct FilesPreprocessor;

impl Preprocessor for FilesPreprocessor {
    fn name(&self) -> &str {
        "files"
    }

    fn run(&self, ctx: &PreprocessorContext, book: Book) -> Result<Book> {
        Ok(book)
    }
}
