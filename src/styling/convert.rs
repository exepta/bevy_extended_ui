use bevy::prelude::*;

// ==================================================
//                     Self made
// ==================================================

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct CssSource(pub String);

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct CssClass(pub Vec<String>);

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct CssID(pub String);