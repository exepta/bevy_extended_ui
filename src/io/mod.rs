use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, LoadContext};
use bevy::prelude::*;
use std::path::{Path, PathBuf};

#[derive(Asset, TypePath, Debug, Clone)]
pub struct CssAsset {
    pub text: String,
}

pub const DEFAULT_UI_CSS_TEXT: &str = include_str!("../../assets/default/extended_ui.css");

#[derive(Resource, Clone)]
pub struct DefaultCssHandle(pub Handle<CssAsset>);

#[derive(Asset, TypePath, Debug, Clone)]
pub struct HtmlAsset {
    pub html: String,
    pub stylesheets: Vec<Handle<CssAsset>>,
}

#[derive(Default, TypePath)]
pub struct CssLoader;

#[derive(Default, TypePath)]
pub struct HtmlLoader;

impl AssetLoader for CssLoader {
    type Asset = CssAsset;
    type Settings = ();
    type Error = std::io::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<CssAsset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        Ok(CssAsset {
            text: String::from_utf8_lossy(&bytes).to_string(),
        })
    }

    fn extensions(&self) -> &[&str] {
        &["css"]
    }
}

impl AssetLoader for HtmlLoader {
    type Asset = HtmlAsset;
    type Settings = ();
    type Error = std::io::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        load_context: &mut LoadContext<'_>,
    ) -> Result<HtmlAsset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let html = String::from_utf8_lossy(&bytes).to_string();

        let base_dir: PathBuf = load_context
            .path()
            .path()
            .parent()
            .unwrap_or(Path::new(""))
            .to_path_buf();

        let css_paths = extract_css_links_lenient(&html);

        let mut stylesheets = Vec::new();
        for css in css_paths {
            let resolved = resolve_relative(&base_dir, &css);
            let handle: Handle<CssAsset> = load_context.load(resolved);
            stylesheets.push(handle);
        }

        Ok(HtmlAsset { html, stylesheets })
    }

    fn extensions(&self) -> &[&str] {
        &["html", "htm"]
    }
}

pub struct ExtendedIoPlugin;

impl Plugin for ExtendedIoPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<HtmlAsset>();
        app.init_asset_loader::<HtmlLoader>();
        app.init_asset::<CssAsset>();
        app.init_asset_loader::<CssLoader>();
        app.add_systems(Startup, register_default_css_asset);
    }
}

fn register_default_css_asset(mut commands: Commands, mut css_assets: ResMut<Assets<CssAsset>>) {
    let handle = css_assets.add(CssAsset {
        text: DEFAULT_UI_CSS_TEXT.to_string(),
    });

    commands.insert_resource(DefaultCssHandle(handle));
}

fn resolve_relative(base_dir: &PathBuf, raw: &str) -> PathBuf {
    let p = PathBuf::from(raw.trim());
    if p.is_absolute() { p } else { base_dir.join(p) }
}

fn extract_css_links_lenient(html: &str) -> Vec<String> {
    let mut out = Vec::new();

    for chunk in html.split("<link").skip(1) {
        let tag = chunk.split('>').next().unwrap_or("");

        let rel_ok = tag.contains("rel=\"stylesheet\"")
            || tag.contains("rel='stylesheet'")
            || tag.contains("ref=\"text/css\"")
            || tag.contains("ref='text/css'");

        if !rel_ok {
            continue;
        }

        if let Some(v) = extract_attr(tag, "href").or_else(|| extract_attr(tag, "src")) {
            out.push(v);
        }
    }

    out
}

fn extract_attr(tag: &str, name: &str) -> Option<String> {
    let needle = format!("{name}=");
    let idx = tag.find(&needle)?;
    let rest = &tag[idx + needle.len()..].trim_start();
    let quote = rest.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let end = rest[1..].find(quote)?;
    Some(rest[1..1 + end].to_string())
}
