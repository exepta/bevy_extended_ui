mod body;
mod div;

use bevy::prelude::*;
use crate::registry::{BODY_ID_POOL, DIV_ID_POOL, UI_ID_GENERATE};
use crate::widgets::body::BodyWidget;
use crate::widgets::div::DivWidget;

/// Marker component for UI elements that should ignore the parent widget state.
///
/// Used to mark UI nodes that do not inherit state like `focused`, `hovered`, etc.
#[derive(Component)]
pub struct IgnoreParentState;

/// Unique identifier for UI elements.
///
/// Each UI element should have a unique `UIGenID` generated atomically.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct UIGenID(usize);

impl Default for UIGenID {

    /// Generates a new unique `UIGenID` using a global atomic counter.
    fn default() -> Self {
        Self(UI_ID_GENERATE.lock().unwrap().acquire())
    }
}

impl UIGenID {
    pub fn get(&self) -> usize {
        self.0
    }
}

/// Associates a UI child entity with a parent widget by ID.
///
/// Used for binding UI components to their logical parent.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct BindToID(pub usize);

impl BindToID {
    pub fn get(&self) -> usize {
        self.0
    }
}

/// Stores the interaction and UI state flags for a widget.
///
/// Contains boolean flags for common widget states such as focused, hovered, disabled, etc.
#[derive(Component, Reflect, Default, PartialEq, Eq, Debug, Clone)]
#[reflect(Component)]
pub struct UIWidgetState {
    pub focused: bool,
    pub hovered: bool,
    pub disabled: bool,
    pub readonly: bool,
    pub checked: bool,
    pub open: bool,
}

#[derive(Component, Default, Clone, Debug)]
pub struct Widget(pub Option<String>);

#[derive(Component, Clone, Copy, Debug)]
pub struct WidgetId {
    pub id: usize,
    pub kind: WidgetKind,
}

#[derive(Debug, Clone, Copy)]
pub enum WidgetKind {
    Body,
    Div,
    Headline,
    Paragraph,
    Button,
    CheckBox,
    Slider,
    InputField,
    ChoiceBox,
    Img,
    ProgressBar,
}

pub struct ExtendedWidgetPlugin;

impl Plugin for ExtendedWidgetPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<UIGenID>();
        app.register_type::<BindToID>();
        app.register_type::<UIWidgetState>();
        app.register_type::<Body>();
        app.add_plugins((
            BodyWidget,
            DivWidget
        ));
    }
}

// ===============================================
//                       Body
// ===============================================

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UIGenID, UIWidgetState, GlobalTransform, InheritedVisibility, Widget)]
pub struct Body {
    pub entry: usize,
    pub html_key: Option<String>,
}

impl Default for Body {
    fn default() -> Self {
        let entry = BODY_ID_POOL.lock().unwrap().acquire();

        Self {
            entry,
            html_key: None,
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
        let entry = DIV_ID_POOL.lock().unwrap().acquire();
        Self(entry)
    }
}