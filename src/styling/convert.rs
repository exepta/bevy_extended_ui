use std::collections::HashSet;
use bevy::prelude::*;

// ==================================================
//                     Css Styling
// ==================================================

#[derive(Resource, Default)]
pub struct ExistingCssIDs(pub HashSet<String>);

#[derive(Component, Reflect, Debug, Clone, Deref, DerefMut)]
#[reflect(Component)]
pub struct TagName(pub String);

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct CssSource(pub String);

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct CssClass(pub Vec<String>);

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct CssID(pub String);