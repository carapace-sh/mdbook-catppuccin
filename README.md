# mdbook-catppuccin

mdBook preprocessor that applies the [Catppuccin](https://catppuccin.com) theme.

- Retrieves `catppuccin.css` from the official [catppuccin/mdBook](https://github.com/catppuccin/mdBook) release asset into `theme/catppuccin.css` if missing.
- Ensures `output.html.additional-css` includes `./theme/catppuccin.css` (existing additional-css mechanism).
- Injects a small runtime script that adds the Catppuccin theme entries (`Latte`, `Frappé`, `Macchiato`, `Mocha`) to the theme picker, so no `theme/index.hbs` template file is required in the book.

## Setup

Add to `book.toml`:

```toml
[preprocessor.catppuccin]

[output.html]
default-theme = "Latte"
preferred-dark-theme = "Mocha"
```
