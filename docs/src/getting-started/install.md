# Install

To install `mdbook-files`, you have two options. The easiest installation path is
to download a prebuilt binary from the GitHub releases. If you have Cargo installed,
you can also build it from source.

### Prebuilt

You can go to the [Releases](https://github.com/xfbs/mdbook-files/releases) page and download
whichever prebuilt binary is appropriate for the operating system that you are using.

For example, to install it in Linux, you can do like this. You will need to set
the `MDBOOK_FILES_VERSION` to the latest version (or whichever version you want to
install).

```bash
export MDBOOK_FILES_VERSION=0.1.0
curl -sSL "https://github.com/xfbs/mdbook-files/releases/download/v$MDBOOK_FILES_VERSION/mdbook-files-v$MDBOOK_FILES_VERSION-x86_64-unknown-linux-musl.tar.gz" | sudo tar -C /usr/local/bin -xzv
```

### From Source

Another approach is to install `mdbook-files` from source. This approach works on operating
systems for which no builds exist. To do so, you need to have Cargo[^1] installed.

    cargo install mdbook-files --version 0.1.0

## Verify

Once you have installed it, you should be able to verify that you have installed it correctly
by running this command:

    mdbook-files --version

Which should respond with whichever version you have installed.

[^1]: Use [rustup](https://rustup.rs/) to install it, if neccessary.
