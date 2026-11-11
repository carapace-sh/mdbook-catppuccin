use serde_json::Value;
use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::Path;
use std::process::Command;

const CATPPUCCIN_CSS_URL: &str =
    "https://github.com/catppuccin/mdBook/releases/latest/download/catppuccin.css";

const INJECT_SCRIPT: &str = r##"<script>
(function() {
  var catppuccin = ["latte", "frappe", "macchiato", "mocha"];

  function getStylesheets() {
    return {
      ayuHighlight: document.querySelector("#mdbook-ayu-highlight-css"),
      tomorrowNight: document.querySelector("#mdbook-tomorrow-night-css"),
      highlight: document.querySelector("#mdbook-highlight-css"),
    };
  }

  function applyTheme(theme) {
    var html = document.documentElement;
    var previous = null;
    try { previous = localStorage.getItem("mdbook-theme"); } catch (e) {}
    if (previous) { html.classList.remove(previous); }
    html.classList.remove("latte", "frappe", "macchiato", "mocha");
    html.classList.add(theme);
    try { localStorage.setItem("mdbook-theme", theme); } catch (e) {}

    var stylesheets = getStylesheets();
    if (stylesheets.ayuHighlight && stylesheets.tomorrowNight && stylesheets.highlight) {
      if (theme === "coal" || theme === "navy" || theme === "ayu" || (catppuccin.indexOf(theme) !== -1 && theme !== "latte")) {
        stylesheets.ayuHighlight.disabled = true;
        stylesheets.tomorrowNight.disabled = false;
        stylesheets.highlight.disabled = true;
      } else {
        stylesheets.ayuHighlight.disabled = false;
        stylesheets.tomorrowNight.disabled = true;
        stylesheets.highlight.disabled = false;
      }
    }

    var popup = document.getElementById("mdbook-theme-list");
    if (popup) { popup.style.display = "none"; }
  }

  function addCatppuccinThemes() {
    var list = document.getElementById("mdbook-theme-list") || document.getElementById("theme-list");
    if (!list) return;
    var labels = { latte: "Latte", frappe: "Frappé", macchiato: "Macchiato", mocha: "Mocha" };
    for (var i = 0; i < catppuccin.length; i++) {
      (function(theme) {
        var id = "mdbook-theme-" + theme;
        if (document.getElementById(id)) return;
        var li = document.createElement("li");
        li.setAttribute("role", "none");
        var btn = document.createElement("button");
        btn.setAttribute("role", "menuitem");
        btn.className = "theme";
        btn.id = id;
        btn.textContent = labels[theme];
        btn.addEventListener("click", function() { applyTheme(theme); });
        li.appendChild(btn);
        list.appendChild(li);
      })(catppuccin[i]);
    }
  }

  function init() {
    addCatppuccinThemes();
    var current = null;
    try { current = localStorage.getItem("mdbook-theme"); } catch (e) {}
    if (current && catppuccin.indexOf(current) !== -1) {
      applyTheme(current);
    }
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
})();
</script>"##;

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
    if content.is_empty() {
        return;
    }
    if content.contains("<!-- mdbook-catppuccin -->") {
        return;
    }

    let source_path = ch
        .get("source_path")
        .and_then(|v| v.as_str())
        .or_else(|| ch.get("path").and_then(|v| v.as_str()))
        .unwrap_or("");
    let css_href = css_href_for(source_path);

    ch["content"] = Value::String(format!(
        "<!-- mdbook-catppuccin -->\n<link rel=\"stylesheet\" href=\"{}\">\n{}\n{}",
        css_href, INJECT_SCRIPT, content
    ));
}

fn css_href_for(source_path: &str) -> String {
    if source_path.is_empty() {
        return "theme/catppuccin.css".to_string();
    }
    let depth = source_path
        .split('/')
        .filter(|s| !s.is_empty())
        .count()
        .saturating_sub(1);
    format!("{}theme/catppuccin.css", "../".repeat(depth))
}
