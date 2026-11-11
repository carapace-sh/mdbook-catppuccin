use serde_json::{json, Value};
use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::Path;
use std::process::Command;

const CATPPUCCIN_CSS_URL: &str =
    "https://github.com/catppuccin/mdBook/releases/latest/download/catppuccin.css";

const INJECT_SCRIPT: &str = r#"<script>
(function() {
  function addCatppuccinThemes() {
    var list = document.getElementById("mdbook-theme-list") || document.getElementById("theme-list");
    if (!list) return;
    var themes = [
      ["latte", "Latte"],
      ["frappe", "Frappé"],
      ["macchiato", "Macchiato"],
      ["mocha", "Mocha"]
    ];
    for (var i = 0; i < themes.length; i++) {
      var id = "mdbook-theme-" + themes[i][0];
      if (document.getElementById(id)) continue;
      var li = document.createElement("li");
      li.setAttribute("role", "none");
      var btn = document.createElement("button");
      btn.setAttribute("role", "menuitem");
      btn.className = "theme";
      btn.id = id;
      btn.textContent = themes[i][1];
      li.appendChild(btn);
      list.appendChild(li);
    }
  }
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", addCatppuccinThemes);
  } else {
    addCatppuccinThemes();
  }
})();
</script>"#;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1] == "supports" {
        std::process::exit(0);
    }

    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut parsed: Value = serde_json::from_str(&input).unwrap();

    let root = parsed[0].get("root").and_then(|v| v.as_str()).unwrap_or("");
    if !root.is_empty() {
        ensure_css(root);
    }

    let mut book = parsed[1].clone();
    process_book(&mut book);
    parsed[1] = book;

    ensure_additional_css(&mut parsed[0]);

    println!("{}", serde_json::to_string(&parsed).unwrap());
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

fn ensure_additional_css(ctx: &mut Value) {
    let css = Value::String("./theme/catppuccin.css".to_string());
    let html = match ctx.pointer_mut("/config/output/html") {
        Some(v) => match v.as_object_mut() {
            Some(o) => o,
            None => return,
        },
        None => return,
    };

    match html.get_mut("additional-css") {
        Some(Value::Array(arr)) => {
            if !arr
                .iter()
                .any(|v| v.as_str() == Some("./theme/catppuccin.css"))
            {
                arr.push(css);
            }
        }
        Some(_) => {
            html.insert(
                "additional-css".to_string(),
                json!(["./theme/catppuccin.css"]),
            );
        }
        None => {
            html.insert(
                "additional-css".to_string(),
                json!(["./theme/catppuccin.css"]),
            );
        }
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
    ch["content"] = Value::String(format!(
        "<!-- mdbook-catppuccin -->\n{}\n{}",
        INJECT_SCRIPT, content
    ));
}
