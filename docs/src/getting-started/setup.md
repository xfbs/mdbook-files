# Setup

To get started with `mdbook-files`, the next step is to add it to your `mdbook`
configuration.  To do this, you have two options: the automatic install method,
and the manual installation method. The former method is recommended, but the
latter is still documented here in case you run into issues.

### Automatic setup

To install `mdbook-files` automatically, given that you have already installed it,
you can run this command:

    mdbook-files install

This will run perform the same steps as the manual installation method.

### Manual setup

Next, setup your project by adding this to the configuration:

```toml
[preprocessor.files]
prefix = "."

[output.html]
additional-css = ["style.css"]
```

Next, you need to add the `style.css` file into your project, by copying it
from the repository.

## Verify

Once you have done this, you should be able to run `mdbook` to build your book
and not get any issues.

    mdbook build

If this succeeds, you are ready for the next step.
