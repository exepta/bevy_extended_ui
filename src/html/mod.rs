pub mod converter;
pub mod builder;
pub mod reload;

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use bevy::prelude::*;

use crate::html::builder::HtmlBuilderSystem;
use crate::html::converter::HtmlConverterSystem;
use crate::html::reload::HtmlReloadPlugin;

use crate::io::{CssAsset, HtmlAsset};
use crate::styles::parser::apply_property_to_style;
use crate::styles::Style;
use crate::widgets::{Button, Body, Div, Widget, CheckBox, Headline, Paragraph, Img};

pub static HTML_ID_COUNTER: AtomicUsize = AtomicUsize::new(1);

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct HtmlSource {
    pub handle: Handle<HtmlAsset>,
    pub source_id: String,
    pub controller: Option<String>,
}

impl HtmlSource {
    pub fn from_handle(handle: Handle<HtmlAsset>) -> Self {
        Self {
            handle,
            source_id: String::new(),
            controller: None,
        }
    }

    /// Returns the asset path (relative to assets/) of this HtmlAsset.
    /// Example: "examples/test.html"
    pub fn get_source_path(&self) -> String {
        self.handle
            .path()
            .expect("Failed to get source path!")
            .path()
            .to_string_lossy()
            .replace('\\', "/")
    }
}

#[derive(Event, Message)]
pub struct AllWidgetsSpawned;

#[derive(Component)]
pub struct NeedHidden;

#[derive(Resource, Default)]
pub struct ShowWidgetsTimer {
    pub timer: Timer,
    pub active: bool,
}

#[derive(Event, Message)]
pub struct HtmlChangeEvent;

/// A simple explicit "UI needs rebuild" flag.
/// We use this because mutating the internal HashMap of HtmlStructureMap
/// does NOT reliably trigger `resource_changed::<HtmlStructureMap>()`.
#[derive(Resource, Default)]
pub struct HtmlDirty(pub bool);

/// Component storing parsed inline CSS (`style="..."`) as your custom Style struct.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct HtmlStyle(pub Style);

impl HtmlStyle {
    /// Parses inline CSS style declarations ("key: value; ...") into Style.
    pub fn from_str(style_code: &str) -> HtmlStyle {
        let mut style = Style::default();

        for part in style_code.split(';') {
            let trimmed = part.trim();
            if trimmed.is_empty() {
                continue;
            }

            let (name, value) = if let Some((k, v)) = trimmed.split_once(':') {
                (k.trim(), v.trim())
            } else if let Some((k, v)) = trimmed.split_once(' ') {
                (k.trim(), v.trim())
            } else {
                continue;
            };

            apply_property_to_style(&mut style, name, value);
        }

        HtmlStyle(style)
    }
}

#[derive(Debug, Clone, Default)]
pub struct HtmlMeta {
    /// All referenced CSS assets for this node.
    pub css: Vec<Handle<CssAsset>>,
    pub id: Option<String>,
    pub class: Option<Vec<String>>,
    pub style: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct HtmlStates {
    pub hidden: bool,
    pub disabled: bool,
    pub readonly: bool,
}

/// Your current DOM model.
#[derive(Debug, Clone)]
pub enum HtmlWidgetNode {
    /// The root `<body>` element of the HTML structure.
    Body(Body, HtmlMeta, HtmlStates, Vec<HtmlWidgetNode>, HtmlEventBindings, Widget, HtmlID),
    /// A `<div>` container element with nested child nodes.
    Div(Div, HtmlMeta, HtmlStates, Vec<HtmlWidgetNode>, HtmlEventBindings, Widget, HtmlID),
    /// A `<button>` element.
    Button(Button, HtmlMeta, HtmlStates, HtmlEventBindings, Widget, HtmlID),
    /// A checkbox `<input type="checkbox">`.
    CheckBox(CheckBox, HtmlMeta, HtmlStates, HtmlEventBindings, Widget, HtmlID),
    /// A heading element (`<h1>`-`<h6>`).
    Headline(Headline, HtmlMeta, HtmlStates, HtmlEventBindings, Widget, HtmlID),
    /// A img element (`<img>`).
    Img(Img, HtmlMeta, HtmlStates, HtmlEventBindings, Widget, HtmlID),
    /// A paragraph `<p>`.
    Paragraph(Paragraph, HtmlMeta, HtmlStates, HtmlEventBindings, Widget, HtmlID),
}

/// Stores all parsed HTML structures keyed by `<meta name="...">`.
#[derive(Resource)]
pub struct HtmlStructureMap {
    pub html_map: HashMap<String, Vec<HtmlWidgetNode>>,
    pub active: Option<String>,
}

impl Default for HtmlStructureMap {
    fn default() -> Self {
        Self {
            html_map: HashMap::new(),
            active: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Component)]
pub struct HtmlID(pub usize);

impl Default for HtmlID {
    fn default() -> Self {
        Self(HTML_ID_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

/// Function pointer type for click event observers.
type ClickObserverFn = fn(On<Pointer<Click>>, Commands);

/// Function pointer type for mouse over event observers.
type OverObserverFn = fn(On<Pointer<Over>>, Commands);

/// Function pointer type for mouse out event observers.
type OutObserverFn = fn(On<Pointer<Out>>, Commands);

#[derive(Default, Resource)]
pub struct HtmlFunctionRegistry {
    pub click: HashMap<String, ClickObserverFn>,
    pub over: HashMap<String, OverObserverFn>,
    pub out: HashMap<String, OutObserverFn>,
}

#[derive(Component, Reflect, Default, Clone, Debug)]
#[reflect(Component)]
pub struct HtmlEventBindings {
    pub onclick: Option<String>,
    pub onmouseenter: Option<String>,
    pub onmouseleave: Option<String>,
    pub onupdate: Option<String>,
    pub onload: Option<String>,
}

/// Main plugin for HTML UI: converter + builder + reload integration.
pub struct ExtendedUiHtmlPlugin;

impl Plugin for ExtendedUiHtmlPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<HtmlChangeEvent>();

        app.init_resource::<HtmlStructureMap>();
        app.init_resource::<HtmlFunctionRegistry>();
        app.init_resource::<HtmlDirty>();

        app.register_type::<HtmlEventBindings>();
        app.register_type::<HtmlSource>();
        app.register_type::<HtmlStyle>();

        app.add_plugins((
            HtmlConverterSystem,
            HtmlBuilderSystem,
            HtmlReloadPlugin,
        ));
    }
}
