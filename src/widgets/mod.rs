mod container;
pub mod attributes;
mod controls;

use bevy::prelude::*;
use crate::registry::{BODY_ID_POOL, BUTTON_ID_POOL, DIV_ID_POOL};
use crate::widgets::attributes::*;
use crate::styles::style_types::IconPlace;
use crate::widgets::container::body::HtmlBodyWidget;
use crate::widgets::container::div::DivWidget;
use crate::widgets::controls::button::ButtonWidget;

#[derive(Component, Default, Clone, Debug)]
pub struct Widget(pub Option<String>);

#[derive(Component, Clone, Copy, Debug)]
pub struct WidgetId {
    pub id: usize,
    pub kind: WidgetKind,
}

#[derive(Debug, Clone, Copy)]
pub enum WidgetKind {
    HtmlBody,
    Div,
    Button,
}

// ===============================================
//                       Body
// ===============================================

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UIGenID, UIWidgetState, GlobalTransform, InheritedVisibility, Widget)]
pub struct HtmlBody {
    pub w_count: usize,
    pub bind_to_html: Option<String>,
}

impl Default for HtmlBody {
    fn default() -> Self {
        let w_count = BODY_ID_POOL.lock().unwrap().acquire();

        Self {
            w_count,
            bind_to_html: None,
        }
    }
}

// ===============================================
//                       Div
// ===============================================

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UIGenID, UIWidgetState, GlobalTransform, InheritedVisibility, Widget)]
pub struct Div(pub usize);

impl Default for Div {
    fn default() -> Self {
        let w_count = DIV_ID_POOL.lock().unwrap().acquire();
        Self(w_count)
    }
}

// ===============================================
//                     Button
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
        let w_count = BUTTON_ID_POOL.lock().unwrap().acquire();

        Self {
            w_count,
            text: String::from("Button"),
            icon_path: None,
            icon_place: IconPlace::default(),
        }
    }
}

pub struct WidgetPlugin;

impl Plugin for WidgetPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            HtmlBodyWidget,
            DivWidget,
            ButtonWidget,
        ));
    }
}