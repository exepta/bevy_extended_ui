use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use bevy::utils::HashMap;
use crate::global::{UiGenID, UiElementState};
use crate::styles::{BaseStyle, HoverStyle, SelectedStyle, InternalStyle};
use crate::resources::ExtendedUiConfiguration;

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(UiGenID, UiElementState, BaseStyle, HoverStyle, SelectedStyle, InternalStyle)]
pub struct InputField {
    pub text: String,
    pub placeholder_text: String,
    pub cap_text_at: InputCap,
    pub cursor_position: usize,
    pub input_type: InputType,
    pub clear_after_focus_lost: bool,
}

impl Default for InputField {
    fn default() -> Self {
        Self {
            text: String::from(""),
            placeholder_text: String::from(""),
            cap_text_at: InputCap::default(),
            input_type: InputType::default(),
            cursor_position: 0,
            clear_after_focus_lost: false,
        }
    }
}

impl InputField {
    pub fn new(text: &str, placeholder_text: &str, input_type: InputType) -> Self {
        Self {
            text: text.to_string(),
            placeholder_text: placeholder_text.to_string(),
            input_type,
            ..default()
        }
    }
}

#[derive(Reflect, Default, Debug, Clone)]
pub enum InputType {
    #[default]
    Text,
    Password,
    Number
}

#[derive(Reflect, Default, Debug, Clone)]
pub enum InputCap {
    #[default]
    NoCap,
    CapAtNodeSize,
    CapAt(usize), // 0 means nothing to print!
}

#[derive(Component)]
struct InputFieldRoot;

#[derive(Component)]
struct InputFieldText;

#[derive(Component)]
struct InputFieldIcon;

#[derive(Component)]
struct InputCursor;

#[derive(Resource, Default)]
struct KeyRepeatTimers {
    timers: HashMap<KeyCode, Timer>,
}

#[derive(Resource)]
pub struct CursorBlinkTimer {
    pub timer: Timer,
}

impl Default for CursorBlinkTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.95, TimerMode::Repeating)
        }
    }
}

pub struct InputWidget;

impl Plugin for InputWidget {
    fn build(&self, app: &mut App) {
        app.insert_resource(KeyRepeatTimers::default());
        app.insert_resource(CursorBlinkTimer::default());
        app.register_type::<InputField>();
        app.add_systems(Update, internal_generate_component_system);
    }
}

fn internal_generate_component_system(
    mut commands: Commands,
    query: Query<(Entity, &UiGenID, &InputField), (Without<InputFieldRoot>, With<InputField>)>,
    config: Res<ExtendedUiConfiguration>
) {
    let layer = config.render_layers.first().unwrap_or(&1);
    for (entity , gen_id, btn) in query.iter() {
        commands.entity(entity).insert((
            Name::new(format!("InputField-{}", gen_id.0)),
            Node::default(),

            RenderLayers::layer(*layer),
            InputFieldRoot,
        ));
    }
}