use serde_json::Value;
use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use std::process::Command;

const CATPPUCCIN_CSS_URL: &str =
    "https://github.com/catppuccin/mdBook/releases/latest/download/catppuccin.css";

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1] == "supports" {
        std::process::exit(0);
    }

    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let parsed: Value = serde_json::from_str(&input).unwrap();
    let arr = parsed.as_array().unwrap();
    let mut book = arr[1].clone();

    let css = retrieve_css();
    process_book(&mut book, css.as_deref());

    println!("{}", serde_json::to_string(&book).unwrap());
}

fn cache_css_path() -> Option<PathBuf> {
    if let Ok(cache) = env::var("XDG_CACHE_HOME") {
        if !cache.is_empty() {
            return Some(PathBuf::from(cache).join("mdbook-catppuccin/catppuccin.css"));
        }
    }
    env::var("HOME")
        .ok()
        .map(|h| PathBuf::from(h).join(".cache/mdbook-catppuccin/catppuccin.css"))
}

fn retrieve_css() -> Option<String> {
    let path = cache_css_path()?;
    if let Some(parent) = path.parent() {
        if fs::create_dir_all(parent).is_err() {
            return None;
        }
    }
    if !path.exists() {
        let status = Command::new("curl")
            .args(["-fsSL", "-o"])
            .arg(&path)
            .arg(CATPPUCCIN_CSS_URL)
            .status()
            .ok()?;
        if !status.success() {
            let _ = fs::remove_file(&path);
            eprintln!(
                "mdbook-catppuccin: failed to download catppuccin.css (status {})",
                status
            );
            return None;
        }
    }
    match fs::read_to_string(&path) {
        Ok(css) => Some(css),
        Err(e) => {
            eprintln!(
                "mdbook-catppuccin: failed to read cached catppuccin.css: {}",
                e
            );
            None
        }
    }
}

fn process_book(book: &mut Value, css: Option<&str>) {
    let inject = build_inject(css);
    if let Some(items) = book.get_mut("items").and_then(|v| v.as_array_mut()) {
        let n = items.len();
        for i in 0..n {
            process_item(&mut items[i], &inject);
        }
    }
}

fn process_item(item: &mut Value, inject: &str) {
    if let Some(ch) = item.get_mut("Chapter") {
        process_chapter(ch, inject);
        if let Some(subs) = ch.get_mut("sub_items").and_then(|v| v.as_array_mut()) {
            let n = subs.len();
            for i in 0..n {
                process_item(&mut subs[i], inject);
            }
        }
    }
}

fn process_chapter(ch: &mut Value, inject: &str) {
    let content = ch
        .get("content")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    if content.is_empty() || content.contains("<!-- mdbook-catppuccin -->") {
        return;
    }
    ch["content"] = Value::String(format!("{}{}", inject, content));
}

fn build_inject(css: Option<&str>) -> String {
    let css_json = css
        .map(|c| serde_json::to_string(c).unwrap_or_else(|_| "null".to_string()))
        .unwrap_or_else(|| "null".to_string());

    format!(
        r##"<!-- mdbook-catppuccin -->
<script>
(function() {{
  if (window.__mdbookCatppuccinInit) return;
  window.__mdbookCatppuccinInit = true;

  var css = {css_json};
  if (css && !document.getElementById("mdbook-catppuccin-style")) {{
    var style = document.createElement("style");
    style.id = "mdbook-catppuccin-style";
    style.textContent = css;
    document.head.appendChild(style);
  }}

  var catppuccin = ["latte", "frappe", "macchiato", "mocha"];
  var labels = {{ latte: "Latte", frappe: "Frappé", macchiato: "Macchiato", mocha: "Mocha" }};

  function getStylesheets() {{
    return {{
      ayuHighlight: document.querySelector("#mdbook-ayu-highlight-css"),
      tomorrowNight: document.querySelector("#mdbook-tomorrow-night-css"),
      highlight: document.querySelector("#mdbook-highlight-css"),
    }};
  }}

  function applyTheme(theme) {{
    var html = document.documentElement;
    var previous = null;
    try {{ previous = localStorage.getItem("mdbook-theme"); }} catch (e) {{}}
    if (previous) html.classList.remove(previous);
    html.classList.remove("latte", "frappe", "macchiato", "mocha");
    html.classList.add(theme);
    try {{ localStorage.setItem("mdbook-theme", theme); }} catch (e) {{}}

    var s = getStylesheets();
    if (s.ayuHighlight && s.tomorrowNight && s.highlight) {{
      var dark = theme === "coal" || theme === "navy" || theme === "ayu" || (catppuccin.indexOf(theme) !== -1 && theme !== "latte");
      s.ayuHighlight.disabled = dark;
      s.tomorrowNight.disabled = !dark;
      s.highlight.disabled = dark;
    }}

    var popup = document.getElementById("mdbook-theme-list");
    if (popup) popup.style.display = "none";
  }}

  function addCatppuccinThemes() {{
    var list = document.getElementById("mdbook-theme-list") || document.getElementById("theme-list");
    if (!list) return;
    for (var i = 0; i < catppuccin.length; i++) {{
      (function(theme) {{
        var id = "mdbook-theme-" + theme;
        if (document.getElementById(id)) return;
        var li = document.createElement("li");
        li.setAttribute("role", "none");
        var btn = document.createElement("button");
        btn.setAttribute("role", "menuitem");
        btn.className = "theme";
        btn.id = id;
        btn.textContent = labels[theme];
        btn.addEventListener("click", function() {{ applyTheme(theme); }});
        li.appendChild(btn);
        list.appendChild(li);
      }})(catppuccin[i]);
    }}
  }}

  function init() {{
    addCatppuccinThemes();
    var current = null;
    try {{ current = localStorage.getItem("mdbook-theme"); }} catch (e) {{}}
    if (current && catppuccin.indexOf(current) !== -1) {{
      applyTheme(current);
    }}
  }}

  if (document.readyState === "loading") {{
    document.addEventListener("DOMContentLoaded", init);
  }} else {{
    init();
  }}
}})();
</script>
"##,
        css_json = css_json
    )
}
