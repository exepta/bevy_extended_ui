use crate::ImageCache;
use crate::styles::{CssClass, CssSource, IconPlace};
use crate::widgets::controls::button::ButtonWidget;
use crate::widgets::controls::check_box::CheckBoxWidget;
use crate::widgets::controls::choice_box::ChoiceBoxWidget;
use crate::widgets::controls::fieldset::FieldSetWidget;
use crate::widgets::controls::input::InputWidget;
use crate::widgets::controls::progress_bar::ProgressBarWidget;
use crate::widgets::controls::radio_button::RadioButtonWidget;
use crate::widgets::controls::scroll_bar::ScrollWidget;
use crate::widgets::controls::slider::SliderWidget;
use crate::widgets::controls::switch_button::SwitchButtonWidget;
use crate::widgets::controls::toggle_button::ToggleButtonWidget;
use crate::widgets::{BindToID, UIWidgetState};
use bevy::camera::visibility::RenderLayers;
use bevy::ecs::relationship::RelatedSpawnerCommands;
use bevy::prelude::*;

pub mod button;
pub mod check_box;
pub mod choice_box;
pub mod fieldset;
pub mod input;
mod progress_bar;
pub mod radio_button;
mod scroll_bar;
pub mod slider;
pub mod switch_button;
pub mod toggle_button;

/// Marker component for spawned button icon images.
#[derive(Component)]
pub struct ButtonImage;

/// Plugin that registers control widgets.
pub struct ExtendedControlWidgets;

impl Plugin for ExtendedControlWidgets {
    /// Adds all control widget plugins.
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ButtonWidget,
            CheckBoxWidget,
            ChoiceBoxWidget,
            FieldSetWidget,
            InputWidget,
            ProgressBarWidget,
            RadioButtonWidget,
            ScrollWidget,
            SliderWidget,
            SwitchButtonWidget,
            ToggleButtonWidget,
        ));
    }
}

/// Spawns an icon when the desired placement matches.
pub fn place_icon_if(
    builder: &mut RelatedSpawnerCommands<ChildOf>,
    icon_place: IconPlace,
    desired_place: IconPlace,
    icon_path: &Option<String>,
    entry: usize,
    asset_server: &Res<AssetServer>,
    image_cache: &mut ResMut<ImageCache>,
    css_class: Vec<String>,
    id: usize,
    layer: usize,
    css_source: CssSource,
) {
    if icon_place == desired_place {
        place_icon(
            builder,
            icon_path,
            entry,
            asset_server,
            image_cache,
            css_class,
            id,
            layer,
            css_source,
        );
    }
}

/// Spawns the icon image node for a control widget.
fn place_icon(
    builder: &mut RelatedSpawnerCommands<ChildOf>,
    icon_path: &Option<String>,
    entry: usize,
    asset_server: &Res<AssetServer>,
    image_cache: &mut ResMut<ImageCache>,
    css_class: Vec<String>,
    id: usize,
    layer: usize,
    css_source: CssSource,
) {
    if let Some(icon) = icon_path.clone() {
        let owned_icon = icon.to_string();
        let handle = image_cache
            .map
            .entry(icon.clone())
            .or_insert_with(|| asset_server.load(owned_icon.clone()))
            .clone();

        builder.spawn((
            Name::new(format!("Button-Icon-{}", entry)),
            ImageNode::new(handle),
            RenderLayers::layer(layer),
            Pickable::IGNORE,
            UIWidgetState::default(),
            css_source.clone(),
            CssClass(css_class),
            BindToID(id),
            ZIndex(1),
            ButtonImage,
        ));
    }
}
