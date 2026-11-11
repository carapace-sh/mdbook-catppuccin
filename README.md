# mdbook-catppuccin

mdBook preprocessor that applies the [Catppuccin](https://catppuccin.com) theme without requiring a `theme/index.hbs` template in the book.

- Retrieves `catppuccin.css` from the official [catppuccin/mdBook](https://github.com/catppuccin/mdBook) release asset into `theme/catppuccin.css` if missing.
- Injects a runtime script that adds the Catppuccin theme entries (`Latte`, `Frappé`, `Macchiato`, `Mocha`) to the theme picker.

The stylesheet is loaded via mdBook's existing `additional-css` mechanism. No `theme/index.hbs` template file is required or modified.

## Setup

Add to `book.toml`:

```toml
[preprocessor.catppuccin]

[output.html]
default-theme = "Latte"
preferred-dark-theme = "Mocha"
additional-css = ["./theme/catppuccin.css"]
```
