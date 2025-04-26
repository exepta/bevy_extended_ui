use bevy::prelude::*;
use crate::global::{BindToID, UiElementState, UiGenID};
use crate::styles::{LabelStyle, Style};
use crate::styles::state_styles::{Disabled, Hover, Selected, Styling};

pub fn resolve_style_by_state<'a>(
    base_style: &'a Styling,
    state: &UiElementState,
    hover: Option<&Hover>,
    selected: Option<&Selected>,
    disabled: Option<&Disabled>,
) -> Styling {
    if state.disabled {
        if let Some(Disabled(style)) = disabled {
            return style.clone();
        }
    }

    if state.selected {
        if let Some(Selected(style)) = selected {
            return style.clone();
        }
    }

    if state.hovered {
        if let Some(Hover(style)) = hover {
            return style.clone();
        }
    }

    base_style.clone()
}

pub fn apply_label_styles_to_child(
    child: Entity,
    ui_id: &UiGenID,
    label_style: &LabelStyle,
    label_query: &mut Query<(&BindToID, &mut TextColor, &mut TextFont, &mut TextLayout)>
) {
    if let Ok((bind_to, mut text_color, mut text_font, mut text_layout)) = label_query.get_mut(child) {
        if bind_to.0 != ui_id.0 {
            return;
        }

        apply_text_styles(label_style, &mut text_color, &mut text_font, &mut text_layout);
    }
}

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
    if let Some(apply_box_shadow) = style.box_shadow {
        box_shadow.color = apply_box_shadow.color;
        box_shadow.blur_radius = apply_box_shadow.blur_radius;
        box_shadow.spread_radius = apply_box_shadow.spread_radius;
        box_shadow.x_offset = apply_box_shadow.x_offset;
        box_shadow.y_offset = apply_box_shadow.y_offset;
    } else {
        box_shadow.color = Color::srgba(0.0, 0.0, 0.0, 0.0);
        box_shadow.blur_radius = Val::Px(0.);
        box_shadow.spread_radius = Val::Px(0.);
    }
}

fn apply_text_styles(
    style: &LabelStyle,
    text_color: &mut TextColor,
    text_font: &mut TextFont,
    text_layout: &mut TextLayout,
) {
    text_color.0 = style.color;
    text_font.font_size = style.font_size;
    text_font.font_smoothing = style.smoothing;
    text_layout.linebreak = style.line_break;
    text_layout.justify = style.justify;
}