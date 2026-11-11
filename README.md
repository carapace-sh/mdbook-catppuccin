# mdbook-catppuccin

mdBook preprocessor that applies the [Catppuccin](https://catppuccin.com) theme without requiring a `theme/index.hbs` template in the book.

- Retrieves `catppuccin.css` from the official [catppuccin/mdBook](https://github.com/catppuccin/mdBook) release asset into `theme/catppuccin.css` if missing.
- Generates `theme/catppuccin.js` to add the Catppuccin theme entries (`Latte`, `Frappé`, `Macchiato`, `Mocha`) to the theme picker at runtime.
- Ensures `theme/head.hbs` loads the Catppuccin stylesheet and script (without requiring or modifying `theme/index.hbs`).

No `theme/index.hbs` template file is required or modified.

## Setup

Add to `book.toml`:

```toml
[preprocessor.catppuccin]

[output.html]
default-theme = "Latte"
preferred-dark-theme = "Mocha"
```

Build once with the installed/updated `mdbook-catppuccin` binary to let the preprocessor create/update `theme/catppuccin.css`, `theme/catppuccin.js`, and `theme/head.hbs`, then rebuild/serve the book.
