pub mod builder;
pub mod converter;
pub mod reload;
mod bindings;

pub use inventory;

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use bevy::ecs::system::SystemId;
use bevy::prelude::*;
use crate::html::bindings::HtmlEventBindingsPlugin;
use crate::html::builder::HtmlBuilderSystem;
use crate::html::converter::HtmlConverterSystem;
use crate::html::reload::HtmlReloadPlugin;
use crate::lang::{UiLangState, UiLangVariables, UILang};

use crate::io::{CssAsset, HtmlAsset};
use crate::styles::Style;
use crate::styles::parser::apply_property_to_style;
use crate::widgets::{Body, Button, CheckBox, ChoiceBox, Div, Divider, FieldSet, Headline, Img, InputField, Paragraph, ProgressBar, RadioButton, Scrollbar, Slider, SwitchButton, ToggleButton, ValidationRules, Widget};

pub static HTML_ID_COUNTER: AtomicUsize = AtomicUsize::new(1);

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum HtmlSystemSet {
    Convert,
    Build,
    ShowWidgets,
    Bindings,
}

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct HtmlSource {
    pub handle: Handle<HtmlAsset>,
    pub source_id: String,
    pub controller: Option<String>,
}

impl HtmlSource {
    pub fn from_handle(handle: Handle<HtmlAsset>) -> Self {
        Self {
            handle,
            source_id: String::new(),
            controller: None,
        }
    }

    /// Returns the asset path (relative to assets/) of this HtmlAsset.
    /// Example: "examples/test.html"
    pub fn get_source_path(&self) -> String {
        self.handle
            .path()
            .expect("Failed to get source path!")
            .path()
            .to_string_lossy()
            .replace('\\', "/")
    }
}

#[derive(Event, Message)]
pub struct HtmlAllWidgetsSpawned;

#[derive(Event, Message)]
pub struct HtmlAllWidgetsVisible;

#[derive(Component, Default)]
pub struct HtmlInitEmitted;

#[derive(Resource, Default)]
pub struct HtmlInitDelay(pub Option<u8>);

#[derive(Component)]
pub struct NeedHidden;

#[derive(Resource, Default)]
pub struct ShowWidgetsTimer {
    pub timer: Timer,
    pub active: bool,
}

#[derive(Event, Message)]
pub struct HtmlChangeEvent;

/// A simple explicit "UI needs rebuild" flag.
/// We use this because mutating the internal HashMap of HtmlStructureMap
/// does NOT reliably trigger `resource_changed::<HtmlStructureMap>()`.
#[derive(Resource, Default)]
pub struct HtmlDirty(pub bool);

/// Component storing parsed inline CSS (`style="..."`) as your custom Style struct.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct HtmlStyle(pub Style);

impl HtmlStyle {
    /// Parses inline CSS style declarations ("key: value; ...") into Style.
    pub fn from_str(style_code: &str) -> HtmlStyle {
        let mut style = Style::default();

        for part in style_code.split(';') {
            let trimmed = part.trim();
            if trimmed.is_empty() {
                continue;
            }

            let (name, value) = if let Some((k, v)) = trimmed.split_once(':') {
                (k.trim(), v.trim())
            } else if let Some((k, v)) = trimmed.split_once(' ') {
                (k.trim(), v.trim())
            } else {
                continue;
            };

            apply_property_to_style(&mut style, name, value);
        }

        HtmlStyle(style)
    }
}

#[derive(Debug, Clone, Default)]
pub struct HtmlMeta {
    /// All referenced CSS assets for this node.
    pub css: Vec<Handle<CssAsset>>,
    pub id: Option<String>,
    pub class: Option<Vec<String>>,
    pub style: Option<HtmlStyle>,
    pub validation: Option<ValidationRules>,
}

#[derive(Debug, Clone, Default)]
pub struct HtmlStates {
    pub hidden: bool,
    pub disabled: bool,
    pub readonly: bool,
}

/// Your current DOM model.
#[derive(Debug, Clone)]
pub enum HtmlWidgetNode {
    /// The root `<body>` element of the HTML structure.
    Body(
        Body,
        HtmlMeta,
        HtmlStates,
        Vec<HtmlWidgetNode>,
        HtmlEventBindings,
        Widget,
        HtmlID,
    ),
    /// A `<div>` container element with nested child nodes.
    Div(
        Div,
        HtmlMeta,
        HtmlStates,
        Vec<HtmlWidgetNode>,
        HtmlEventBindings,
        Widget,
        HtmlID,
    ),
    /// A `<divider>` element.
    Divider(
        Divider,
        HtmlMeta,
        HtmlStates,
        HtmlEventBindings,
        Widget,
        HtmlID,
    ),
    /// A `<button>` element.
    Button(
        Button,
        HtmlMeta,
        HtmlStates,
        HtmlEventBindings,
        Widget,
        HtmlID,
    ),
    /// A checkbox `<checkbox>`.
    CheckBox(
        CheckBox,
        HtmlMeta,
        HtmlStates,
        HtmlEventBindings,
        Widget,
        HtmlID,
    ),
    /// A dropdown or select box.
    ChoiceBox(
        ChoiceBox,
        HtmlMeta,
        HtmlStates,
        HtmlEventBindings,
        Widget,
        HtmlID,
    ),
    /// A `<fieldset>` container element with nested child nodes from type `<radio> and <toggle>`.
    FieldSet(
        FieldSet,
        HtmlMeta,
        HtmlStates,
        Vec<HtmlWidgetNode>,
        HtmlEventBindings,
        Widget,
        HtmlID,
    ),
    /// A heading element (`<h1>`-`<h6>`).
    Headline(
        Headline,
        HtmlMeta,
        HtmlStates,
        HtmlEventBindings,
        Widget,
        HtmlID,
    ),
    /// A img element (`<img>`).
    Img(Img, HtmlMeta, HtmlStates, HtmlEventBindings, Widget, HtmlID),
    /// An `<input type="text">` field.
    Input(
        InputField,
        HtmlMeta,
        HtmlStates,
        HtmlEventBindings,
        Widget,
        HtmlID,
    ),
    /// A paragraph `<p>`.
    Paragraph(
        Paragraph,
        HtmlMeta,
        HtmlStates,
        HtmlEventBindings,
        Widget,
        HtmlID,
    ),
    /// A progressbar `<progressbar>`.
    ProgressBar(
        ProgressBar,
        HtmlMeta,
        HtmlStates,
        HtmlEventBindings,
        Widget,
        HtmlID,
    ),
    /// A radio-button `<radio>`.
    RadioButton(
        RadioButton,
        HtmlMeta,
        HtmlStates,
        HtmlEventBindings,
        Widget,
        HtmlID,
    ),
    /// A slider input `<slider>`).
    Scrollbar(
        Scrollbar,
        HtmlMeta,
        HtmlStates,
        HtmlEventBindings,
        Widget,
        HtmlID,
    ),
    /// A slider input `<slider>`).
    Slider(
        Slider,
        HtmlMeta,
        HtmlStates,
        HtmlEventBindings,
        Widget,
        HtmlID,
    ),
    /// A switch-button `<switch>`).
    SwitchButton(
        SwitchButton,
        HtmlMeta,
        HtmlStates,
        HtmlEventBindings,
        Widget,
        HtmlID,
    ),
    /// A toggle-button `<toggle>`.
    ToggleButton(
        ToggleButton,
        HtmlMeta,
        HtmlStates,
        HtmlEventBindings,
        Widget,
        HtmlID,
    ),
}

/// Stores all parsed HTML structures keyed by `<meta name="...">`.
#[derive(Resource)]
pub struct HtmlStructureMap {
    pub html_map: HashMap<String, Vec<HtmlWidgetNode>>,
    pub active: Option<Vec<String>>,
}

impl Default for HtmlStructureMap {
    fn default() -> Self {
        Self {
            html_map: HashMap::new(),
            active: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Component)]
pub struct HtmlID(pub usize);

impl Default for HtmlID {
    fn default() -> Self {
        Self(HTML_ID_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

pub enum HtmlFnRegistration {
    HtmlEvent {
        name: &'static str,
        build: fn(&mut World) -> SystemId<In<HtmlEvent>, ()>,
    },
    HtmlClick {
        name: &'static str,
        build: fn(&mut World) -> SystemId<In<HtmlClick>, ()>,
    },
    HtmlChange {
        name: &'static str,
        build: fn(&mut World) -> SystemId<In<HtmlChange>, ()>,
    },
    HtmlInit {
        name: &'static str,
        build: fn(&mut World) -> SystemId<In<HtmlInit>, ()>,
    },
    HtmlMouseOut {
        name: &'static str,
        build: fn(&mut World) -> SystemId<In<HtmlMouseOut>, ()>,
    },
    HtmlMouseOver {
        name: &'static str,
        build: fn(&mut World) -> SystemId<In<HtmlMouseOver>, ()>,
    },
    HtmlFocus {
        name: &'static str,
        build: fn(&mut World) -> SystemId<In<HtmlFocus>, ()>,
    },
    HtmlScroll {
        name: &'static str,
        build: fn(&mut World) -> SystemId<In<HtmlScroll>, ()>,
    },
    HtmlKeyDown {
        name: &'static str,
        build: fn(&mut World) -> SystemId<In<HtmlKeyDown>, ()>,
    },
    HtmlKeyUp {
        name: &'static str,
        build: fn(&mut World) -> SystemId<In<HtmlKeyUp>, ()>,
    },
    HtmlDragStart {
        name: &'static str,
        build: fn(&mut World) -> SystemId<In<HtmlDragStart>, ()>,
    },
    HtmlDrag {
        name: &'static str,
        build: fn(&mut World) -> SystemId<In<HtmlDrag>, ()>,
    },
    HtmlDragStop {
        name: &'static str,
        build: fn(&mut World) -> SystemId<In<HtmlDragStop>, ()>,
    },
}

inventory::collect!(HtmlFnRegistration);

#[derive(Clone, Copy)]
pub struct HtmlEvent {
    pub entity: Entity,
}

impl HtmlEvent {
    pub fn target(&self) -> Entity { self.entity }

}

#[derive(Default, Resource)]
pub struct HtmlFunctionRegistry {
    pub click: HashMap<String, SystemId<In<HtmlEvent>>>,
    pub over: HashMap<String, SystemId<In<HtmlEvent>>>,
    pub out: HashMap<String, SystemId<In<HtmlEvent>>>,
    pub change: HashMap<String, SystemId<In<HtmlEvent>>>,
    pub init: HashMap<String, SystemId<In<HtmlEvent>>>,
    pub focus: HashMap<String, SystemId<In<HtmlEvent>>>,
    pub scroll: HashMap<String, SystemId<In<HtmlEvent>>>,
    pub keydown: HashMap<String, SystemId<In<HtmlEvent>>>,
    pub keyup: HashMap<String, SystemId<In<HtmlEvent>>>,
    pub dragstart: HashMap<String, SystemId<In<HtmlEvent>>>,
    pub drag: HashMap<String, SystemId<In<HtmlEvent>>>,
    pub dragstop: HashMap<String, SystemId<In<HtmlEvent>>>,
    pub click_typed: HashMap<String, SystemId<In<HtmlClick>>>,
    pub over_typed: HashMap<String, SystemId<In<HtmlMouseOver>>>,
    pub out_typed: HashMap<String, SystemId<In<HtmlMouseOut>>>,
    pub change_typed: HashMap<String, SystemId<In<HtmlChange>>>,
    pub init_typed: HashMap<String, SystemId<In<HtmlInit>>>,
    pub focus_typed: HashMap<String, SystemId<In<HtmlFocus>>>,
    pub scroll_typed: HashMap<String, SystemId<In<HtmlScroll>>>,
    pub keydown_typed: HashMap<String, SystemId<In<HtmlKeyDown>>>,
    pub keyup_typed: HashMap<String, SystemId<In<HtmlKeyUp>>>,
    pub dragstart_typed: HashMap<String, SystemId<In<HtmlDragStart>>>,
    pub drag_typed: HashMap<String, SystemId<In<HtmlDrag>>>,
    pub dragstop_typed: HashMap<String, SystemId<In<HtmlDragStop>>>,
}

#[derive(Component, Reflect, Default, Clone, Debug)]
#[reflect(Component)]
pub struct HtmlEventBindings {
    pub onclick: Option<String>,
    pub onmouseover: Option<String>,
    pub onmouseout: Option<String>,
    pub onchange: Option<String>,
    pub oninit: Option<String>,
    pub onfoucs: Option<String>,
    pub onscroll: Option<String>,
    pub onkeydown: Option<String>,
    pub onkeyup: Option<String>,
    pub ondragstart: Option<String>,
    pub ondrag: Option<String>,
    pub ondragstop: Option<String>,
}

#[derive(EntityEvent, Clone, Copy)]
pub struct HtmlClick {
    #[event_target]
    pub entity: Entity,
    pub position: Vec2,
    pub inner_position: Vec2,
}

#[derive(EntityEvent, Clone, Copy)]
pub struct HtmlMouseOver {
    #[event_target]
    pub entity: Entity,
}

#[derive(EntityEvent, Clone, Copy)]
pub struct HtmlMouseOut {
    #[event_target]
    pub entity: Entity,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HtmlChangeAction {
    State,
    Style,
    Unknown,
}

#[derive(EntityEvent, Clone, Copy)]
pub struct HtmlChange {
    #[event_target]
    pub entity: Entity,
    pub action: HtmlChangeAction,
}

#[derive(EntityEvent, Clone, Copy)]
pub struct HtmlInit {
    #[event_target]
    pub entity: Entity,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HtmlFocusState {
    Gained,
    Lost,
}

#[derive(EntityEvent, Clone, Copy)]
pub struct HtmlFocus {
    #[event_target]
    pub entity: Entity,
    pub state: HtmlFocusState,
}

#[derive(EntityEvent, Clone, Copy)]
pub struct HtmlScroll {
    #[event_target]
    pub entity: Entity,
    pub delta: Vec2,
    pub x: f32,
    pub y: f32,
}

#[derive(EntityEvent, Clone, Copy)]
pub struct HtmlKeyDown {
    #[event_target]
    pub entity: Entity,
    pub key: KeyCode,
}

#[derive(EntityEvent, Clone, Copy)]
pub struct HtmlKeyUp {
    #[event_target]
    pub entity: Entity,
    pub key: KeyCode,
}

#[derive(EntityEvent, Clone, Copy)]
pub struct HtmlDragStart {
    #[event_target]
    pub entity: Entity,
    pub position: Vec2,
}

#[derive(EntityEvent, Clone, Copy)]
pub struct HtmlDrag {
    #[event_target]
    pub entity: Entity,
    pub position: Vec2,
}

#[derive(EntityEvent, Clone, Copy)]
pub struct HtmlDragStop {
    #[event_target]
    pub entity: Entity,
    pub position: Vec2,
}

/// Main plugin for HTML UI: converter + builder + reload integration.
pub struct ExtendedUiHtmlPlugin;

impl Plugin for ExtendedUiHtmlPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<HtmlChangeEvent>();

        app.init_resource::<HtmlStructureMap>();
        app.init_resource::<HtmlFunctionRegistry>();
        app.init_resource::<HtmlDirty>();
        app.init_resource::<HtmlInitDelay>();
        app.init_resource::<UILang>();
        app.init_resource::<UiLangState>();
        app.init_resource::<UiLangVariables>();

        app.register_type::<HtmlEventBindings>();
        app.register_type::<HtmlSource>();
        app.register_type::<HtmlStyle>();

        app.configure_sets(
            Update,
            (
                HtmlSystemSet::Convert,
                HtmlSystemSet::Build,
                HtmlSystemSet::ShowWidgets,
                HtmlSystemSet::Bindings,
            )
                .chain(),
        );
        app.add_plugins((
            HtmlConverterSystem,
            HtmlBuilderSystem,
            HtmlReloadPlugin,
            HtmlEventBindingsPlugin,
        ));

        app.add_systems(Startup, register_html_fns);
    }
}

pub fn register_html_fns(world: &mut World) {
    let mut to_insert: Vec<(String, SystemId<In<HtmlEvent>>)> = Vec::new();

    for item in inventory::iter::<HtmlFnRegistration> {
        match item {
            HtmlFnRegistration::HtmlEvent { name, build } => {
                let id = (*build)(world);
                to_insert.push(((*name).to_string(), id));
            }
            HtmlFnRegistration::HtmlClick { name, build } => {
                let id = (*build)(world);
                world.resource_mut::<HtmlFunctionRegistry>()
                    .click_typed
                    .insert((*name).to_string(), id);
            }
            HtmlFnRegistration::HtmlChange { name, build } => {
                let id = (*build)(world);
                world.resource_mut::<HtmlFunctionRegistry>()
                    .change_typed
                    .insert((*name).to_string(), id);
            }
            HtmlFnRegistration::HtmlInit { name, build } => {
                let id = (*build)(world);
                world.resource_mut::<HtmlFunctionRegistry>()
                    .init_typed
                    .insert((*name).to_string(), id);
            }
            HtmlFnRegistration::HtmlMouseOut { name, build } => {
                let id = (*build)(world);
                world.resource_mut::<HtmlFunctionRegistry>()
                    .out_typed
                    .insert((*name).to_string(), id);
            }
            HtmlFnRegistration::HtmlMouseOver { name, build } => {
                let id = (*build)(world);
                world.resource_mut::<HtmlFunctionRegistry>()
                    .over_typed
                    .insert((*name).to_string(), id);
            }
            HtmlFnRegistration::HtmlFocus { name, build } => {
                let id = (*build)(world);
                world.resource_mut::<HtmlFunctionRegistry>()
                    .focus_typed
                    .insert((*name).to_string(), id);
            }
            HtmlFnRegistration::HtmlScroll { name, build } => {
                let id = (*build)(world);
                world.resource_mut::<HtmlFunctionRegistry>()
                    .scroll_typed
                    .insert((*name).to_string(), id);
            }
            HtmlFnRegistration::HtmlKeyDown { name, build } => {
                let id = (*build)(world);
                world.resource_mut::<HtmlFunctionRegistry>()
                    .keydown_typed
                    .insert((*name).to_string(), id);
            }
            HtmlFnRegistration::HtmlKeyUp { name, build } => {
                let id = (*build)(world);
                world.resource_mut::<HtmlFunctionRegistry>()
                    .keyup_typed
                    .insert((*name).to_string(), id);
            }
            HtmlFnRegistration::HtmlDragStart { name, build } => {
                let id = (*build)(world);
                world.resource_mut::<HtmlFunctionRegistry>()
                    .dragstart_typed
                    .insert((*name).to_string(), id);
            }
            HtmlFnRegistration::HtmlDrag { name, build } => {
                let id = (*build)(world);
                world.resource_mut::<HtmlFunctionRegistry>()
                    .drag_typed
                    .insert((*name).to_string(), id);
            }
            HtmlFnRegistration::HtmlDragStop { name, build } => {
                let id = (*build)(world);
                world.resource_mut::<HtmlFunctionRegistry>()
                    .dragstop_typed
                    .insert((*name).to_string(), id);
            }
        }
    }

    let mut reg = world.resource_mut::<HtmlFunctionRegistry>();
    for (name, id) in to_insert {
        reg.change.insert(name.clone(), id);
        reg.click.insert(name.clone(), id);
        reg.focus.insert(name.clone(), id);
        reg.init.insert(name.clone(), id);
        reg.scroll.insert(name.clone(), id);
        reg.keydown.insert(name.clone(), id);
        reg.keyup.insert(name.clone(), id);
        reg.dragstart.insert(name.clone(), id);
        reg.drag.insert(name.clone(), id);
        reg.dragstop.insert(name.clone(), id);
        reg.out.insert(name.clone(), id);
        reg.over.insert(name.clone(), id);
        debug!("Registered html fn '{name}' with id {id:?}");
    }
}
