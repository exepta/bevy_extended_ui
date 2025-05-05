mod button;
mod div;

use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;
use bevy::prelude::*;
use crate::{UIGenID, UIWidgetState};
use crate::widgets::button::ButtonWidget;
use crate::widgets::div::DivWidget;

static BUTTON_COUNT: AtomicUsize = AtomicUsize::new(1);
static DIV_COUNT: AtomicUsize = AtomicUsize::new(1);

// ===============================================
//                       Div
// ===============================================

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UIGenID, UIWidgetState, GlobalTransform, InheritedVisibility)]
pub struct Div(usize);

impl Default for Div {
    fn default() -> Self {
        Self(DIV_COUNT.fetch_add(1, Relaxed))
    }
}

// ===============================================
//                       Button
// ===============================================

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UIGenID, UIWidgetState)]
pub struct Button {
    pub w_count: usize,
    pub text: String,
}

impl Default for Button {
    fn default() -> Self {
        Self {
            w_count: BUTTON_COUNT.fetch_add(1, Relaxed),
            text: String::from("Button")
        }
    }
}

pub struct WidgetPlugin;

impl Plugin for WidgetPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Div>();
        app.register_type::<Button>();
        app.add_plugins((
            DivWidget, 
            ButtonWidget
        ));
    }
}