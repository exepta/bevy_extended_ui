use std::cmp::PartialEq;
use bevy::prelude::*;
use crate::styling::paint::Colored;
use crate::styling::convert::{CssClass, CssSource, CssID};
use crate::styling::system::WidgetStyle;

pub mod css;
pub mod paint;
pub mod convert;
pub mod system;

#[derive(Reflect, Default, Clone, PartialEq, Debug)]
pub struct Radius {
    pub top_left: Val,
    pub top_right: Val,
    pub bottom_left: Val,
    pub bottom_right: Val,
}

impl Radius {
    pub fn all(val: Val) -> Self {
        Self {
            top_left: val,
            top_right: val,
            bottom_left: val,
            bottom_right: val
        }
    }
}


#[derive(Reflect, Debug, Clone, PartialEq)]
pub struct Background {
    pub color: Color,
    pub image: Option<String>,
}

impl Default for Background {
    fn default() -> Self {
        Self {
            color: Colored::WHITE,
            image: None,
        }
    }
}


#[derive(Reflect, Debug, Clone)]
pub struct FontWeight;

impl FontWeight {
    pub const THIN: u16 = 100;
    pub const EXTRA_LIGHT: u16 = 200;
    pub const LIGHT: u16 = 300;
    pub const NORMAL: u16 = 400;
    pub const MEDIUM: u16 = 500;
    pub const SEMI_BOLD: u16 = 600;
    pub const BOLD: u16 = 700;
    pub const EXTRA_BOLD: u16 = 800;
    pub const BLACK: u16 = 900;
}

#[derive(Reflect, Debug, Clone, PartialEq)]
pub enum IconPlace {
    Left,
    Right
}

impl Default for IconPlace {
    fn default() -> Self {
        IconPlace::Right
    }
}


#[derive(Reflect, Debug, Clone, PartialEq)]
pub enum FontVal {
    Px(f32),
    Rem(f32)
}

impl Default for FontVal {
    fn default() -> Self {
        FontVal::Px(12.0)
    }
}

impl FontVal {
    pub fn get(&self, base: Option<f32>) -> f32 {
        match self { 
            FontVal::Px(x) => x.clone(),
            FontVal::Rem(x) => x * base.unwrap_or(1.0),
        }
    }
}

#[derive(Reflect, Default, Debug, Clone, PartialEq)]
pub struct Style {
    pub display: Option<Display>,
    pub position_type: Option<PositionType>,
    pub width: Option<Val>,
    pub min_width: Option<Val>,
    pub max_width: Option<Val>,
    pub height: Option<Val>,
    pub min_height: Option<Val>,
    pub max_height: Option<Val>,
    pub left: Option<Val>,
    pub top: Option<Val>,
    pub right: Option<Val>,
    pub bottom: Option<Val>,
    pub padding: Option<UiRect>,
    pub margin: Option<UiRect>,
    pub border: Option<UiRect>,
    pub overflow: Option<Overflow>,
    pub color: Option<Color>,
    pub background: Option<Background>,
    pub border_color: Option<Color>,
    pub border_width: Option<Val>,
    pub border_radius: Option<Radius>,
    pub font_size: Option<FontVal>,
    pub box_shadow: Option<BoxShadow>,
    pub justify_content: Option<JustifyContent>,
    pub justify_items: Option<JustifyItems>,
    pub justify_self: Option<JustifySelf>,
    pub align_content: Option<AlignContent>,
    pub align_items: Option<AlignItems>,
    pub align_self: Option<AlignSelf>,
    pub flex_direction: Option<FlexDirection>,
    pub flex_grow: Option<f32>,
    pub gap: Option<Val>,
    pub text_wrap: Option<LineBreak>,
    pub z_index: Option<i32>,
}

impl Style {
    pub fn merge(&mut self, other: &Style) {
        if other.display.is_some()           { self.display = other.display.clone(); }
        if other.position_type.is_some()     { self.position_type = other.position_type.clone(); }
        if other.width.is_some()             { self.width = other.width.clone(); }
        if other.min_width.is_some()         { self.min_width = other.min_width.clone(); }
        if other.max_width.is_some()         { self.max_width = other.max_width.clone(); }
        if other.height.is_some()            { self.height = other.height.clone(); }
        if other.min_height.is_some()        { self.min_height = other.min_height.clone(); }
        if other.max_height.is_some()        { self.max_height = other.max_height.clone(); }
        if other.left.is_some()              { self.left = other.left.clone(); }
        if other.top.is_some()               { self.top = other.top.clone(); }
        if other.right.is_some()             { self.right = other.right.clone(); }
        if other.bottom.is_some()            { self.bottom = other.bottom.clone(); }
        if other.padding.is_some()           { self.padding = other.padding.clone(); }
        if other.margin.is_some()            { self.margin = other.margin.clone(); }
        if other.border.is_some()            { self.border = other.border.clone(); }
        if other.overflow.is_some()          { self.overflow = other.overflow.clone(); }
        if other.color.is_some()             { self.color = other.color.clone(); }
        if other.background.is_some()        { self.background = other.background.clone(); }
        if other.border_color.is_some()      { self.border_color = other.border_color.clone(); }
        if other.border_width.is_some()      { self.border_width = other.border_width.clone(); }
        if other.border_radius.is_some()     { self.border_radius = other.border_radius.clone(); }
        if other.font_size.is_some()         { self.font_size = other.font_size.clone(); }
        if other.box_shadow.is_some()        { self.box_shadow = other.box_shadow.clone(); }
        if other.justify_content.is_some()   { self.justify_content = other.justify_content.clone(); }
        if other.justify_items.is_some()     { self.justify_items = other.justify_items.clone(); }
        if other.justify_self.is_some()      { self.justify_self = other.justify_self.clone(); }
        if other.align_content.is_some()     { self.align_content = other.align_content.clone(); }
        if other.align_items.is_some()       { self.align_items = other.align_items.clone(); }
        if other.align_self.is_some()        { self.align_self = other.align_self.clone(); }
        if other.flex_direction.is_some()    { self.flex_direction = other.flex_direction.clone(); }
        if other.flex_grow.is_some()         { self.flex_grow = other.flex_grow.clone(); }
        if other.gap.is_some()               { self.gap = other.gap.clone(); }
        if other.text_wrap.is_some()         { self.text_wrap = other.text_wrap.clone(); }
        if other.z_index.is_some()           { self.z_index = other.z_index.clone(); }
    }
}

pub struct StylingPlugin;

impl Plugin for StylingPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<WidgetStyle>();
        app.register_type::<CssSource>();
        app.register_type::<CssClass>();
        app.register_type::<CssID>();
    }
}