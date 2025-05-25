use bevy::prelude::*;
use crate::styling::css::apply_property_to_style;
use crate::styling::Style;

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

pub struct HtmlSystem;

impl Plugin for HtmlSystem {
    fn build(&self, app: &mut App) {
        app.register_type::<HtmlStyle>();
    }
}