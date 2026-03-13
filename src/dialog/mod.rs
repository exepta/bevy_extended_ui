use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
#[cfg(all(target_os = "linux", not(target_arch = "wasm32")))]
use once_cell::sync::Lazy;
#[cfg(all(target_os = "linux", not(target_arch = "wasm32")))]
use std::sync::atomic::{AtomicBool, Ordering};

use crate::ExtendedUiConfiguration;
use crate::html::HtmlStyle;
use crate::styles::{BackdropFilter, Background};
use crate::styles::{CssClass, CssID, TagName};
use crate::widgets::{Body, UIWidgetState};

#[cfg(all(target_os = "linux", not(target_arch = "wasm32")))]
static SYSTEM_DIALOG_IN_FLIGHT: Lazy<AtomicBool> = Lazy::new(|| AtomicBool::new(false));

const DIALOG_WIDGET_Z_INDEX: i32 = 120_000;
const DIALOG_RUNTIME_BASE_Z_INDEX: i32 = 130_000;

/// Selects how a dialog is presented.
#[derive(Debug, Clone, Copy, Reflect, Default, Eq, PartialEq)]
pub enum DialogProvider {
    /// In-window modal rendered inside the Bevy UI tree.
    #[default]
    BevyApp,
    /// Native operating system dialog window.
    System,
}

impl DialogProvider {
    /// Parses `renderer` values from HTML attributes.
    pub fn from_attr(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "bevy-app" | "bevyapp" | "bevy" => Some(Self::BevyApp),
            "system" => Some(Self::System),
            _ => None,
        }
    }
}

/// Placement mode for Bevy in-window dialogs.
#[derive(Debug, Clone, Copy, Reflect, Default, Eq, PartialEq)]
pub enum DialogLayout {
    /// Centered floating panel modal.
    #[default]
    FloatingPanel,
    /// Bottom-anchored sheet modal.
    BottomSheet,
}

/// Dialog category for HTML-defined dialogs (`<dialog type="...">`).
#[derive(Debug, Clone, Copy, Reflect, Default, Eq, PartialEq)]
pub enum DialogWidgetType {
    Warn,
    Error,
    #[default]
    Info,
    Blank,
}

impl DialogWidgetType {
    /// Parses `type` values from HTML attributes.
    pub fn from_attr(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "warn" | "warning" => Some(Self::Warn),
            "error" | "failure" => Some(Self::Error),
            "info" | "default" => Some(Self::Info),
            "blank" => Some(Self::Blank),
            _ => None,
        }
    }
}

/// HTML widget backing `<dialog ...>` elements.
///
/// The widget can be styled with CSS like other HTML widgets and opened via a trigger id.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(
    crate::widgets::UIGenID,
    crate::widgets::UIWidgetState,
    crate::widgets::Widget,
    GlobalTransform,
    InheritedVisibility
)]
pub struct DialogWidget {
    /// Trigger element id (`trigger` or `triggger` attribute).
    pub trigger: Option<String>,
    /// Target renderer: in-window (`BevyApp`) or native (`System`).
    pub renderer: DialogProvider,
    /// Dialog type (`warn`, `error`, `info`, `blank`).
    pub dialog_type: DialogWidgetType,
    /// Plain text extracted from node content, mainly used for `System` renderer.
    pub content_text: String,
    /// Open/closed runtime state for `BevyApp`.
    pub open: bool,
}

impl Default for DialogWidget {
    fn default() -> Self {
        Self {
            trigger: None,
            renderer: DialogProvider::BevyApp,
            dialog_type: DialogWidgetType::Info,
            content_text: String::new(),
            open: false,
        }
    }
}

/// Canonical modal kind.
#[derive(Debug, Clone, Copy, Reflect, Default, Eq, PartialEq)]
pub enum DialogModalKind {
    #[default]
    Default,
    Failure,
    Question,
    Blank,
}

/// Dialog content model.
#[derive(Debug, Clone, Reflect, Eq, PartialEq)]
pub enum DialogModalType {
    /// Simple modal with title, content and a close button.
    Default,
    /// Error modal with required error code and a confirmation button.
    Failure {
        error_code: String,
        confirm_label: String,
    },
    /// Question modal with two footer buttons and a close option in header.
    Question {
        confirm_label: String,
        cancel_label: String,
    },
    /// Empty dialog shell (no header/body/footer).
    Blank,
}

impl Default for DialogModalType {
    fn default() -> Self {
        Self::Default
    }
}

impl DialogModalType {
    /// Returns the canonical modal kind without payload details.
    pub fn kind(&self) -> DialogModalKind {
        match self {
            DialogModalType::Default => DialogModalKind::Default,
            DialogModalType::Failure { .. } => DialogModalKind::Failure,
            DialogModalType::Question { .. } => DialogModalKind::Question,
            DialogModalType::Blank => DialogModalKind::Blank,
        }
    }
}

/// Full dialog configuration payload.
#[derive(Debug, Clone, Reflect)]
pub struct DialogConfig {
    pub provider: DialogProvider,
    pub layout: DialogLayout,
    pub modal: DialogModalType,
    pub title: String,
    pub content: String,
    /// Allows dismissing a Bevy dialog by clicking outside the panel.
    pub close_on_backdrop: bool,
}

impl Default for DialogConfig {
    fn default() -> Self {
        Self {
            provider: DialogProvider::default(),
            layout: DialogLayout::default(),
            modal: DialogModalType::default(),
            title: String::new(),
            content: String::new(),
            close_on_backdrop: false,
        }
    }
}

impl DialogConfig {
    /// Creates a simple default dialog.
    pub fn default_modal(title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            content: content.into(),
            modal: DialogModalType::Default,
            ..default()
        }
    }

    /// Creates an error dialog with required error code.
    pub fn failure(
        title: impl Into<String>,
        content: impl Into<String>,
        error_code: impl Into<String>,
    ) -> Self {
        Self {
            title: title.into(),
            content: content.into(),
            modal: DialogModalType::Failure {
                error_code: error_code.into(),
                confirm_label: "Confirm".to_string(),
            },
            close_on_backdrop: false,
            ..default()
        }
    }

    /// Creates a question dialog with confirm/cancel actions.
    pub fn question(title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            content: content.into(),
            modal: DialogModalType::Question {
                confirm_label: "Confirm".to_string(),
                cancel_label: "Cancel".to_string(),
            },
            close_on_backdrop: true,
            ..default()
        }
    }

    /// Creates an empty dialog shell for manual composition.
    pub fn blank(layout: DialogLayout) -> Self {
        Self {
            layout,
            modal: DialogModalType::Blank,
            ..default()
        }
    }
}

/// Request message to open a dialog.
#[derive(Message, Clone, Debug)]
pub struct ShowDialog {
    pub request_id: Option<u64>,
    pub config: DialogConfig,
}

impl ShowDialog {
    pub fn new(config: DialogConfig) -> Self {
        Self {
            request_id: None,
            config,
        }
    }
}

/// User action result when a dialog closes.
#[derive(Debug, Clone, Copy, Reflect, Default, Eq, PartialEq)]
pub enum DialogResult {
    /// Closed via header close affordance.
    #[default]
    Closed,
    /// Confirmed via primary action button.
    Confirmed,
    /// Cancelled via secondary action button.
    Cancelled,
    /// Dismissed via backdrop click.
    Dismissed,
    /// Dialog type/provider combination is not available.
    Unavailable,
}

/// Message emitted when a dialog has closed.
#[derive(Message, Clone, Debug)]
pub struct DialogClosed {
    pub request_id: u64,
    pub provider: DialogProvider,
    pub modal: DialogModalKind,
    pub result: DialogResult,
}

/// Message emitted for a spawned Bevy dialog root.
///
/// `panel` can be used to manually add children (especially for `Blank` modals).
#[derive(Message, Clone, Debug)]
pub struct DialogSpawned {
    pub request_id: u64,
    pub modal: DialogModalKind,
    pub root: Entity,
    pub panel: Entity,
}

#[derive(Resource, Debug)]
struct DialogRuntimeState {
    next_request_id: u64,
    next_z_index: i32,
}

impl Default for DialogRuntimeState {
    fn default() -> Self {
        Self {
            next_request_id: 0,
            next_z_index: DIALOG_RUNTIME_BASE_Z_INDEX,
        }
    }
}

#[derive(Component, Debug, Clone, Copy)]
struct DialogAction {
    request_id: u64,
    root: Entity,
    provider: DialogProvider,
    modal: DialogModalKind,
    result: DialogResult,
}

#[derive(Component, Debug, Clone, Copy)]
struct DialogBackdropAction {
    request_id: u64,
    root: Entity,
    provider: DialogProvider,
    modal: DialogModalKind,
}

#[derive(Component, Debug, Clone, Copy)]
struct DialogWidgetBase;

#[derive(Component, Debug, Clone, Copy)]
struct DialogWidgetPanel;

#[derive(Component, Debug, Clone)]
struct DialogTriggerTargets(Vec<Entity>);

#[derive(Component, Debug, Clone, Copy)]
struct DialogTriggerObserver;

#[derive(Component, Debug, Clone, Copy)]
struct DialogTriggerBound;

#[derive(Component, Debug, Clone, Copy)]
struct DialogWidgetButtonAction {
    dialog: Entity,
}

/// Public marker component for Bevy in-window dialog overlays.
#[derive(Component, Debug, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct DialogOverlayWidget {
    pub request_id: u64,
    pub modal: DialogModalKind,
}

/// Public marker component for Bevy in-window dialog panels.
#[derive(Component, Debug, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct DialogPanelWidget {
    pub request_id: u64,
    pub modal: DialogModalKind,
    pub layout: DialogLayout,
}

/// Plugin wiring the dialog request/response pipeline.
pub struct ExtendedDialogPlugin;

impl Plugin for ExtendedDialogPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DialogRuntimeState>();
        app.register_type::<DialogProvider>();
        app.register_type::<DialogLayout>();
        app.register_type::<DialogModalKind>();
        app.register_type::<DialogModalType>();
        app.register_type::<DialogWidgetType>();
        app.register_type::<DialogWidget>();
        app.register_type::<DialogConfig>();
        app.register_type::<DialogResult>();
        app.register_type::<DialogOverlayWidget>();
        app.register_type::<DialogPanelWidget>();
        app.add_message::<ShowDialog>();
        app.add_message::<DialogClosed>();
        app.add_message::<DialogSpawned>();
        app.add_systems(
            Update,
            (
                process_dialog_requests,
                initialize_dialog_widgets,
                bind_dialog_triggers,
                sync_dialog_widget_visibility,
            ),
        );
    }
}

fn process_dialog_requests(
    mut commands: Commands,
    config: Res<ExtendedUiConfiguration>,
    mut runtime: ResMut<DialogRuntimeState>,
    mut requests: MessageReader<ShowDialog>,
    mut dialog_closed: MessageWriter<DialogClosed>,
    mut dialog_spawned: MessageWriter<DialogSpawned>,
) {
    let layer = *config.render_layers.first().unwrap_or(&1);

    for request in requests.read() {
        let request_id = resolve_request_id(&mut runtime, request.request_id);
        let modal_kind = request.config.modal.kind();

        match request.config.provider {
            DialogProvider::BevyApp => {
                let (root, panel) = spawn_bevy_dialog(
                    &mut commands,
                    &request.config,
                    request_id,
                    modal_kind,
                    &mut runtime,
                    layer,
                );
                dialog_spawned.write(DialogSpawned {
                    request_id,
                    modal: modal_kind,
                    root,
                    panel,
                });
            }
            DialogProvider::System => {
                let result = show_system_dialog(&request.config);
                dialog_closed.write(DialogClosed {
                    request_id,
                    provider: DialogProvider::System,
                    modal: modal_kind,
                    result,
                });
            }
        }
    }
}

fn resolve_request_id(runtime: &mut DialogRuntimeState, requested: Option<u64>) -> u64 {
    if let Some(id) = requested {
        runtime.next_request_id = runtime.next_request_id.max(id);
        return id;
    }

    runtime.next_request_id += 1;
    runtime.next_request_id
}

fn initialize_dialog_widgets(
    mut commands: Commands,
    config: Res<ExtendedUiConfiguration>,
    parent_query: Query<&ChildOf>,
    body_query: Query<(), With<Body>>,
    child_classes: Query<&CssClass>,
    dialogs: Query<
        (
            Entity,
            &DialogWidget,
            Option<&CssClass>,
            Option<&Children>,
            Option<&CssID>,
            Option<&HtmlStyle>,
        ),
        (With<DialogWidget>, Without<DialogWidgetBase>),
    >,
) {
    let layer = *config.render_layers.first().unwrap_or(&1);

    for (entity, dialog, class_opt, children_opt, css_id_opt, html_style_opt) in dialogs.iter() {
        if let Some(body_entity) = find_body_ancestor(entity, &parent_query, &body_query) {
            commands.entity(body_entity).add_child(entity);
        }

        let dialog_name = css_id_opt
            .map(|id| id.0.trim())
            .filter(|id| !id.is_empty())
            .map(|id| format!("DialogWidget-{}-{entity:?}", id))
            .unwrap_or_else(|| format!("DialogWidget-{entity:?}"));

        let mut classes = class_opt
            .map(|classes| classes.0.clone())
            .unwrap_or_default();
        push_unique_class(&mut classes, "dialog-widget");
        push_unique_class(
            &mut classes,
            match dialog.renderer {
                DialogProvider::BevyApp => "dialog-renderer-bevy-app",
                DialogProvider::System => "dialog-renderer-system",
            },
        );
        push_unique_class(&mut classes, dialog_widget_type_class(dialog.dialog_type));

        let mut root_node = Node::default();
        root_node.position_type = PositionType::Absolute;
        root_node.left = Val::Px(0.0);
        root_node.right = Val::Px(0.0);
        root_node.top = Val::Px(0.0);
        root_node.bottom = Val::Px(0.0);
        root_node.width = Val::Percent(100.0);
        root_node.height = Val::Percent(100.0);
        root_node.justify_content = JustifyContent::Center;
        root_node.align_items = AlignItems::Center;
        root_node.padding = UiRect::all(Val::Px(16.0));

        let mut overlay_style = html_style_opt
            .map(|style| style.0.clone())
            .unwrap_or_default();
        overlay_style.display = Some(Display::Flex);
        overlay_style.position_type = Some(PositionType::Absolute);
        overlay_style.left = Some(Val::Px(0.0));
        overlay_style.right = Some(Val::Px(0.0));
        overlay_style.top = Some(Val::Px(0.0));
        overlay_style.bottom = Some(Val::Px(0.0));
        overlay_style.width = Some(Val::Percent(100.0));
        overlay_style.height = Some(Val::Percent(100.0));
        overlay_style.justify_content = Some(JustifyContent::Center);
        overlay_style.align_items = Some(AlignItems::Center);
        overlay_style.padding = Some(UiRect::all(Val::Px(16.0)));
        overlay_style.z_index = Some(DIALOG_WIDGET_Z_INDEX);
        overlay_style.background = Some(Background {
            color: Color::srgba(0.0, 0.0, 0.0, 0.55),
            image: None,
            gradient: None,
        });
        overlay_style.backdrop_filter = Some(BackdropFilter::Blur(15.0));

        commands.entity(entity).insert((
            Name::new(dialog_name),
            root_node,
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
            ZIndex(DIALOG_WIDGET_Z_INDEX),
            GlobalZIndex(DIALOG_WIDGET_Z_INDEX),
            RenderLayers::layer(layer),
            Pickable::default(),
            TagName("dialog".to_string()),
            CssClass(classes),
            DialogWidgetBase,
            Visibility::Hidden,
            HtmlStyle(overlay_style),
        ));

        if dialog.renderer == DialogProvider::BevyApp {
            commands
                .entity(entity)
                .observe(on_dialog_widget_overlay_click);

            let mut panel_node = Node::default();
            panel_node.width = Val::Px(520.0);
            panel_node.max_width = Val::Percent(95.0);
            panel_node.min_width = Val::Px(260.0);
            panel_node.padding = UiRect::all(Val::Px(16.0));
            panel_node.row_gap = Val::Px(8.0);
            panel_node.flex_direction = FlexDirection::Column;
            panel_node.border = UiRect::all(Val::Px(1.0));
            panel_node.border_radius = BorderRadius::all(Val::Px(12.0));

            let panel = commands
                .spawn((
                    Name::new(format!("DialogWidgetPanel-{entity:?}")),
                    panel_node,
                    BackgroundColor(Color::srgb(0.12, 0.15, 0.2)),
                    BorderColor::all(Color::srgb(0.28, 0.33, 0.45)),
                    RenderLayers::layer(layer),
                    Pickable::default(),
                    TagName("div".to_string()),
                    CssClass(vec![
                        "dialog-panel".to_string(),
                        dialog_widget_type_class(dialog.dialog_type).to_string(),
                    ]),
                    DialogWidgetPanel,
                ))
                .observe(stop_dialog_click_propagation)
                .id();

            commands.entity(entity).add_child(panel);

            let existing_children: Vec<Entity> = children_opt
                .map(|children| children.iter().collect())
                .unwrap_or_default();

            let mut header = None;
            let mut body = None;
            let mut footer = None;
            let mut content_children = Vec::new();

            for child in existing_children {
                let class_opt = child_classes.get(child).ok();
                if header.is_none() && class_has(class_opt, "dialog-header") {
                    header = Some(child);
                    continue;
                }
                if body.is_none() && class_has(class_opt, "dialog-body") {
                    body = Some(child);
                    continue;
                }
                if footer.is_none() && class_has(class_opt, "dialog-footer") {
                    footer = Some(child);
                    continue;
                }
                content_children.push(child);
            }

            let build_defaults = dialog.dialog_type != DialogWidgetType::Blank;
            if build_defaults {
                if header.is_none() {
                    header = Some(spawn_default_dialog_header(&mut commands, panel, layer));
                }
                if body.is_none() {
                    body = Some(spawn_default_dialog_body(&mut commands, panel, layer));
                }
                if footer.is_none() {
                    footer = Some(spawn_default_dialog_footer(
                        &mut commands,
                        panel,
                        entity,
                        layer,
                        dialog.dialog_type,
                    ));
                }
            }

            if let Some(header_entity) = header {
                commands.entity(panel).add_child(header_entity);
            }

            if let Some(body_entity) = body {
                commands.entity(panel).add_child(body_entity);
                for child in content_children {
                    commands.entity(body_entity).add_child(child);
                }
            } else {
                for child in content_children {
                    commands.entity(panel).add_child(child);
                }
            }

            if let Some(footer_entity) = footer {
                commands.entity(panel).add_child(footer_entity);
            }
        }
    }
}

fn find_body_ancestor(
    start: Entity,
    parents: &Query<&ChildOf>,
    bodies: &Query<(), With<Body>>,
) -> Option<Entity> {
    let mut current = start;
    while let Ok(parent) = parents.get(current) {
        let parent_entity = parent.parent();
        if bodies.get(parent_entity).is_ok() {
            return Some(parent_entity);
        }
        current = parent_entity;
    }
    None
}

fn class_has(class_opt: Option<&CssClass>, expected: &str) -> bool {
    class_opt
        .map(|classes| classes.0.iter().any(|class_name| class_name == expected))
        .unwrap_or(false)
}

fn spawn_default_dialog_header(commands: &mut Commands, panel: Entity, layer: usize) -> Entity {
    let mut node = Node::default();
    node.width = Val::Percent(100.0);
    node.padding = UiRect::all(Val::Px(10.0));
    node.display = Display::Flex;
    node.justify_content = JustifyContent::Start;
    node.align_items = AlignItems::Center;

    let header = commands
        .spawn((
            Name::new("DialogDefaultHeader"),
            node,
            RenderLayers::layer(layer),
            TagName("dialog-header".to_string()),
            CssClass(vec!["dialog-header".to_string()]),
        ))
        .id();

    let mut title_font = TextFont::default();
    title_font.font_size = 20.0;

    let title = commands
        .spawn((
            Name::new("DialogDefaultHeaderText"),
            Text::new("Dialog"),
            title_font,
            TextColor(Color::srgb(0.95, 0.96, 0.99)),
            RenderLayers::layer(layer),
            TagName("h5".to_string()),
            CssClass(vec!["dialog-header-title".to_string()]),
        ))
        .id();

    commands.entity(header).add_child(title);
    let _ = panel;
    header
}

fn spawn_default_dialog_body(commands: &mut Commands, panel: Entity, layer: usize) -> Entity {
    let mut node = Node::default();
    node.width = Val::Percent(100.0);
    node.min_width = Val::Px(200.0);
    node.min_height = Val::Px(75.0);
    node.padding = UiRect::all(Val::Px(10.0));
    node.display = Display::Flex;
    node.flex_direction = FlexDirection::Column;
    node.row_gap = Val::Px(8.0);

    let body = commands
        .spawn((
            Name::new("DialogDefaultBody"),
            node,
            RenderLayers::layer(layer),
            TagName("dialog-body".to_string()),
            CssClass(vec!["dialog-body".to_string()]),
        ))
        .id();

    let _ = panel;
    body
}

fn spawn_default_dialog_footer(
    commands: &mut Commands,
    panel: Entity,
    dialog_entity: Entity,
    layer: usize,
    dialog_type: DialogWidgetType,
) -> Entity {
    let mut node = Node::default();
    node.width = Val::Percent(100.0);
    node.padding = UiRect::all(Val::Px(10.0));
    node.display = Display::Flex;
    node.justify_content = JustifyContent::End;
    node.align_items = AlignItems::Center;
    node.column_gap = Val::Px(8.0);

    let footer = commands
        .spawn((
            Name::new("DialogDefaultFooter"),
            node,
            RenderLayers::layer(layer),
            TagName("dialog-footer".to_string()),
            CssClass(vec!["dialog-footer".to_string()]),
        ))
        .id();

    let cancel_bg = Color::srgb(0.35, 0.38, 0.45);
    let ok_color = match dialog_type {
        DialogWidgetType::Error => Color::srgb(0.71, 0.2, 0.26),
        DialogWidgetType::Warn => Color::srgb(0.72, 0.56, 0.2),
        DialogWidgetType::Info | DialogWidgetType::Blank => Color::srgb(0.25, 0.41, 0.82),
    };

    match dialog_type {
        DialogWidgetType::Error => {
            spawn_default_dialog_footer_button(
                commands,
                footer,
                dialog_entity,
                layer,
                "Ok",
                ok_color,
                "dialog-footer-button-ok",
            );
        }
        DialogWidgetType::Warn | DialogWidgetType::Info => {
            spawn_default_dialog_footer_button(
                commands,
                footer,
                dialog_entity,
                layer,
                "Cancel",
                cancel_bg,
                "dialog-footer-button-cancel",
            );
            spawn_default_dialog_footer_button(
                commands,
                footer,
                dialog_entity,
                layer,
                "Ok",
                ok_color,
                "dialog-footer-button-ok",
            );
        }
        DialogWidgetType::Blank => {}
    }

    let _ = panel;
    footer
}

fn spawn_default_dialog_footer_button(
    commands: &mut Commands,
    footer: Entity,
    dialog_entity: Entity,
    layer: usize,
    label: &str,
    background_color: Color,
    role_class: &str,
) -> Entity {
    let mut node = Node::default();
    node.min_width = Val::Px(82.0);
    node.height = Val::Px(34.0);
    node.padding = UiRect::axes(Val::Px(12.0), Val::Px(8.0));
    node.display = Display::Flex;
    node.justify_content = JustifyContent::Center;
    node.align_items = AlignItems::Center;
    node.border = UiRect::all(Val::Px(1.0));
    node.border_radius = BorderRadius::all(Val::Px(6.0));

    let button = commands
        .spawn((
            Name::new(format!("DialogDefaultButton-{label}")),
            node,
            BackgroundColor(background_color),
            BorderColor::all(Color::srgba(0.0, 0.0, 0.0, 0.2)),
            RenderLayers::layer(layer),
            Pickable::default(),
            TagName("button".to_string()),
            CssClass(vec![
                "dialog-footer-button".to_string(),
                role_class.to_string(),
            ]),
            DialogWidgetButtonAction {
                dialog: dialog_entity,
            },
        ))
        .observe(on_dialog_widget_button_click)
        .observe(stop_dialog_click_propagation)
        .id();

    let mut font = TextFont::default();
    font.font_size = 14.0;

    let text = commands
        .spawn((
            Name::new(format!("DialogDefaultButtonText-{label}")),
            Text::new(label.to_string()),
            font,
            TextColor(Color::WHITE),
            RenderLayers::layer(layer),
            TagName("p".to_string()),
            CssClass(vec!["dialog-footer-button-label".to_string()]),
        ))
        .id();

    commands.entity(button).add_child(text);
    commands.entity(footer).add_child(button);
    button
}

fn on_dialog_widget_button_click(
    mut trigger: On<Pointer<Click>>,
    mut dialogs: Query<&mut DialogWidget>,
    actions: Query<&DialogWidgetButtonAction>,
) {
    let Ok(action) = actions.get(trigger.entity) else {
        return;
    };
    if let Ok(mut dialog) = dialogs.get_mut(action.dialog) {
        dialog.open = false;
    }
    trigger.propagate(false);
}

fn bind_dialog_triggers(
    mut commands: Commands,
    dialogs: Query<(Entity, &DialogWidget, Option<&DialogTriggerBound>), With<DialogWidget>>,
    targets: Query<(Entity, &CssID)>,
    mut target_lists: Query<&mut DialogTriggerTargets>,
    trigger_observers: Query<(), With<DialogTriggerObserver>>,
) {
    for (dialog_entity, dialog, bound_opt) in dialogs.iter() {
        if bound_opt.is_some() {
            continue;
        }

        let Some(trigger_id) = dialog.trigger.as_ref().map(|id| normalize_trigger_id(id)) else {
            continue;
        };
        if trigger_id.is_empty() {
            continue;
        }

        let Some((target_entity, _)) = targets
            .iter()
            .find(|(_, css_id)| css_id.0.trim() == trigger_id)
        else {
            continue;
        };

        if let Ok(mut list) = target_lists.get_mut(target_entity) {
            if !list.0.contains(&dialog_entity) {
                list.0.push(dialog_entity);
            }
        } else {
            commands
                .entity(target_entity)
                .insert(DialogTriggerTargets(vec![dialog_entity]));
        }

        if trigger_observers.get(target_entity).is_err() {
            commands
                .entity(target_entity)
                .insert(DialogTriggerObserver)
                .observe(on_dialog_trigger_click);
        }

        commands.entity(dialog_entity).insert(DialogTriggerBound);
    }
}

fn sync_dialog_widget_visibility(
    mut dialogs: Query<
        (&DialogWidget, &mut Visibility),
        Or<(Added<DialogWidget>, Changed<DialogWidget>)>,
    >,
) {
    for (dialog, mut visibility) in dialogs.iter_mut() {
        let should_show = dialog.renderer == DialogProvider::BevyApp && dialog.open;
        *visibility = if should_show {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

fn on_dialog_trigger_click(
    trigger: On<Pointer<Click>>,
    targets: Query<&DialogTriggerTargets>,
    mut dialogs: Query<&mut DialogWidget>,
) {
    let Ok(targets) = targets.get(trigger.entity) else {
        return;
    };

    for dialog_entity in targets.0.iter().copied() {
        let Ok(mut dialog) = dialogs.get_mut(dialog_entity) else {
            continue;
        };

        match dialog.renderer {
            DialogProvider::BevyApp => {
                dialog.open = true;
            }
            DialogProvider::System => {
                let _ = show_system_message(dialog.dialog_type, &dialog.content_text);
            }
        }
    }
}

fn on_dialog_widget_overlay_click(
    mut trigger: On<Pointer<Click>>,
    mut dialogs: Query<&mut DialogWidget>,
) {
    let Ok(mut dialog) = dialogs.get_mut(trigger.entity) else {
        return;
    };
    if dialog.renderer == DialogProvider::BevyApp {
        dialog.open = false;
    }
    trigger.propagate(false);
}

fn normalize_trigger_id(input: &str) -> String {
    input.trim().trim_start_matches('#').to_string()
}

fn push_unique_class(classes: &mut Vec<String>, class_name: &str) {
    if !classes.iter().any(|existing| existing == class_name) {
        classes.push(class_name.to_string());
    }
}

fn dialog_widget_type_class(kind: DialogWidgetType) -> &'static str {
    match kind {
        DialogWidgetType::Warn => "dialog-type-warn",
        DialogWidgetType::Error => "dialog-type-error",
        DialogWidgetType::Info => "dialog-type-info",
        DialogWidgetType::Blank => "dialog-type-blank",
    }
}

fn spawn_bevy_dialog(
    commands: &mut Commands,
    dialog: &DialogConfig,
    request_id: u64,
    modal_kind: DialogModalKind,
    runtime: &mut DialogRuntimeState,
    layer: usize,
) -> (Entity, Entity) {
    let z_index = runtime.next_z_index;
    runtime.next_z_index += 5;

    let mut root_node = Node::default();
    root_node.position_type = PositionType::Absolute;
    root_node.left = Val::Px(0.0);
    root_node.right = Val::Px(0.0);
    root_node.top = Val::Px(0.0);
    root_node.bottom = Val::Px(0.0);
    root_node.width = Val::Percent(100.0);
    root_node.height = Val::Percent(100.0);
    root_node.justify_content = JustifyContent::Center;
    root_node.align_items = match dialog.layout {
        DialogLayout::FloatingPanel => AlignItems::Center,
        DialogLayout::BottomSheet => AlignItems::End,
    };
    root_node.padding = UiRect::all(Val::Px(16.0));

    let root = commands
        .spawn((
            Name::new(format!("DialogOverlay-{request_id}")),
            root_node,
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.45)),
            ZIndex(z_index),
            GlobalZIndex(z_index),
            RenderLayers::layer(layer),
            Pickable::default(),
            UIWidgetState::default(),
            DialogOverlayWidget {
                request_id,
                modal: modal_kind,
            },
            TagName("dialog-overlay".to_string()),
            CssClass(vec!["dialog-overlay".to_string()]),
        ))
        .id();

    if dialog.close_on_backdrop {
        commands
            .entity(root)
            .insert(DialogBackdropAction {
                request_id,
                root,
                provider: DialogProvider::BevyApp,
                modal: modal_kind,
            })
            .observe(on_dialog_backdrop_click);
    }

    let mut panel_node = Node::default();
    panel_node.flex_direction = FlexDirection::Column;
    panel_node.row_gap = Val::Px(12.0);
    panel_node.padding = UiRect::all(Val::Px(16.0));
    panel_node.border = UiRect::all(Val::Px(1.0));
    panel_node.border_radius = BorderRadius::all(Val::Px(12.0));
    panel_node.min_width = Val::Px(280.0);
    panel_node.max_width = Val::Px(860.0);
    panel_node.width = match dialog.layout {
        DialogLayout::FloatingPanel => Val::Px(520.0),
        DialogLayout::BottomSheet => Val::Percent(100.0),
    };

    let panel = commands
        .spawn((
            Name::new(format!("DialogPanel-{request_id}")),
            panel_node,
            BackgroundColor(Color::srgb(0.1, 0.12, 0.16)),
            BorderColor::all(Color::srgb(0.2, 0.24, 0.32)),
            RenderLayers::layer(layer),
            Pickable::default(),
            UIWidgetState::default(),
            DialogPanelWidget {
                request_id,
                modal: modal_kind,
                layout: dialog.layout,
            },
            TagName("dialog-panel".to_string()),
            CssClass(vec![
                "dialog-panel".to_string(),
                match dialog.layout {
                    DialogLayout::FloatingPanel => "dialog-floating-panel".to_string(),
                    DialogLayout::BottomSheet => "dialog-bottom-sheet".to_string(),
                },
                format!("dialog-modal-{}", modal_class_suffix(modal_kind)),
            ]),
        ))
        .observe(stop_dialog_click_propagation)
        .id();

    commands.entity(root).add_child(panel);
    populate_dialog_panel(commands, panel, root, dialog, request_id, modal_kind, layer);

    (root, panel)
}

fn populate_dialog_panel(
    commands: &mut Commands,
    panel: Entity,
    root: Entity,
    dialog: &DialogConfig,
    request_id: u64,
    modal_kind: DialogModalKind,
    layer: usize,
) {
    match &dialog.modal {
        DialogModalType::Blank => {}
        DialogModalType::Default => {
            spawn_header(
                commands,
                panel,
                &dialog.title,
                Some(("Close".to_string(), DialogResult::Closed)),
                request_id,
                root,
                modal_kind,
                layer,
            );
            spawn_body(commands, panel, &dialog.content, layer, None);
        }
        DialogModalType::Failure {
            error_code,
            confirm_label,
        } => {
            spawn_header(
                commands,
                panel,
                &dialog.title,
                None,
                request_id,
                root,
                modal_kind,
                layer,
            );
            let detail = format!("{}\nError code: {}", dialog.content, error_code);
            spawn_body(commands, panel, &detail, layer, Some("dialog-error-code"));
            spawn_footer(
                commands,
                panel,
                vec![(confirm_label.clone(), DialogResult::Confirmed)],
                request_id,
                root,
                modal_kind,
                layer,
            );
        }
        DialogModalType::Question {
            confirm_label,
            cancel_label,
        } => {
            spawn_header(
                commands,
                panel,
                &dialog.title,
                Some(("Close".to_string(), DialogResult::Closed)),
                request_id,
                root,
                modal_kind,
                layer,
            );
            spawn_body(commands, panel, &dialog.content, layer, None);
            spawn_footer(
                commands,
                panel,
                vec![
                    (cancel_label.clone(), DialogResult::Cancelled),
                    (confirm_label.clone(), DialogResult::Confirmed),
                ],
                request_id,
                root,
                modal_kind,
                layer,
            );
        }
    }
}

fn spawn_header(
    commands: &mut Commands,
    panel: Entity,
    title: &str,
    close_button: Option<(String, DialogResult)>,
    request_id: u64,
    root: Entity,
    modal_kind: DialogModalKind,
    layer: usize,
) {
    let mut header_node = Node::default();
    header_node.width = Val::Percent(100.0);
    header_node.display = Display::Flex;
    header_node.justify_content = JustifyContent::SpaceBetween;
    header_node.align_items = AlignItems::Center;
    header_node.column_gap = Val::Px(8.0);

    let header = commands
        .spawn((
            Name::new(format!("DialogHeader-{request_id}")),
            header_node,
            RenderLayers::layer(layer),
            TagName("dialog-header".to_string()),
            CssClass(vec!["dialog-header".to_string()]),
        ))
        .id();

    let mut title_font = TextFont::default();
    title_font.font_size = 21.0;

    let title_entity = commands
        .spawn((
            Name::new(format!("DialogTitle-{request_id}")),
            Text::new(title.to_string()),
            title_font,
            TextColor(Color::srgb(0.94, 0.95, 0.98)),
            RenderLayers::layer(layer),
            TagName("dialog-title".to_string()),
            CssClass(vec!["dialog-title".to_string()]),
        ))
        .id();
    commands.entity(header).add_child(title_entity);

    if let Some((label, result)) = close_button {
        let button = spawn_action_button(
            commands,
            request_id,
            root,
            modal_kind,
            label,
            result,
            layer,
            "dialog-close",
        );
        commands.entity(header).add_child(button);
    }

    commands.entity(panel).add_child(header);
}

fn spawn_body(
    commands: &mut Commands,
    panel: Entity,
    content: &str,
    layer: usize,
    extra_class: Option<&str>,
) {
    let mut body_node = Node::default();
    body_node.width = Val::Percent(100.0);

    let mut classes = vec!["dialog-body".to_string()];
    if let Some(class) = extra_class {
        classes.push(class.to_string());
    }

    let body = commands
        .spawn((
            Name::new("DialogBody"),
            body_node,
            RenderLayers::layer(layer),
            TagName("dialog-body".to_string()),
            CssClass(classes),
        ))
        .id();

    let mut body_font = TextFont::default();
    body_font.font_size = 16.0;

    let content_entity = commands
        .spawn((
            Name::new("DialogContent"),
            Text::new(content.to_string()),
            body_font,
            TextColor(Color::srgb(0.84, 0.86, 0.92)),
            RenderLayers::layer(layer),
            TagName("dialog-content".to_string()),
            CssClass(vec!["dialog-content".to_string()]),
        ))
        .id();

    commands.entity(body).add_child(content_entity);
    commands.entity(panel).add_child(body);
}

fn spawn_footer(
    commands: &mut Commands,
    panel: Entity,
    buttons: Vec<(String, DialogResult)>,
    request_id: u64,
    root: Entity,
    modal_kind: DialogModalKind,
    layer: usize,
) {
    let mut footer_node = Node::default();
    footer_node.width = Val::Percent(100.0);
    footer_node.display = Display::Flex;
    footer_node.justify_content = JustifyContent::End;
    footer_node.align_items = AlignItems::Center;
    footer_node.column_gap = Val::Px(8.0);

    let footer = commands
        .spawn((
            Name::new(format!("DialogFooter-{request_id}")),
            footer_node,
            RenderLayers::layer(layer),
            TagName("dialog-footer".to_string()),
            CssClass(vec!["dialog-footer".to_string()]),
        ))
        .id();

    for (label, result) in buttons {
        let button = spawn_action_button(
            commands,
            request_id,
            root,
            modal_kind,
            label,
            result,
            layer,
            "dialog-action",
        );
        commands.entity(footer).add_child(button);
    }

    commands.entity(panel).add_child(footer);
}

fn spawn_action_button(
    commands: &mut Commands,
    request_id: u64,
    root: Entity,
    modal_kind: DialogModalKind,
    label: String,
    result: DialogResult,
    layer: usize,
    role_class: &str,
) -> Entity {
    let mut button_node = Node::default();
    button_node.min_width = Val::Px(88.0);
    button_node.height = Val::Px(36.0);
    button_node.padding = UiRect::axes(Val::Px(12.0), Val::Px(8.0));
    button_node.justify_content = JustifyContent::Center;
    button_node.align_items = AlignItems::Center;
    button_node.border = UiRect::all(Val::Px(1.0));
    button_node.border_radius = BorderRadius::all(Val::Px(8.0));

    let button = commands
        .spawn((
            Name::new(format!("DialogButton-{request_id}-{result:?}")),
            button_node,
            BackgroundColor(Color::srgb(0.23, 0.28, 0.39)),
            BorderColor::all(Color::srgb(0.36, 0.42, 0.55)),
            RenderLayers::layer(layer),
            Pickable::default(),
            UIWidgetState::default(),
            TagName("button".to_string()),
            CssClass(vec![
                "dialog-button".to_string(),
                role_class.to_string(),
                format!("dialog-modal-{}", modal_class_suffix(modal_kind)),
            ]),
            DialogAction {
                request_id,
                root,
                provider: DialogProvider::BevyApp,
                modal: modal_kind,
                result,
            },
        ))
        .observe(on_dialog_action_click)
        .observe(stop_dialog_click_propagation)
        .id();

    let mut label_font = TextFont::default();
    label_font.font_size = 15.0;

    let label_entity = commands
        .spawn((
            Name::new("DialogButtonLabel"),
            Text::new(label),
            label_font,
            TextColor(Color::srgb(0.97, 0.98, 0.99)),
            RenderLayers::layer(layer),
            TagName("dialog-button-label".to_string()),
            CssClass(vec!["dialog-button-label".to_string()]),
        ))
        .id();

    commands.entity(button).add_child(label_entity);
    button
}

fn modal_class_suffix(kind: DialogModalKind) -> &'static str {
    match kind {
        DialogModalKind::Default => "default",
        DialogModalKind::Failure => "failure",
        DialogModalKind::Question => "question",
        DialogModalKind::Blank => "blank",
    }
}

fn stop_dialog_click_propagation(mut trigger: On<Pointer<Click>>) {
    trigger.propagate(false);
}

fn on_dialog_backdrop_click(
    mut trigger: On<Pointer<Click>>,
    mut commands: Commands,
    query: Query<&DialogBackdropAction>,
    mut closed_writer: MessageWriter<DialogClosed>,
) {
    let Ok(action) = query.get(trigger.entity) else {
        return;
    };

    closed_writer.write(DialogClosed {
        request_id: action.request_id,
        provider: action.provider,
        modal: action.modal,
        result: DialogResult::Dismissed,
    });
    if commands.get_entity(action.root).is_ok() {
        commands.entity(action.root).despawn();
    }
    trigger.propagate(false);
}

fn on_dialog_action_click(
    mut trigger: On<Pointer<Click>>,
    mut commands: Commands,
    query: Query<&DialogAction>,
    mut closed_writer: MessageWriter<DialogClosed>,
) {
    let Ok(action) = query.get(trigger.entity) else {
        return;
    };

    closed_writer.write(DialogClosed {
        request_id: action.request_id,
        provider: action.provider,
        modal: action.modal,
        result: action.result,
    });
    if commands.get_entity(action.root).is_ok() {
        commands.entity(action.root).despawn();
    }
    trigger.propagate(false);
}

#[cfg(all(target_os = "linux", not(target_arch = "wasm32")))]
fn spawn_linux_system_dialog_task(task: impl FnOnce() + Send + 'static) -> bool {
    if SYSTEM_DIALOG_IN_FLIGHT
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        warn!(
            "System dialog request ignored: another system dialog is already open (linux backend)."
        );
        return false;
    }

    std::thread::spawn(move || {
        task();
        SYSTEM_DIALOG_IN_FLIGHT.store(false, Ordering::Release);
    });

    true
}

#[cfg(not(target_arch = "wasm32"))]
fn show_system_message_blocking(kind: DialogWidgetType, content: &str) -> DialogResult {
    use rfd::{MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};

    if matches!(kind, DialogWidgetType::Blank) {
        return DialogResult::Unavailable;
    }

    let title = match kind {
        DialogWidgetType::Warn => "Warning",
        DialogWidgetType::Error => "Error",
        DialogWidgetType::Info => "Info",
        DialogWidgetType::Blank => "Dialog",
    };

    let level = match kind {
        DialogWidgetType::Warn => MessageLevel::Warning,
        DialogWidgetType::Error => MessageLevel::Error,
        DialogWidgetType::Info | DialogWidgetType::Blank => MessageLevel::Info,
    };

    let result = MessageDialog::new()
        .set_title(title)
        .set_description(content)
        .set_buttons(MessageButtons::Ok)
        .set_level(level)
        .show();

    match result {
        MessageDialogResult::Ok | MessageDialogResult::Yes => DialogResult::Confirmed,
        MessageDialogResult::No | MessageDialogResult::Cancel => DialogResult::Cancelled,
        _ => DialogResult::Closed,
    }
}

#[cfg(all(target_arch = "wasm32", feature = "clipboard-wasm"))]
fn show_wasm_alert_dialog(title: &str, content: &str) -> DialogResult {
    let message = if title.trim().is_empty() {
        content.to_string()
    } else {
        format!("{title}\n\n{content}")
    };

    let Some(window) = web_sys::window() else {
        return DialogResult::Unavailable;
    };

    match window.alert_with_message(&message) {
        Ok(()) => DialogResult::Closed,
        Err(_) => DialogResult::Unavailable,
    }
}

#[cfg(all(target_arch = "wasm32", feature = "clipboard-wasm"))]
fn show_wasm_confirm_dialog(title: &str, content: &str) -> DialogResult {
    let message = if title.trim().is_empty() {
        content.to_string()
    } else {
        format!("{title}\n\n{content}")
    };

    let Some(window) = web_sys::window() else {
        return DialogResult::Unavailable;
    };

    match window.confirm_with_message(&message) {
        Ok(true) => DialogResult::Confirmed,
        Ok(false) => DialogResult::Cancelled,
        Err(_) => DialogResult::Unavailable,
    }
}

fn show_system_message(kind: DialogWidgetType, content: &str) -> DialogResult {
    #[cfg(all(target_os = "linux", not(target_arch = "wasm32")))]
    {
        let content_owned = content.to_string();
        let started = spawn_linux_system_dialog_task(move || {
            let _ = show_system_message_blocking(kind, &content_owned);
        });
        return if started {
            DialogResult::Closed
        } else {
            DialogResult::Unavailable
        };
    }

    #[cfg(all(not(target_os = "linux"), not(target_arch = "wasm32")))]
    {
        return show_system_message_blocking(kind, content);
    }

    #[cfg(target_arch = "wasm32")]
    {
        if matches!(kind, DialogWidgetType::Blank) {
            return DialogResult::Unavailable;
        }

        let title = match kind {
            DialogWidgetType::Warn => "Warning",
            DialogWidgetType::Error => "Error",
            DialogWidgetType::Info => "Info",
            DialogWidgetType::Blank => "Dialog",
        };

        #[cfg(feature = "clipboard-wasm")]
        {
            return show_wasm_alert_dialog(title, content);
        }

        #[cfg(not(feature = "clipboard-wasm"))]
        {
            warn!("System dialogs are not supported on wasm targets without `clipboard-wasm`.");
            let _ = (title, content);
            return DialogResult::Unavailable;
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn show_system_dialog_blocking(dialog: &DialogConfig) -> DialogResult {
    use rfd::{MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};

    if matches!(dialog.modal, DialogModalType::Blank) {
        return DialogResult::Unavailable;
    }

    let (buttons, level, description) = match &dialog.modal {
        DialogModalType::Default => (
            MessageButtons::Ok,
            MessageLevel::Info,
            dialog.content.clone(),
        ),
        DialogModalType::Failure { error_code, .. } => (
            MessageButtons::Ok,
            MessageLevel::Error,
            format!("{}\nError code: {}", dialog.content, error_code),
        ),
        DialogModalType::Question { .. } => (
            MessageButtons::OkCancel,
            MessageLevel::Warning,
            dialog.content.clone(),
        ),
        DialogModalType::Blank => unreachable!("Blank dialogs return earlier"),
    };

    let title = if dialog.title.trim().is_empty() {
        "Dialog"
    } else {
        dialog.title.as_str()
    };

    let result = MessageDialog::new()
        .set_title(title)
        .set_description(description)
        .set_buttons(buttons)
        .set_level(level)
        .show();

    match (&dialog.modal, result) {
        (DialogModalType::Question { .. }, MessageDialogResult::Ok)
        | (DialogModalType::Question { .. }, MessageDialogResult::Yes) => DialogResult::Confirmed,
        (DialogModalType::Question { .. }, MessageDialogResult::Cancel)
        | (DialogModalType::Question { .. }, MessageDialogResult::No) => DialogResult::Cancelled,
        (DialogModalType::Failure { .. }, MessageDialogResult::Ok)
        | (DialogModalType::Failure { .. }, MessageDialogResult::Yes) => DialogResult::Confirmed,
        (_, MessageDialogResult::Cancel) | (_, MessageDialogResult::No) => DialogResult::Closed,
        _ => DialogResult::Closed,
    }
}

fn show_system_dialog(dialog: &DialogConfig) -> DialogResult {
    #[cfg(all(target_os = "linux", not(target_arch = "wasm32")))]
    {
        let dialog_owned = dialog.clone();
        let started = spawn_linux_system_dialog_task(move || {
            let _ = show_system_dialog_blocking(&dialog_owned);
        });
        return if started {
            DialogResult::Closed
        } else {
            DialogResult::Unavailable
        };
    }

    #[cfg(all(not(target_os = "linux"), not(target_arch = "wasm32")))]
    {
        return show_system_dialog_blocking(dialog);
    }

    #[cfg(target_arch = "wasm32")]
    {
        if matches!(dialog.modal, DialogModalType::Blank) {
            return DialogResult::Unavailable;
        }

        let title = if dialog.title.trim().is_empty() {
            "Dialog"
        } else {
            dialog.title.as_str()
        };

        let description = match &dialog.modal {
            DialogModalType::Default => dialog.content.clone(),
            DialogModalType::Failure { error_code, .. } => {
                format!("{}\nError code: {}", dialog.content, error_code)
            }
            DialogModalType::Question { .. } => dialog.content.clone(),
            DialogModalType::Blank => String::new(),
        };

        #[cfg(feature = "clipboard-wasm")]
        {
            return match &dialog.modal {
                DialogModalType::Question { .. } => show_wasm_confirm_dialog(title, &description),
                DialogModalType::Default | DialogModalType::Failure { .. } => {
                    show_wasm_alert_dialog(title, &description)
                }
                DialogModalType::Blank => DialogResult::Unavailable,
            };
        }

        #[cfg(not(feature = "clipboard-wasm"))]
        {
            warn!("System dialogs are not supported on wasm targets without `clipboard-wasm`.");
            let _ = (title, description);
            return DialogResult::Unavailable;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modal_kind_mapping_matches_modal_variants() {
        assert_eq!(DialogModalType::Default.kind(), DialogModalKind::Default);
        assert_eq!(
            DialogModalType::Failure {
                error_code: "E-1".to_string(),
                confirm_label: "Confirm".to_string()
            }
            .kind(),
            DialogModalKind::Failure
        );
        assert_eq!(
            DialogModalType::Question {
                confirm_label: "Yes".to_string(),
                cancel_label: "No".to_string()
            }
            .kind(),
            DialogModalKind::Question
        );
        assert_eq!(DialogModalType::Blank.kind(), DialogModalKind::Blank);
    }

    #[test]
    fn dialog_constructors_set_expected_defaults() {
        let default = DialogConfig::default_modal("Title", "Body");
        assert_eq!(default.modal.kind(), DialogModalKind::Default);
        assert_eq!(default.provider, DialogProvider::BevyApp);

        let failure = DialogConfig::failure("Error", "Something failed", "E-500");
        assert_eq!(failure.modal.kind(), DialogModalKind::Failure);
        assert!(!failure.close_on_backdrop);

        let question = DialogConfig::question("Question", "Proceed?");
        assert_eq!(question.modal.kind(), DialogModalKind::Question);
        assert!(question.close_on_backdrop);

        let blank = DialogConfig::blank(DialogLayout::BottomSheet);
        assert_eq!(blank.modal.kind(), DialogModalKind::Blank);
        assert_eq!(blank.layout, DialogLayout::BottomSheet);
    }
}
