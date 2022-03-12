use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::Result;
use once_cell::sync::Lazy;
use tera::{Context, Tera};

use crate::{entity::Entity, Zine, ZINE_FILE};

static TEMPLATE_DIR: &'static str = "templates/*.jinja";

static TERA: Lazy<Tera> = Lazy::new(|| {
    let mut tera = Tera::new(TEMPLATE_DIR).expect("Invalid template dir.");
    tera.register_function("featured", featured_fn);
    tera
});

#[derive(Debug)]
pub struct ZineEngine {
    source: PathBuf,
    dest: PathBuf,
}

#[derive(Debug, Clone)]
pub struct Render {
    context: Context,
}

impl Render {
    pub fn new(context: Context) -> Self {
        Render { context }
    }

    pub fn insert<T: serde::Serialize + ?Sized, S: Into<String>>(&mut self, key: S, val: &T) {
        self.context.insert(key, val);
    }

    pub fn render(&self, template: &str, dest_path: impl AsRef<Path>) -> Result<()> {
        let mut buf = vec![];
        let dest = dest_path.as_ref().join("index.html");
        TERA.render_to(template, &self.context, &mut buf)?;
        if let Some(parent_dir) = dest.parent() {
            if !parent_dir.exists() {
                fs::create_dir_all(&parent_dir)?;
            }
        }
        File::create(dest)?.write_all(&buf)?;
        Ok(())
    }
}

impl ZineEngine {
    pub fn new<P: AsRef<Path>>(source: P, dest: P) -> Result<Self> {
        let dest = dest.as_ref().to_path_buf();
        if !dest.exists() {
            fs::create_dir_all(&dest)?;
        }

        Ok(ZineEngine {
            source: source.as_ref().to_path_buf(),
            dest,
        })
    }

    pub fn bootstrap(&self) -> Result<()> {
        let content = fs::read_to_string(&self.source.join(ZINE_FILE))?;
        let mut zine = toml::from_str::<Zine>(&content)?;
        let mut context = Context::new();
        context.insert("theme", &zine.theme);
        context.insert("site", &zine.site);

        let render = Render::new(context);
        zine.parse(&self.source)?;
        zine.render(render, &self.dest)?;
        println!("Zine engine: {:?}", zine);
        Ok(())
    }
}

// A tera function to filter featured articles.
fn featured_fn(
    map: &std::collections::HashMap<String, serde_json::Value>,
) -> tera::Result<serde_json::Value> {
    if let Some(serde_json::Value::Array(articles)) = map.get("articles") {
        Ok(serde_json::Value::Array(
            articles
                .iter()
                .filter(|article| article.get("featured") == Some(&serde_json::Value::Bool(true)))
                .cloned()
                .collect(),
        ))
    } else {
        Ok(serde_json::Value::Array(vec![]))
    }
}
