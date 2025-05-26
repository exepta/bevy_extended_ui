mod converter;

use std::collections::HashMap;
use bevy::prelude::*;
use crate::html::converter::HtmlConverterSystem;
use crate::styling::css::apply_property_to_style;
use crate::styling::Style;
use crate::widgets::{CheckBox, Div, InputField, Button};

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct Html(pub String);

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct HtmlStyle(pub Style);

impl HtmlStyle {

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
    pub css: String,
    pub id: Option<String>,
    pub class: Option<Vec<String>>,
    pub style: Option<String>,
}

#[derive(Debug, Clone)]
pub enum HtmlWidgetNode {
    Button(Button, HtmlMeta),
    Input(InputField, HtmlMeta),
    CheckBox(CheckBox, HtmlMeta),
    Div(Div, HtmlMeta, Vec<HtmlWidgetNode>),
}

#[derive(Resource, Default)]
pub struct HtmlStructureMap(pub HashMap<String, Vec<HtmlWidgetNode>>);

pub struct HtmlPlugin;

impl Plugin for HtmlPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HtmlStructureMap>();
        app.register_type::<Html>();
        app.register_type::<HtmlStyle>();
        app.add_plugins(HtmlConverterSystem);
    }
}