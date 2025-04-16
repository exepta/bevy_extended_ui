use bevy::prelude::*;
use crate::global::{UiElementState, UiGenID};
use crate::styles::{BaseStyle, HoverStyle, PartialStyle, SelectedStyle, Style};

pub struct StyleService;

impl Plugin for StyleService {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, internal_ui_element_styling);
    }
}

fn internal_ui_element_styling(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &UiElementState,
        &BaseStyle,
        Option<&HoverStyle>,
        Option<&SelectedStyle>,
        &mut Node,
        &mut BorderRadius,
        &mut BorderColor,
        &mut BackgroundColor,
        Option<&mut ImageNode>
    ), With<UiGenID>>,
) {
    for (entity, state, base_style, hover_style, selected_style,
        mut node, mut border_radius, mut border_color,
        mut background_color, mut image_node)
    in query.iter_mut() {

        let mut current_style = base_style.0.clone();

        if state.selected {
            if let Some(focus) = selected_style {
                merge_styles(&mut current_style, &focus.0);
            }
        }

        if state.hovered {
            if let Some(hover) = hover_style {
                merge_styles(&mut current_style, &hover.0);
            }
        }

        apply_to_bevy_style(&mut commands, &entity, &current_style, &mut node, &mut border_radius,
                            &mut border_color, &mut background_color, &mut image_node);
    }
}

fn merge_styles(target: &mut Style, other: &PartialStyle) {
    if let Some(val) = other.width { target.width = val; }
    if let Some(val) = other.min_width { target.min_width = val; }
    if let Some(val) = other.max_width { target.max_width = val; }
    if let Some(val) = other.height { target.height = val; }
    if let Some(val) = other.min_height { target.min_height = val; }
    if let Some(val) = other.max_height { target.max_height = val; }
    if let Some(val) = other.top { target.top = val; }
    if let Some(val) = other.bottom { target.bottom = val; }
    if let Some(val) = other.left { target.left = val; }
    if let Some(val) = other.right { target.right = val; }
    if let Some(val) = other.padding { target.padding = val; }
    if let Some(val) = other.margin { target.margin = val; }
    if let Some(val) = other.align_content { target.align_content = val; }
    if let Some(val) = other.align_self { target.align_self = val; }
    if let Some(val) = other.align_items { target.align_items = val; }
    if let Some(val) = other.justify_content { target.justify_content = val; }
    if let Some(val) = other.justify_self { target.justify_self = val; }
    if let Some(val) = other.justify_items { target.justify_items = val; }
    if let Some(val) = other.display { target.display = val; }
    if let Some(val) = other.position_type { target.position_type = val; }
    if let Some(val) = other.border_radius.clone() {
        target.border_radius.top_left = val.top_left;
        target.border_radius.top_right = val.top_right;
        target.border_radius.bottom_left = val.bottom_left;
        target.border_radius.bottom_right = val.bottom_right;
    }
    if let Some(val) = other.border { target.border = val; }
    if let Some(val) = other.border_color { target.border_color = val; }
    if let Some(val) = other.background.clone() { target.background = val; }
    if let Some(val) = other.flex_grow { target.flex_grow = val; }
    if let Some(val) = other.flex_shrink { target.flex_shrink = val; }
    if let Some(val) = other.flex_direction { target.flex_direction = val; }
    if let Some(val) = other.flex_basis { target.flex_basis = val; }
    if let Some(val) = other.flex_wrap { target.flex_wrap = val; }
    if let Some(val) = other.gap_row { target.gap_row = val; }
    if let Some(val) = other.gap_column { target.gap_column = val; }
}

fn apply_to_bevy_style(commands: &mut Commands, entity: &Entity, from: &Style, to: &mut Node, to_border_radius: &mut BorderRadius,
                       to_border_color: &mut BorderColor, to_background_color: &mut BackgroundColor,
                       image_node: &mut Option<Mut<ImageNode>>
) {
    to.width = from.width;
    to.min_width = from.min_width;
    to.max_width = from.max_width;
    to.height = from.height;
    to.min_height = from.min_height;
    to.max_height = from.max_height;
    to.top = from.top;
    to.bottom = from.bottom;
    to.left = from.left;
    to.right = from.right;
    to.padding = from.padding;
    to.margin = from.margin;
    to.align_content = from.align_content;
    to.align_self = from.align_self;
    to.align_items = from.align_items;
    to.justify_content = from.justify_content;
    to.justify_self = from.justify_self;
    to.justify_items = from.justify_items;
    to.display = from.display;
    to.position_type = from.position_type;
    to_border_radius.top_left = from.border_radius.top_left;
    to_border_radius.top_right = from.border_radius.top_right;
    to_border_radius.bottom_right = from.border_radius.bottom_right;
    to_border_radius.bottom_left = from.border_radius.bottom_left;
    to.border = from.border;
    to_border_color.0 = from.border_color;
    to_background_color.0 = from.background.color;
    if let Some(image) = from.background.image.clone() {
        if let Some(image_node) = image_node {
            if !image_node.image.eq(&image) {}
            image_node.image = image;
        } else {
            commands.entity(*entity).insert(ImageNode::new(image));
        }
    }
    to.flex_grow = from.flex_grow;
    to.flex_shrink = from.flex_shrink;
    to.flex_direction = from.flex_direction;
    to.flex_basis = from.flex_basis;
    to.flex_wrap = from.flex_wrap;
    to.column_gap = from.gap_row;
    to.row_gap = from.gap_column;
}