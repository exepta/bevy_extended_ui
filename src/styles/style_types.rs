use bevy::prelude::*;

/// Represents the border-radius of a rectangle with individual corner values.
#[derive(Reflect, Default, Clone, PartialEq, Debug)]
pub struct Radius {
    pub top_left: Val,
    pub top_right: Val,
    pub bottom_left: Val,
    pub bottom_right: Val,
}

impl Radius {

    /**
    * Creates a `Radius` where all corners have the same radius value.
    *
    * @param val The radius value to use for all four corners.
    * @return A new `Radius` with uniform corner radii.
    */
    pub fn all(val: Val) -> Self {
        Self {
            top_left: val,
            top_right: val,
            bottom_left: val,
            bottom_right: val
        }
    }
}

/// Defines the background style including color and optional image.
#[derive(Reflect, Debug, Clone, PartialEq)]
pub struct Background {
    pub color: Color,
    pub image: Option<String>,
}

impl Default for Background {

    /**
    * Creates a default `Background` with transparent color and no image.
    */
    fn default() -> Self {
        Self {
            color: Color::NONE,
            image: None,
        }
    }
}

/// Constants for common font weight values.
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

/// Placement of an icon relative to text.
#[derive(Reflect, Debug, Clone, PartialEq)]
pub enum IconPlace {
    Left,
    Right
}

impl Default for IconPlace {

    /**
    * Returns the default icon placement (`Right`).
    */
    fn default() -> Self {
        IconPlace::Right
    }
}

/// Represents a font size value, either in pixels or rem units.
#[derive(Reflect, Debug, Clone, PartialEq)]
pub enum FontVal {
    Px(f32),
    Rem(f32)
}

impl Default for FontVal {

    /**
    * Returns the default font size of 12 pixels.
    */
    fn default() -> Self {
        FontVal::Px(12.0)
    }
}

impl FontVal {

    /**
    * Computes the absolute font size in pixels, resolving rem units using a base size.
    *
    * @param base Optional base font size in pixels for rem calculations. Defaults to 1.0 if not provided.
    * @return The computed font size in pixels.
    */
    pub fn get(&self, base: Option<f32>) -> f32 {
        match self {
            FontVal::Px(x) => x.clone(),
            FontVal::Rem(x) => x * base.unwrap_or(1.0),
        }
    }
}
