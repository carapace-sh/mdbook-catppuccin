use serde_json::Value;
use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::Path;
use std::process::Command;

const CATPPUCCIN_CSS_URL: &str =
    "https://github.com/catppuccin/mdBook/releases/latest/download/catppuccin.css";

const THEME_JS: &str = r##"(function() {
  if (window.__mdbookCatppuccinInit) return;
  window.__mdbookCatppuccinInit = true;

  var catppuccin = ["latte", "frappe", "macchiato", "mocha"];
  var labels = { latte: "Latte", frappe: "Frappé", macchiato: "Macchiato", mocha: "Mocha" };

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

    var s = getStylesheets();
    if (s.ayuHighlight && s.tomorrowNight && s.highlight) {
      var dark = theme === "coal" || theme === "navy" || theme === "ayu" ||
        (catppuccin.indexOf(theme) !== -1 && theme !== "latte");
      s.ayuHighlight.disabled = dark;
      s.tomorrowNight.disabled = !dark;
      s.highlight.disabled = dark;
    }

    var popup = document.getElementById("mdbook-theme-list") || document.getElementById("theme-list");
    if (popup) { popup.style.display = "none"; }
  }

  function addCatppuccinThemes() {
    var list = document.getElementById("mdbook-theme-list") || document.getElementById("theme-list");
    if (!list) return;
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
        btn.addEventListener("click", function(e) {
          e.preventDefault();
          e.stopPropagation();
          applyTheme(theme);
        });
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
"##;

const HEAD_HBS: &str = r##"<script defer src="{{ resource "theme/catppuccin.js" }}"></script>
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
    let book = &arr[1];

    let root = ctx.get("root").and_then(|v| v.as_str()).unwrap_or("");
    if !root.is_empty() {
        ensure_assets(root);
    }

    println!("{}", serde_json::to_string(book).unwrap());
}

fn ensure_assets(root: &str) {
    let theme_dir = Path::new(root).join("theme");
    if let Err(e) = fs::create_dir_all(&theme_dir) {
        eprintln!("mdbook-catppuccin: {}", e);
        return;
    }

    let css_path = theme_dir.join("catppuccin.css");
    if !css_path.exists() {
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

    let js_path = theme_dir.join("catppuccin.js");
    if let Err(e) = write_if_changed(&js_path, THEME_JS) {
        eprintln!("mdbook-catppuccin: {}", e);
    }

    let head_path = theme_dir.join("head.hbs");
    ensure_head_hbs(&head_path);
}

fn ensure_head_hbs(path: &Path) {
    if path.exists() {
        match fs::read_to_string(path) {
            Ok(existing) => {
                if existing.contains("catppuccin.js") {
                    return;
                }
                let updated = format!("{}{}", HEAD_HBS, existing);
                if let Err(e) = write_if_changed(path, &updated) {
                    eprintln!("mdbook-catppuccin: {}", e);
                }
            }
            Err(e) => eprintln!("mdbook-catppuccin: {}", e),
        }
    } else if let Err(e) = write_if_changed(path, HEAD_HBS) {
        eprintln!("mdbook-catppuccin: {}", e);
    }
}

fn write_if_changed(path: &Path, content: &str) -> io::Result<()> {
    if let Ok(existing) = fs::read_to_string(path) {
        if existing == content {
            return Ok(());
        }
    }
    fs::write(path, content)
}
