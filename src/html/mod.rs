mod converter;
mod builder;

use std::collections::HashMap;
use bevy::prelude::*;
use crate::html::builder::HtmlBuilderSystem;
use crate::html::converter::HtmlConverterSystem;
use crate::styling::css::apply_property_to_style;
use crate::styling::Style;
use crate::widgets::{CheckBox, Div, InputField, Button, HtmlBody, ChoiceBox, Slider, Headline, Paragraph, Img};

/// A component that stores the raw HTML source as a string.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct HtmlSource(pub String);

/// A component that stores parsed CSS style data using Bevy's `Style` struct.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct HtmlStyle(pub Style);

impl HtmlStyle {
    /// Parses a raw CSS style string and converts it into an `HtmlStyle`.
    ///
    /// The input string should be a semicolon-separated list of CSS properties.
    ///
    /// # Example
    /// ```rust
    /// use bevy_extended_ui::html::HtmlStyle;
    /// let style = HtmlStyle::from_str("display: flex; justify-content: center;");
    /// ```
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

/// Metadata attached to HTML elements, such as class, id, inline styles, or embedded CSS.
#[derive(Debug, Clone, Default)]
pub struct HtmlMeta {
    /// Embedded `<style>` or global CSS rules.
    pub css: String,
    /// Value of the `id` attribute.
    pub id: Option<String>,
    /// Value(s) of the `class` attribute.
    pub class: Option<Vec<String>>,
    /// Inline CSS from the `style` attribute.
    pub style: Option<String>,
}

/// An enum representing a node in the HTML DOM hierarchy,
/// mapped to Bevy UI components.
#[derive(Debug, Clone)]
pub enum HtmlWidgetNode {
    /// A `<button>` element.
    Button(Button, HtmlMeta, HtmlEventBindings),
    /// An `<input type="text">` field.
    Input(InputField, HtmlMeta, HtmlEventBindings),
    /// A checkbox `<input type="checkbox">`.
    CheckBox(CheckBox, HtmlMeta, HtmlEventBindings),
    /// A dropdown or select box.
    ChoiceBox(ChoiceBox, HtmlMeta, HtmlEventBindings),
    /// A img element (`<img>`).
    Img(Img, HtmlMeta, HtmlEventBindings),
    /// A heading element (`<h1>`-`<h6>`).
    Headline(Headline, HtmlMeta, HtmlEventBindings),
    /// A paragraph `<p>`.
    Paragraph(Paragraph, HtmlMeta, HtmlEventBindings),
    /// A slider input (range).
    Slider(Slider, HtmlMeta, HtmlEventBindings),
    /// A `<div>` container element with nested child nodes.
    Div(Div, HtmlMeta, Vec<HtmlWidgetNode>, HtmlEventBindings),
    /// The root `<body>` element of the HTML structure.
    HtmlBody(HtmlBody, HtmlMeta, Vec<HtmlWidgetNode>, HtmlEventBindings),
}

/// A resource that holds all parsed HTML structures keyed by identifier.
/// One entry can be marked as currently active.
#[derive(Resource)]
pub struct HtmlStructureMap {
    /// Map of structure names (e.g., file or document names) to their HTML node trees.
    pub html_map: HashMap<String, Vec<HtmlWidgetNode>>,
    /// Currently active structure identifier, if any.
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

type ClickObserverFn = fn(Trigger<Pointer<Click>>, Commands);
type OverObserverFn = fn(Trigger<Pointer<Over>>, Commands);
type OutObserverFn = fn(Trigger<Pointer<Out>>, Commands);

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
}

/// The main plugin that registers all HTML UI systems and resources.
pub struct HtmlPlugin;

impl Plugin for HtmlPlugin {
    /// Configures the app to support HTML parsing and UI construction.
    fn build(&self, app: &mut App) {
        app.init_resource::<HtmlStructureMap>();
        app.init_resource::<HtmlFunctionRegistry>();
        app.register_type::<HtmlEventBindings>();
        app.register_type::<HtmlSource>();
        app.register_type::<HtmlStyle>();
        app.add_plugins((HtmlConverterSystem, HtmlBuilderSystem));
    }
}
