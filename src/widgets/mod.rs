mod body;
mod div;
mod button;
mod check_box;
mod headline;
mod paragraph;
mod image;

use std::fmt;
use bevy::prelude::*;
use crate::registry::*;
use crate::styles::IconPlace;
use crate::widgets::body::BodyWidget;
use crate::widgets::button::ButtonWidget;
use crate::widgets::check_box::CheckBoxWidget;
use crate::widgets::div::DivWidget;
use crate::widgets::headline::HeadlineWidget;
use crate::widgets::image::ImageWidget;
use crate::widgets::paragraph::ParagraphWidget;

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
            ButtonWidget,
            DivWidget,
            CheckBoxWidget,
            HeadlineWidget,
            ImageWidget,
            ParagraphWidget,
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

// ===============================================
//                     Button
// ===============================================

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UIGenID, UIWidgetState, Widget)]
pub struct Button {
    pub entry: usize,
    pub text: String,
    pub icon_place: IconPlace,
    pub icon_path: Option<String>,
}

impl Default for Button {
    fn default() -> Self {
        let entry = BUTTON_ID_POOL.lock().unwrap().acquire();

        Self {
            entry,
            text: String::from("Button"),
            icon_path: None,
            icon_place: IconPlace::default(),
        }
    }
}

// ===============================================
//                     CheckBox
// ===============================================

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UIGenID, UIWidgetState, Widget)]
pub struct CheckBox {
    pub entry: usize,
    pub label: String,
    pub icon_path: Option<String>,
}

impl Default for CheckBox {
    fn default() -> Self {
        let entry = CHECK_BOX_ID_POOL.lock().unwrap().acquire();

        Self {
            entry,
            label: String::from("label"),
            icon_path: Some(String::from("extended_ui/icons/check-mark.png")),
        }
    }
}

// ===============================================
//                       Headline
// ===============================================

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UIGenID, UIWidgetState, Widget)]
pub struct Headline {
    pub entry: usize,
    pub text: String,
    pub h_type: HeadlineType
}

impl Default for Headline {

    fn default() -> Self {
        let entry = HEADLINE_ID_POOL.lock().unwrap().acquire();
        Self {
            entry,
            text: String::from("Headline"),
            h_type: HeadlineType::H3
        }
    }
}

#[derive(Reflect, Default, Debug, Clone, Eq, PartialEq)]
pub enum HeadlineType {
    #[default]
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
}

impl fmt::Display for HeadlineType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            HeadlineType::H1 => "h1",
            HeadlineType::H2 => "h2",
            HeadlineType::H3 => "h3",
            HeadlineType::H4 => "h4",
            HeadlineType::H5 => "h5",
            HeadlineType::H6 => "h6",
        };
        write!(f, "{}", s)
    }
}

// ===============================================
//                       Image
// ===============================================

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UIGenID, UIWidgetState, GlobalTransform, InheritedVisibility, Widget)]
pub struct Img {
    pub entry: usize,
    pub src: Option<String>,
    pub alt: String,
}

impl Default for Img {
    fn default() -> Self {
        let entry = IMAGE_ID_POOL.lock().unwrap().acquire();

        Self {
            entry,
            src: None,
            alt: String::from(""),
        }
    }
}

// ===============================================
//                     Paragraph
// ===============================================

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UIGenID, UIWidgetState, Widget)]
pub struct Paragraph {
    pub entry: usize,
    pub text: String,
}

impl Default for Paragraph {
    fn default() -> Self {
        let entry = PARAGRAPH_ID_POOL.lock().unwrap().acquire();

        Self {
            entry,
            text: String::from(""),
        }
    }
}