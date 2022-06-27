# Description
An mdbook preprocessor which allows inline mathjax with the '$...$' delimeters.
Note, the implementation is pretty ad hoc and may not be very optimised.

# Installation & Use

To install, run in your terminal:
```
cargo install mdbook-inline-mathjax
```

To use in your project, add `[preprocessor.inline-mathjax]` to your `book.toml` file.