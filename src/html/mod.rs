mod converter;
mod builder;
mod reload;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::Receiver;
use bevy::prelude::*;
use notify::{ RecommendedWatcher, Event, Error };
use crate::html::builder::HtmlBuilderSystem;
use crate::html::converter::HtmlConverterSystem;
use crate::html::reload::HtmlReloadSystem;
use crate::observer::time_tick_trigger::TimeTick;
use crate::observer::widget_init_trigger::WidgetInit;
use crate::styling::css::apply_property_to_style;
use crate::styling::Style;
use crate::widgets::{CheckBox, Div, InputField, Button, HtmlBody, ChoiceBox, Slider, Headline, Paragraph, Img, ProgressBar, Widget};

static HTML_ID_COUNTER: AtomicUsize = AtomicUsize::new(1);

/// Represents a chunk of HTML source code along with its unique identifier.
///
/// This component stores the raw HTML source string and an ID string
/// that uniquely identifies the source within the UI registry.
///
/// # Fields
///
/// * `source` - The raw HTML source path as a `String`.
/// * `source_id` - A unique identifier for this HTML source, typically the name under which
///   it is registered.
///
/// # Derives
///
/// This struct derives:
/// - `Component` to be used as a Bevy ECS component.
/// - `Reflect` to enable reflection, useful for editor integration and serialization.
/// - `Debug` for formatting and debugging.
/// - `Clone` for duplicating instances.
/// - `Default` to provide a default empty instance.
///
/// # Example
///
/// ```rust
/// use bevy_extended_ui::html::HtmlSource;
/// let html = HtmlSource {
///     source: "path/to/html".to_string(),
///     source_id: "main_ui".to_string(),
///     controller: None,
///     was_updated: false,
/// };
/// ```
#[derive(Component, Reflect, Debug, Clone, Default)]
#[reflect(Component)]
pub struct HtmlSource {
    /// The raw HTML source code.
    pub source: String,
    /// Unique identifier for the HTML source.
    pub source_id: String,
    /// Controls the function support location
    pub controller: Option<String>,
    
    pub was_updated: bool,
}

impl HtmlSource {

    /// Creates a new `HtmlSource` from a file path.
    ///
    /// This constructor initializes the `source` field with the given path string
    /// and uses the default values for all other fields.
    ///
    /// # Arguments
    ///
    /// * `path` - A string slice representing the file path to the HTML source.
    ///
    /// # Returns
    ///
    /// A new instance of `HtmlSource` with the specified path.
    ///
    /// # Example
    ///
    /// ```
    /// use bevy_extended_ui::html::HtmlSource;
    /// let html_source = HtmlSource::from_file_path("assets/ui/main.html");
    /// ```
    pub fn from_file_path(path: &str) -> HtmlSource {
        HtmlSource {
            source: path.to_string(),
            ..default()
        }
    }
    
}

#[derive(Event)]
pub struct AllWidgetsSpawned;

#[derive(Component)]
pub struct NeedHidden;

#[derive(Resource, Default)]
pub struct ShowWidgetsTimer {
    pub timer: Timer,
    pub active: bool,
}

#[derive(Resource)]
pub struct HtmlWatcher {
    pub watcher: RecommendedWatcher,
    rx: Arc<Mutex<Receiver<std::result::Result<Event, Error>>>>,
}

#[derive(Event)]
pub struct HtmlChangeEvent;

/// A component that stores parsed CSS style data using Bevy's `Style` struct.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct HtmlStyle(pub Style);

impl HtmlStyle {
    /// Parses a raw CSS style string and converts it into an `HtmlStyle`.
    ///
    /// The input string should be a semicolon-separated list of CSS properties.
    ///
    /// # Example
    /// ```rust
    /// use bevy_extended_ui::html::HtmlStyle;
    /// let style = HtmlStyle::from_str("display: flex; justify-content: center;");
    /// ```
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

/// Metadata attached to HTML elements, such as class, id, inline styles, or embedded CSS.
#[derive(Debug, Clone, Default)]
pub struct HtmlMeta {
    /// Embedded `<style>` or global CSS rules.
    pub css: Vec<String>,
    /// Value of the `id` attribute.
    pub id: Option<String>,
    /// Value(s) of the `class` attribute.
    pub class: Option<Vec<String>>,
    /// Inline CSS from the `style` attribute.
    pub style: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct HtmlStates {
    pub hidden: bool,
    pub disabled: bool,
    pub readonly: bool
}

/// An enum representing a node in the HTML DOM hierarchy,
/// mapped to Bevy UI components.
#[derive(Debug, Clone)]
pub enum HtmlWidgetNode {
    /// A `<button>` element.
    Button(Button, HtmlMeta, HtmlStates, HtmlEventBindings, Widget, HtmlID),
    /// An `<input type="text">` field.
    Input(InputField, HtmlMeta, HtmlStates, HtmlEventBindings, Widget, HtmlID),
    /// A checkbox `<input type="checkbox">`.
    CheckBox(CheckBox, HtmlMeta, HtmlStates, HtmlEventBindings, Widget, HtmlID),
    /// A dropdown or select box.
    ChoiceBox(ChoiceBox, HtmlMeta, HtmlStates, HtmlEventBindings, Widget, HtmlID),
    /// A img element (`<img>`).
    Img(Img, HtmlMeta, HtmlStates, HtmlEventBindings, Widget, HtmlID),
    /// A img element (`<img>`).
    ProgressBar(ProgressBar, HtmlMeta, HtmlStates, HtmlEventBindings, Widget, HtmlID),
    /// A heading element (`<h1>`-`<h6>`).
    Headline(Headline, HtmlMeta, HtmlStates, HtmlEventBindings, Widget, HtmlID),
    /// A paragraph `<p>`.
    Paragraph(Paragraph, HtmlMeta, HtmlStates, HtmlEventBindings, Widget, HtmlID),
    /// A slider input (range).
    Slider(Slider, HtmlMeta, HtmlStates, HtmlEventBindings, Widget, HtmlID),
    /// A `<div>` container element with nested child nodes.
    Div(Div, HtmlMeta, HtmlStates, Vec<HtmlWidgetNode>, HtmlEventBindings, Widget, HtmlID),
    /// The root `<body>` element of the HTML structure.
    HtmlBody(HtmlBody, HtmlMeta, HtmlStates, Vec<HtmlWidgetNode>, HtmlEventBindings, Widget, HtmlID),
}

/// A resource that holds all parsed HTML structures keyed by identifier.
/// One entry can be marked as currently active.
#[derive(Resource)]
pub struct HtmlStructureMap {
    /// Map of structure names (e.g., file or document names) to their HTML node trees.
    pub html_map: HashMap<String, Vec<HtmlWidgetNode>>,
    /// Currently active structure identifier, if any.
    pub active: Option<String>,
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

/// Function pointer type for click event observers.
///
/// These functions are called when a `Trigger` event for a pointer click occurs,
/// receiving the event trigger and a `Commands` object to issue commands.
type ClickObserverFn = fn(Trigger<Pointer<Click>>, Commands);

/// Function pointer type for mouse over event observers.
///
/// These functions are called when a `Trigger` event for a pointer over occurs,
/// receiving the event trigger and a `Commands` object.
type OverObserverFn = fn(Trigger<Pointer<Over>>, Commands);

/// Function pointer type for mouse out event observers.
///
/// These functions are called when a `Trigger` event for a pointer out occurs,
/// receiving the event trigger and a `Commands` object.
type OutObserverFn = fn(Trigger<Pointer<Out>>, Commands);

/// Function pointer type for update event observers.
///
/// These functions are invoked whenever a `TimeTick` event is triggered,
/// which typically occurs on every system update tick.
///
/// They receive the event trigger and a `Commands` object for issuing commands.
/// Due to the frequency of these events, observers should be designed for efficient execution.
type UpdateObserverFn = fn(Trigger<TimeTick>, Commands);

/// Type alias for a load observer function used to handle [`WidgetInit`] events.
///
/// This function type defines a callback invoked when a widget initialization event is triggered,
/// allowing custom logic to run during widget setup.
///
/// # Parameters
/// - `Trigger<WidgetInit>`: The trigger object carrying the [`WidgetInit`] event data.
/// - `Commands`: The [`Commands`] used to issue additional actions or spawn entities.
///
/// # See also
/// [`WidgetInit`], [`Commands`]
type LoadObserverFn = fn(Trigger<WidgetInit>, Commands);

/// Registry resource that maps event handler names to their observer functions.
///
/// Holds hash maps for click, mouse over, mouse out, and update events.
/// Used to look up the observer system functions by name for attaching to entities.
#[derive(Default, Resource)]
pub struct HtmlFunctionRegistry {
    /// Map of function names to click event observer functions.
    pub click: HashMap<String, ClickObserverFn>,

    /// Map of function names to mouse over event observer functions.
    pub over: HashMap<String, OverObserverFn>,

    /// Map of function names to mouse out event observer functions.
    pub out: HashMap<String, OutObserverFn>,

    /// Map of function names to update event observer functions.
    pub update: HashMap<String, UpdateObserverFn>,

    pub load: HashMap<String, LoadObserverFn>,
}

/// Component representing HTML event bindings on an entity.
///
/// Each optional field corresponds to the name of a registered observer function
/// that will be called on the respective event.
///
/// Reflect is derived for use with Bevy reflection and editing tools.
#[derive(Component, Reflect, Default, Clone, Debug)]
#[reflect(Component)]
pub struct HtmlEventBindings {
    /// Optional function name to call on a click event.
    pub onclick: Option<String>,

    /// Optional function name to call on mouse enter event.
    pub onmouseenter: Option<String>,

    /// Optional function name to call on mouse leave event.
    pub onmouseleave: Option<String>,

    /// Optional function name to call on update event.
    pub onupdate: Option<String>,

    pub onload: Option<String>,
}

/// The main plugin that registers all HTML UI systems and resources.
pub struct HtmlPlugin;

impl Plugin for HtmlPlugin {
    /// Configures the app to support HTML parsing and UI construction.
    fn build(&self, app: &mut App) {
        app.add_event::<HtmlChangeEvent>();
        app.init_resource::<HtmlStructureMap>();
        app.init_resource::<HtmlFunctionRegistry>();
        app.register_type::<HtmlEventBindings>();
        app.register_type::<HtmlSource>();
        app.register_type::<HtmlStyle>();
        app.add_plugins((HtmlConverterSystem, HtmlBuilderSystem, HtmlReloadSystem));
    }
}
