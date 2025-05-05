use bevy::prelude::*;
use crate::styling::system::WidgetStyle;
use crate::UIWidgetState;

pub struct StyleService;

impl Plugin for StyleService {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_widget_styles_system);
    }
}

fn update_widget_styles_system(
    mut query: Query<(
        Entity,
        Option<&UIWidgetState>,
        &mut WidgetStyle,
    ), Or<(Changed<WidgetStyle>, Changed<UIWidgetState>)>>,
    mut style_query: Query<(
        Option<&mut Node>,
        Option<&mut BackgroundColor>,
        Option<&mut BorderColor>,
        Option<&mut BorderRadius>,
        Option<&mut BoxShadow>,
        Option<&mut TextColor>,
        Option<&mut TextFont>,
        Option<&mut TextLayout>,
    )>,
) {
    for (entity, state_opt, widget_style) in query.iter_mut() {
        let current = state_opt.cloned().unwrap_or_default();

            if let Ok((
                          node,
                          background,
                          border_color,
                          border_radius,
                          box_shadow,
                          text_color,
                          text_font,
                          _text_layout,
                      )) = style_query.get_mut(entity)
            {
                let base_selector = widget_style.css_selector.clone();

                // Build active selector string
                let mut selector = base_selector.clone();
                if current.hovered {
                    selector.push_str(":hover");
                }
                if current.focused {
                    selector.push_str(":focus");
                }
                if current.readonly {
                    selector.push_str(":read-only");
                }
                if current.disabled {
                    selector.push_str(":disabled");
                }

                // Start with base style
                let mut final_style = widget_style
                    .styles
                    .get(&base_selector)
                    .cloned()
                    .unwrap_or_default();

                // Then merge in state-specific style
                if let Some(state_style) = widget_style.styles.get(&selector) {
                    final_style.merge(state_style);
                }

                // Apply style to UI components
                if let Some(mut node) = node {
                    if let Some(v) = final_style.width {
                        node.width = v;
                    }
                    if let Some(v) = final_style.min_width {
                        node.min_width = v;
                    }
                    if let Some(v) = final_style.max_width {
                        node.max_width = v;
                    }
                    if let Some(v) = final_style.height {
                        node.height = v;
                    }
                    if let Some(v) = final_style.min_height {
                        node.min_height = v;
                    }
                    if let Some(v) = final_style.max_height {
                        node.max_height = v;
                    }
                    if let Some(v) = final_style.display {
                        node.display = v;
                    }
                    if let Some(v) = final_style.position_type {
                        node.position_type = v;
                    }
                    if let Some(v) = final_style.left {
                        node.left = v;
                    }
                    if let Some(v) = final_style.top {
                        node.top = v;
                    }
                    if let Some(v) = final_style.right {
                        node.right = v;
                    }
                    if let Some(v) = final_style.bottom {
                        node.bottom = v;
                    }
                    if let Some(v) = final_style.padding {
                        node.padding = v;
                    }
                    if let Some(v) = final_style.margin {
                        node.margin = v;
                    }
                    if let Some(v) = final_style.border {
                        node.border = v;
                    }
                    if let Some(v) = final_style.justify_content {
                        node.justify_content = v;
                    }
                    if let Some(v) = final_style.align_items {
                        node.align_items = v;
                    }
                    if let Some(v) = final_style.flex_direction {
                        node.flex_direction = v;
                        match v {
                            FlexDirection::Row | FlexDirection::RowReverse => {
                                if let Some(gap) = final_style.gap {
                                    node.column_gap = gap;
                                }
                            }
                            _ => {
                                if let Some(gap) = final_style.gap {
                                    node.row_gap = gap;
                                }
                            }
                        }
                    }
                }

                if let Some(mut background) = background {
                    if let Some(bg) = final_style.background.clone() {
                        background.0 = bg.color;
                    }
                }

                if let Some(mut border_radius) = border_radius {
                    if let Some(radius) = final_style.border_radius.clone() {
                        border_radius.top_left = radius.top_left;
                        border_radius.top_right = radius.top_right;
                        border_radius.bottom_left = radius.bottom_left;
                        border_radius.bottom_right = radius.bottom_right;
                    }
                }

                if let Some(mut border_color) = border_color {
                    if let Some(color) = final_style.border_color.clone() {
                        border_color.0 = color;
                    }
                }

                if let Some(mut text_color) = text_color {
                    if let Some(color) = final_style.color {
                        text_color.0 = color;
                    }
                }

                if let Some(mut text_font) = text_font {
                    if final_style.font_size.is_some() {
                        text_font.font_size = 15.0;
                    }
                }

                if let Some(mut box_shadow) = box_shadow {
                    if let Some(shadow) = final_style.box_shadow.clone() {
                        box_shadow.0 = shadow.0;
                    }
                }

                info!("update");
            }
        }
}
