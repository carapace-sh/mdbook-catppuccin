# mdbook-catppuccin

mdBook preprocessor that applies the [Catppuccin](https://catppuccin.com) theme without requiring a `theme/index.hbs` template in the book.

- Retrieves `catppuccin.css` from the official [catppuccin/mdBook](https://github.com/catppuccin/mdBook) release asset into `theme/catppuccin.css` if missing.
- Generates `theme/catppuccin.js` to add the Catppuccin theme entries (`Latte`, `Frappé`, `Macchiato`, `Mocha`) to the theme picker at runtime.
- Ensures `theme/head.hbs` loads the generated script (without requiring or modifying `theme/index.hbs`).

The stylesheet is loaded via mdBook's existing `additional-css` mechanism.

## Setup

Add to `book.toml`:

```toml
[preprocessor.catppuccin]

[output.html]
default-theme = "Latte"
preferred-dark-theme = "Mocha"
additional-css = ["./theme/catppuccin.css"]
```

Build once to let the preprocessor create/update theme assets, then rebuild/serve the book.
