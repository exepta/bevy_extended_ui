use bevy::prelude::*;

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct CssBase(pub String);

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct CssHover(pub String);

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct CssDisabled(pub String);

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct CssReadOnly(pub String);

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct CssFocus(pub String);

// ==================================================
//                     Self made
// ==================================================

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct CssFile(pub String);

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct CssClass(pub Vec<String>);

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct CssID(pub String);