use bevy::prelude::*;
use crate::styles::Style;

pub fn apply_base_component_style(style: &Style, node: &mut Node) {
    node.width = style.width;
    node.min_width = style.min_width;
    node.max_width = style.max_width;
    node.height = style.height;
    node.min_height = style.min_height;
    node.max_height = style.max_height;
    node.top = style.top;
    node.left = style.left;
    node.right = style.right;
    node.bottom = style.bottom;
    node.padding = style.padding;
    node.margin = style.margin;
    node.border = style.border;
    node.position_type = style.position_type;
    node.display = style.display;
    node.flex_direction = style.flex_direction;
    node.flex_shrink = style.flex_shrink;
    node.flex_grow = style.flex_grow;
    node.flex_basis = style.flex_basis;
    node.column_gap = style.gap_row;
    node.row_gap = style.gap_column;
    node.flex_wrap = style.flex_wrap;
    node.align_items = style.align_items;
    node.align_content = style.align_content;
    node.align_self = style.align_self;
    node.justify_content = style.justify_content;
    node.justify_self = style.justify_self;
    node.justify_items = style.justify_items;
}

pub fn apply_design_styles(
    style: &Style,
    background_color: &mut BackgroundColor,
    border_color: &mut BorderColor,
    border_radius: &mut BorderRadius,
    box_shadow: &mut BoxShadow,
) {
    background_color.0 = style.background.color;
    border_color.0 = style.border_color;
    border_radius.top_left = style.border_radius.top_left;
    border_radius.top_right = style.border_radius.top_right;
    border_radius.bottom_left = style.border_radius.bottom_left;
    border_radius.bottom_right = style.border_radius.bottom_right;
    box_shadow.color = style.box_shadow.color;
    box_shadow.blur_radius = style.box_shadow.blur_radius;
    box_shadow.spread_radius = style.box_shadow.spread_radius;
    box_shadow.x_offset = style.box_shadow.x_offset;
    box_shadow.y_offset = style.box_shadow.y_offset;
}