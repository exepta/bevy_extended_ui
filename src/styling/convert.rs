use std::collections::HashSet;
use bevy::prelude::*;

// ==================================================
//                     Css Styling
// ==================================================

/// Resource that tracks existing CSS IDs to ensure uniqueness.
#[derive(Resource, Default)]
pub struct ExistingCssIDs(pub HashSet<String>);

/// Component representing the tag name of an element (e.g., "div", "span").
#[derive(Component, Reflect, Debug, Clone, Deref, DerefMut)]
#[reflect(Component)]
pub struct TagName(pub String);

/// Component containing the raw CSS source code associated with an entity.
#[derive(Component, Reflect, Debug, Clone, Default)]
#[reflect(Component)]
pub struct CssSource(pub String);

/// Component representing one or more CSS classes applied to an element.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct CssClass(pub Vec<String>);

/// Component representing the CSS ID of an element.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct CssID(pub String);