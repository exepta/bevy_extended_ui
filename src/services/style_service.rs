use bevy::prelude::*;
use crate::global::{UiElementState, UiGenID};
use crate::styles::{BaseStyle, HoverStyle, InternalStyle, SelectedStyle, Style};

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
        &mut InternalStyle,
        Option<&BaseStyle>,
        Option<&HoverStyle>,
        Option<&SelectedStyle>,
        &mut Node,
        &mut BorderRadius,
        &mut BorderColor,
        &mut BackgroundColor,
        Option<&mut ImageNode>,
        Option<&Children>,
    ), With<UiGenID>>,
    mut text_query: Query<(Option<&mut TextColor>, Option<&mut TextFont>, Option<&mut TextLayout>, Option<&mut ImageNode>), Without<UiGenID>>,
    children_query: Query<&Children>,
) {
    for (entity, state, mut internal_style, base_style, hover_style, selected_style,
        mut node, mut border_radius, mut border_color,
        mut background_color, mut image_node,
        children)
    in query.iter_mut() {

        if let Some(base) = base_style {
            internal_style.merge_styles(&base.0);
        }

        if state.hovered {
            if let Some(hover) = hover_style {
                internal_style.merge_styles(&hover.0);
            }
        }


        if state.selected {
            if let Some(focus) = selected_style {
                internal_style.merge_styles(&focus.0);
            }
        }

        apply_to_bevy_style(&mut commands, &entity, &internal_style.0, &mut node, &mut border_radius,
                            &mut border_color, &mut background_color, &mut image_node,
                            children, &mut text_query, &children_query);
    }
}

fn apply_to_bevy_style(
    commands: &mut Commands,
    entity: &Entity,
    from: &Style,
    to: &mut Node,
    to_border_radius: &mut BorderRadius,
    to_border_color: &mut BorderColor,
    to_background_color: &mut BackgroundColor,
    image_node: &mut Option<Mut<ImageNode>>,
    children: Option<&Children>,
    text_query: &mut Query<(Option<&mut TextColor>, Option<&mut TextFont>, Option<&mut TextLayout>, Option<&mut ImageNode>), Without<UiGenID>>,
    children_query: &Query<&Children>,
) {
    // Apply layout-related style
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
    to.flex_grow = from.flex_grow;
    to.flex_shrink = from.flex_shrink;
    to.flex_direction = from.flex_direction;
    to.flex_basis = from.flex_basis;
    to.flex_wrap = from.flex_wrap;
    to.column_gap = from.gap_row;
    to.row_gap = from.gap_column;

    // Apply border and background
    to_border_radius.top_left = from.border_radius.top_left;
    to_border_radius.top_right = from.border_radius.top_right;
    to_border_radius.bottom_right = from.border_radius.bottom_right;
    to_border_radius.bottom_left = from.border_radius.bottom_left;
    to.border = from.border;
    to_border_color.0 = from.border_color;
    to_background_color.0 = from.background.color;

    if let Some(image) = from.background.image.clone() {
        if let Some(image_node) = image_node {
            if !image_node.image.eq(&image) {
                image_node.image = image;
            }
        } else {
            commands.entity(*entity).insert(ImageNode::new(image));
        }
    }

    // Apply text/image styles recursively to children
    if let Some(children) = children {
        for &child in children.iter() {
            apply_style_recursively(child, from, text_query, children_query);
        }
    }
}

fn apply_style_recursively(
    entity: Entity,
    from: &Style,
    text_query: &mut Query<(Option<&mut TextColor>, Option<&mut TextFont>, Option<&mut TextLayout>, Option<&mut ImageNode>), Without<UiGenID>>,
    children_query: &Query<&Children>,
) {
    if let Ok((text_color, text_font, text_layout, image_node)) = text_query.get_mut(entity) {
        if let Some(mut text_color) = text_color {
            text_color.0 = from.color;
        }
        if let Some(mut text_font) = text_font {
            text_font.font = from.font.clone();
            text_font.font_size = from.font_size;
        }
        if let Some(mut text_layout) = text_layout {
            text_layout.linebreak = from.line_break;
        }
        if let Some(mut image_node) = image_node {
            image_node.color = from.color;
        }
    }

    if let Ok(children) = children_query.get(entity) {
        for &child in children.iter() {
            apply_style_recursively(child, from, text_query, children_query);
        }
    }
}