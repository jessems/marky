use crate::paths;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

macro_rules! include_theme {
    ($path:expr) => {
        Theme {
            name: std::path::Path::new($path)
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string(),
            inline: Some(include_str!($path).to_string()),

            path: None,
            url: None,
        }
    };
}

#[derive(serde::Deserialize, Clone)]
pub struct Theme {
    pub name: String,

    path: Option<PathBuf>,
    inline: Option<String>,
    url: Option<url::Url>,
}

impl Theme {
    pub fn resolve(&self) -> Result<String, Box<dyn std::error::Error>> {
        let resolved = {
            if self.inline.is_some() {
                Some(self.resolve_inline()?)
            } else if self.path.is_some() {
                Some(self.resolve_path()?)
            } else if self.url.is_some() {
                Some(self.resolve_url()?)
            } else {
                None
            }
        };

        match resolved {
            Some(css) => match minifier::css::minify(css.as_str()) {
                Ok(minfied) => Ok(minfied.to_string()),
                Err(error) => Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    error,
                ))),
            },
            None => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "theme source is not specified",
            ))),
        }
    }

    fn resolve_inline(&self) -> std::io::Result<String> {
        assert!(self.inline.is_some());

        let inline = self.inline.as_ref().unwrap().to_string();
        Ok(inline)
    }

    fn resolve_path(&self) -> std::io::Result<String> {
        assert!(self.path.is_some());

        let path = {
            let path = self.path.as_ref().unwrap();

            if path.is_relative() {
                paths::dirs::config().join(path)
            } else {
                path.clone()
            }
        };

        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        Ok(contents)
    }

    fn resolve_url(&self) -> std::io::Result<String> {
        assert!(self.url.is_some());

        unimplemented!();
    }
}

impl Default for Theme {
    fn default() -> Self {
        include_theme!("themes/air.css")
    }
}

#[derive(serde::Deserialize)]
pub struct Themes {
    pub themes: Vec<Theme>,
}

impl Themes {
    pub fn by_name(&self, name: &str) -> Option<Theme> {
        self.themes.iter().find(|theme| theme.name == name).cloned()
    }
}

impl Default for Themes {
    fn default() -> Self {
        let mut themes = Vec::new();

        for theme in [
            include_theme!("themes/air.css"),
            include_theme!("themes/modest.css"),
            include_theme!("themes/retro.css"),
            include_theme!("themes/splendor.css"),
        ]
        .into_iter()
        {
            themes.push(theme);
        }

        Themes { themes }
    }
}

pub fn available_themes() -> Result<Themes, Box<dyn Error>> {
    let mut default = Themes::default();

    let themes_path = paths::files::themes();
    if !themes_path.exists() {
        return Ok(default);
    }

    let mut file = File::open(themes_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let mut custom: Themes = toml::from_str(contents.as_str())?;

    default.themes.append(&mut custom.themes);

    Ok(default)
}
