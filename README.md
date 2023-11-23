# mdbook-files

A (work-in-progress) preprocessor for [mdBook][] which renders files from a directory
as an interactive widget. The idea here is to make it easy to show project layouts
concisely, without needing too much space to render every file sequentially.

## Usage

Put the following into your `book.toml`:

```toml
[preprocessor.files]
prefix = "examples"
```

It is mandatory to give a prefix. Every include path in the book must be within
this prefix.

To use it, add something like this to your book:

    ```files
    paths = ["subfolder/**"]
    title = "Files in subfolder"
    ```

This will produce a widget with all files in `examples/subfolder`, with the given
title. The content of this is a TOML document which contains configuration.

## License

MIT.

[mdBook]: https://github.com/rust-lang/mdBook/
