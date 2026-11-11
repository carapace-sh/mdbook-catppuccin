use serde_json::Value;
use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::Path;
use std::process::Command;

const CATPPUCCIN_CSS_URL: &str =
    "https://github.com/catppuccin/mdBook/releases/latest/download/catppuccin.css";

const THEME_SCRIPT: &str = r##"<!-- mdbook-catppuccin -->
<script>
(function() {
  var catppuccin = ["latte", "frappe", "macchiato", "mocha"];
  var labels = { latte: "Latte", frappe: "Frappé", macchiato: "Macchiato", mocha: "Mocha" };

  function addCatppuccinThemes() {
    var list = document.getElementById("mdbook-theme-list") || document.getElementById("theme-list");
    if (!list) return;
    for (var i = 0; i < catppuccin.length; i++) {
      var theme = catppuccin[i];
      var id = "mdbook-theme-" + theme;
      if (document.getElementById(id)) continue;
      var li = document.createElement("li");
      li.setAttribute("role", "none");
      var btn = document.createElement("button");
      btn.type = "button";
      btn.setAttribute("role", "menuitem");
      btn.className = "theme";
      btn.id = id;
      btn.textContent = labels[theme];
      li.appendChild(btn);
      list.appendChild(li);
    }
  }

  function fixHighlight() {
    var theme = null;
    try { theme = localStorage.getItem("mdbook-theme"); } catch (e) {}
    if (!theme) {
      theme = document.documentElement.className.split(/\s+/).filter(Boolean).pop();
    }
    var ayu = document.querySelector("#mdbook-ayu-highlight-css");
    var tomorrow = document.querySelector("#mdbook-tomorrow-night-css");
    var highlight = document.querySelector("#mdbook-highlight-css");
    if (!ayu || !tomorrow || !highlight) return;
    var dark = theme === "coal" || theme === "navy" || theme === "ayu" ||
      theme === "frappe" || theme === "macchiato" || theme === "mocha";
    ayu.disabled = dark;
    tomorrow.disabled = !dark;
    highlight.disabled = dark;
  }

  addCatppuccinThemes();

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", function() {
      addCatppuccinThemes();
      fixHighlight();
    });
  } else {
    fixHighlight();
  }
})();
</script>
"##;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1] == "supports" {
        std::process::exit(0);
    }

    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let parsed: Value = serde_json::from_str(&input).unwrap();
    let arr = parsed.as_array().unwrap();
    let ctx = &arr[0];
    let mut book = arr[1].clone();

    let root = ctx.get("root").and_then(|v| v.as_str()).unwrap_or("");
    if !root.is_empty() {
        ensure_css(root);
    }

    process_book(&mut book);

    println!("{}", serde_json::to_string(&book).unwrap());
}

fn ensure_css(root: &str) {
    let theme_dir = Path::new(root).join("theme");
    if let Err(e) = fs::create_dir_all(&theme_dir) {
        eprintln!("mdbook-catppuccin: {}", e);
        return;
    }

    let css_path = theme_dir.join("catppuccin.css");
    if css_path.exists() {
        return;
    }
    let status = Command::new("curl")
        .args(["-fsSL", "-o"])
        .arg(&css_path)
        .arg(CATPPUCCIN_CSS_URL)
        .status();
    match status {
        Ok(s) if s.success() => {}
        Ok(s) => {
            let _ = fs::remove_file(&css_path);
            eprintln!(
                "mdbook-catppuccin: failed to download catppuccin.css (status {})",
                s
            );
        }
        Err(e) => eprintln!(
            "mdbook-catppuccin: failed to download catppuccin.css: {}",
            e
        ),
    }
}

fn process_book(book: &mut Value) {
    if let Some(items) = book.get_mut("items").and_then(|v| v.as_array_mut()) {
        let n = items.len();
        for i in 0..n {
            process_item(&mut items[i]);
        }
    }
}

fn process_item(item: &mut Value) {
    if let Some(ch) = item.get_mut("Chapter") {
        process_chapter(ch);
        if let Some(subs) = ch.get_mut("sub_items").and_then(|v| v.as_array_mut()) {
            let n = subs.len();
            for i in 0..n {
                process_item(&mut subs[i]);
            }
        }
    }
}

fn process_chapter(ch: &mut Value) {
    let content = ch
        .get("content")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    if content.is_empty() || content.contains("<!-- mdbook-catppuccin -->") {
        return;
    }
    ch["content"] = Value::String(format!("{}{}", THEME_SCRIPT, content));
}
