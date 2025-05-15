use bevy::prelude::*;
use crate::service::state_service::update_widget_states;
use crate::styling::convert::{CssClass, CssID, TagName};
use crate::styling::Style;
use crate::styling::system::WidgetStyle;
use crate::UIWidgetState;

pub struct StyleService;

impl Plugin for StyleService {
    fn build(&self, app: &mut App) {
        app.add_systems(Update,
            update_widget_styles_system.after(update_widget_states)
        );
    }
}

fn update_widget_styles_system(
    mut query: Query<(
        Entity,
        Option<&UIWidgetState>,
        &mut WidgetStyle,
        Option<&CssClass>,
        Option<&TagName>,
        Option<&CssID>,
    ), Or<(Changed<WidgetStyle>, Changed<UIWidgetState>, Changed<CssClass>, Changed<CssID>)>>,
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
    for (entity, state_opt, widget_style, class_opt, tag_opt, id_opt) in query.iter_mut() {
        let current = state_opt.cloned().unwrap_or_default();

        let mut selector_variants = vec![];

        // Parent Selector Variants (Tag, Class, ID)
        if let Some(tag) = tag_opt {
            selector_variants.push(tag.0.clone());
            if current.hovered {
                selector_variants.push(format!("{}:hover", tag.0));
            }
            if current.focused {
                selector_variants.push(format!("{}:focus", tag.0));
            }
            if current.readonly {
                selector_variants.push(format!("{}:read-only", tag.0));
            }
            if current.disabled {
                selector_variants.push(format!("{}:disabled", tag.0));
            }
        }

        if let Some(classes) = class_opt {
            for class in &classes.0 {
                selector_variants.push(format!(".{}", class.clone()));
                if current.hovered {
                    selector_variants.push(format!(".{}:hover", class));
                }
                if current.focused {
                    selector_variants.push(format!(".{}:focus", class));
                }
                if current.readonly {
                    selector_variants.push(format!(".{}:read-only", class));
                }
                if current.disabled {
                    selector_variants.push(format!(".{}:disabled", class));
                }
            }
        }

        if let Some(css_id) = id_opt {
            let id = format!("#{}", css_id.0);
            selector_variants.push(id.clone());
            if current.hovered {
                selector_variants.push(format!("{}:hover", id));
            }
            if current.focused {
                selector_variants.push(format!("{}:focus", id));
            }
            if current.readonly {
                selector_variants.push(format!("{}:read-only", id));
            }
            if current.disabled {
                selector_variants.push(format!("{}:disabled", id));
            }
        }

        // Child selectors like `.button-text`
        let mut final_style = Style::default();

        // Sort selectors to apply more specific ones first (ID > Class > Tag)
        selector_variants.sort_by_key(|sel| {
            if sel.starts_with('#') {
                3
            } else if sel.starts_with('.') {
                2
            } else {
                1
            }
        });

        // For each variant, check if the style exists and apply it
        for sel in selector_variants {
            if let Some(style) = widget_style.styles.get(&sel) {
                final_style.merge(style);
            }
        }

        // Apply final style to the node and child elements
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
            apply_style_to_node(&final_style, node);

            // Apply background
            if let Some(mut background) = background {
                background.0 = final_style.background.map(|b| b.color).unwrap_or(Color::NONE);
            }

            // Apply border radius
            if let Some(mut border_radius) = border_radius {
                if let Some(radius) = final_style.border_radius.clone() {
                    border_radius.top_left = radius.top_left;
                    border_radius.top_right = radius.top_right;
                    border_radius.bottom_left = radius.bottom_left;
                    border_radius.bottom_right = radius.bottom_right;
                } else {
                    border_radius.top_left = Val::ZERO;
                    border_radius.top_right = Val::ZERO;
                    border_radius.bottom_left = Val::ZERO;
                    border_radius.bottom_right = Val::ZERO;
                }
            }

            // Apply border color
            if let Some(mut border_color) = border_color {
                border_color.0 = final_style.border_color.unwrap_or(Color::NONE);
            }

            // Apply text color
            if let Some(mut text_color) = text_color {
                text_color.0 = final_style.color.unwrap_or(Color::WHITE);
            }

            // Apply text font size
            if let Some(mut text_font) = text_font {
                text_font.font_size = 15.0;
            }

            // Apply box shadow
            if let Some(mut box_shadow) = box_shadow {
                box_shadow.0 = final_style.box_shadow.map(|b| b.0.clone()).unwrap_or_default();
            }
        }
    }
}


fn apply_style_to_node(style: &Style, node: Option<Mut<Node>>) {
    if let Some(mut node) = node {
        node.width = style.width.unwrap_or_default();
        node.min_width = style.min_width.unwrap_or_default();
        node.max_width = style.max_width.unwrap_or_default();
        node.height = style.height.unwrap_or_default();
        node.min_height = style.min_height.unwrap_or_default();
        node.max_height = style.max_height.unwrap_or_default();
        node.display = style.display.unwrap_or_default();
        node.position_type = style.position_type.unwrap_or_default();
        node.left = style.left.unwrap_or_default();
        node.top = style.top.unwrap_or_default();
        node.right = style.right.unwrap_or_default();
        node.bottom = style.bottom.unwrap_or_default();
        node.padding = style.padding.unwrap_or_default();
        node.margin = style.margin.unwrap_or_default();
        node.border = style.border.unwrap_or_default();
        node.justify_content = style.justify_content.unwrap_or_default();
        node.align_items = style.align_items.unwrap_or_default();

        node.flex_direction = style.flex_direction.unwrap_or(FlexDirection::Row);
        match node.flex_direction {
            FlexDirection::Row | FlexDirection::RowReverse => {
                node.column_gap = style.gap.unwrap_or_default();
                node.row_gap = Val::Auto;
            }
            _ => {
                node.row_gap = style.gap.unwrap_or_default();
                node.column_gap = Val::Auto;
            }
        }
    }
}

