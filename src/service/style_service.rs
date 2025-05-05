use bevy::prelude::*;
use crate::styling::convert::{CssBase, CssDisabled, CssFocus, CssHover, CssReadOnly};
use crate::styling::system::WidgetStyle;
use crate::{LastWidgetState, UIWidgetState};

pub struct StyleService;

impl Plugin for StyleService {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_widget_styles_system);
    }
}

fn update_widget_styles_system(
    mut commands: Commands,
    mut query: Query<(Entity, Option<&UIWidgetState>, Option<&LastWidgetState>,  Option<&mut WidgetStyle>,
                      Option<&CssBase>, Option<&CssHover>, Option<&CssDisabled>, Option<&CssReadOnly>, Option<&CssFocus>),
        With<WidgetStyle>>,
    mut style_query: Query<(Option<&mut Node>, Option<&mut BackgroundColor>, Option<&mut BorderColor>,
                            Option<&mut BorderRadius>, Option<&mut BoxShadow>, Option<&mut TextColor>,
                            Option<&mut TextFont>, Option<&mut TextLayout>),
        With<WidgetStyle>>,
) {
    for (entity, state, last_state, style, base, hover,
        disabled, readonly, focus)
    in query.iter_mut() {

        let current = state.cloned().unwrap_or_default();
        let last = last_state.cloned().unwrap_or_default();

        let needs_update = last_state.is_none() ||
            current.hovered != last.hovered ||
            current.disabled != last.disabled ||
            current.readonly != last.readonly ||
            current.focused != last.focused;
        
        if needs_update {

            commands.entity(entity).insert(LastWidgetState {
                hovered: current.hovered,
                disabled: current.disabled,
                readonly: current.readonly,
                focused: current.focused,
            });

            if let Ok((node, background, border_color,
                          border_radius, box_shadow,
                          text_color, text_font, text_layout))
                = style_query.get_mut(entity) {

                if let Some(widget_style) = style {
                    let mut merged_style = widget_style.clone();

                    if let Some(css_base) = base {
                        let base_styled = merged_style.load_selector(css_base.0.as_str());
                        merged_style.style.merge(&base_styled.style);
                    }

                    if let Some(state) = state {
                        if state.hovered {
                            if let Some(css_hover) = hover {
                                let hovered_style = merged_style.load_selector(css_hover.0.as_str());
                                merged_style.style.merge(&hovered_style.style);
                            }
                        }

                        if state.readonly {
                            if let Some(css_read_only) = readonly {
                                let read_only_style = merged_style.load_selector(css_read_only.0.as_str());
                                merged_style.style.merge(&read_only_style.style);
                            }
                        }

                        if state.disabled {
                            if let Some(css_disabled) = disabled {
                                let disabled_style = merged_style.load_selector(css_disabled.0.as_str());
                                merged_style.style.merge(&disabled_style.style);
                            }
                        }

                        if state.focused {
                            if let Some(css_focus) = focus {
                                let focused_style = merged_style.load_selector(css_focus.0.as_str());
                                merged_style.style.merge(&focused_style.style);
                            }
                        }
                    }

                    // Node
                    if let Some(mut node) = node {
                        node.width = if let Some(width) = merged_style.style.width { width } else { node.width };
                        node.min_width = if let Some(min_width) = merged_style.style.min_width { min_width } else { node.min_width };
                        node.max_width = if let Some(max_width) = merged_style.style.max_width { max_width } else { node.max_width };
                        node.height = if let Some(height) = merged_style.style.height { height } else { node.height };
                        node.min_height = if let Some(min_height) = merged_style.style.min_height { min_height } else { node.min_height };
                        node.max_height = if let Some(max_height) = merged_style.style.max_height { max_height } else { node.max_height };
                        node.display = if let Some(display) = merged_style.style.display { display } else { node.display };
                        node.position_type = if let Some(position) = merged_style.style.position_type { position } else { node.position_type };
                        node.left = if let Some(left) = merged_style.style.left { left } else { node.left };
                        node.top = if let Some(top) = merged_style.style.top { top } else { node.top };
                        node.right = if let Some(right) = merged_style.style.right { right } else { node.right };
                        node.bottom = if let Some(bottom) = merged_style.style.bottom { bottom } else { node.bottom };
                        node.padding = if let Some(padding) = merged_style.style.padding { padding } else { node.padding };
                        node.margin = if let Some(margin) = merged_style.style.margin { margin } else { node.margin };
                        node.border = if let Some(border) = merged_style.style.border { border } else { node.border };
                        node.justify_content = if let Some(just_con) = merged_style.style.justify_content { just_con } else { node.justify_content };
                        node.align_items = if let Some(align_items) = merged_style.style.align_items { align_items } else { node.align_items };
                        node.flex_direction = if let Some(flex_direction) = merged_style.style.flex_direction { flex_direction } else { node.flex_direction };
                        if node.flex_direction.eq(&FlexDirection::Row) || node.flex_direction.eq(&FlexDirection::RowReverse) {
                            node.column_gap = if let Some(gap) = merged_style.style.gap { gap } else { node.column_gap };
                        } else {
                            node.row_gap = if let Some(gap) = merged_style.style.gap { gap } else { node.row_gap };
                        }
                    }

                    // Background
                    if let Some(mut background) = background {
                        background.0 = if let Some(background) = merged_style.style.background.clone() { background.color } else { background.0 };
                    }

                    // Border Radius
                    if let Some(mut border_radius) = border_radius {
                        if let Some(radius) = merged_style.style.border_radius.clone() {
                            border_radius.top_left = radius.top_left;
                            border_radius.top_right = radius.top_right;
                            border_radius.bottom_left = radius.bottom_left;
                            border_radius.bottom_right = radius.bottom_right;
                        }
                    }

                    // Border Color
                    if let Some(mut border_color) = border_color {
                        border_color.0 = if let Some(color) = merged_style.style.border_color.clone() { color } else { border_color.0 };
                    }

                    // Text Color
                    if let Some(mut text_color) = text_color {
                        text_color.0 = if let Some(color) = merged_style.style.color { color } else { text_color.0 };
                    }

                    // Text Font
                    if let Some(mut text_font) = text_font {
                        text_font.font_size = if let Some(_) = merged_style.style.font_size.clone() { 15.0 } else { text_font.font_size };
                    }

                    // Box Shadow
                    if let Some(mut box_shadow) = box_shadow {
                        box_shadow.0 = if let Some(shadow) = merged_style.style.box_shadow.clone() { shadow.0 } else { box_shadow.0.clone() };
                    }
                }
            }
        }
    }

}