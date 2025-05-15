mod button;
mod div;
mod check_box;

use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;
use bevy::prelude::*;
use crate::{UIGenID, UIWidgetState};
use crate::styling::IconPlace;
use crate::widgets::button::ButtonWidget;
use crate::widgets::check_box::CheckBoxWidget;
use crate::widgets::div::DivWidget;

static BUTTON_COUNT: AtomicUsize = AtomicUsize::new(1);
static CHECK_BOX_COUNT: AtomicUsize = AtomicUsize::new(1);
static DIV_COUNT: AtomicUsize = AtomicUsize::new(1);

#[derive(Component, Default)]
pub struct Widget;

// ===============================================
//                       Div
// ===============================================

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UIGenID, UIWidgetState, GlobalTransform, InheritedVisibility, Widget)]
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
#[require(UIGenID, UIWidgetState, Widget)]
pub struct Button {
    pub w_count: usize,
    pub text: String,
    pub icon_place: IconPlace,
    pub icon_path: Option<String>,
}

impl Default for Button {
    fn default() -> Self {
        Self {
            w_count: BUTTON_COUNT.fetch_add(1, Relaxed),
            text: String::from("Button"),
            icon_path: None,
            icon_place: IconPlace::default(),
        }
    }
}

// ===============================================
//                       CheckBox
// ===============================================

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UIGenID, UIWidgetState, Widget)]
pub struct CheckBox {
    pub w_count: usize,
    pub label: String,
    pub icon_path: Option<String>,
}

impl Default for CheckBox {
    fn default() -> Self {
        Self {
            w_count: CHECK_BOX_COUNT.fetch_add(1, Relaxed),
            label: String::from("label"),
            icon_path: Some(String::from("icons/check-mark.png")),
        }
    }
}

pub struct WidgetPlugin;

impl Plugin for WidgetPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Div>();
        app.register_type::<Button>();
        app.register_type::<CheckBox>();
        app.add_plugins((
            DivWidget, 
            ButtonWidget,
            CheckBoxWidget,
        ));
    }
}