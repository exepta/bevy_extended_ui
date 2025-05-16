use bevy::prelude::*;
use crate::service::state_service::update_widget_states;
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
        &WidgetStyle,
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
        Option<&mut ImageNode>
    )>,
) {
    for (entity, state_opt, widget_style) in query.iter_mut() {
        let state = state_opt.cloned().unwrap_or_default();
        
        let mut applicable: Vec<(&String, u32)> = widget_style
            .styles
            .keys()
            .filter_map(|sel| {
                if selector_matches_state(sel, &state) {
                    Some((sel, selector_specificity(sel)))
                } else {
                    None
                }
            })
            .collect();
        
        applicable.sort_by_key(|&(_, spec)| spec);
        
        let mut final_style = Style::default();
        for (sel, _) in applicable {
            final_style.merge(&widget_style.styles[sel]);
        }
        
        if let Ok((node, background, border_color, 
                      border_radius, box_shadow, text_color, 
                      text_font, _, image_node)) =
            style_query.get_mut(entity)
        {
            apply_style_to_node(&final_style, node);
            if let Some(mut bg) = background {
                bg.0 = final_style.background.map(|b| b.color).unwrap_or(Color::NONE);
            }

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
            if let Some(mut bc) = border_color {
                bc.0 = final_style.border_color.unwrap_or(Color::NONE);
            }
            if let Some(mut tc) = text_color {
                tc.0 = final_style.color.unwrap_or(Color::WHITE);
            }
            
            if let Some(mut image_node) = image_node {
                image_node.color = final_style.color.unwrap_or(Color::WHITE);
            }
            
            if let Some(mut tf) = text_font {
                if let Some(font_size) = final_style.font_size.clone() {
                    tf.font_size = font_size.get(None);
                }
            }
            if let Some(mut bs) = box_shadow {
                bs.0 = final_style.box_shadow.unwrap_or_default().0;
            }
        }
    }
}

fn selector_matches_state(selector: &str, state: &UIWidgetState) -> bool {
    let first = selector.split_whitespace().next().unwrap_or(selector);
    let parts: Vec<&str> = first.split(':').collect();
    for pseudo in &parts[1..] {
        match *pseudo {
            "read-only" if !state.readonly => return false,
            "disabled" if !state.disabled => return false,
            "checked" if !state.checked => return false,
            "focus" if !state.focused => return false,
            "hover" if !state.hovered => return false,
            _ => {}
        }
    }
    true
}

fn selector_specificity(selector: &str) -> u32 {
    let mut spec = 0;
    for part in selector.split_whitespace() {
        let secs: Vec<&str> = part.split(':').collect();
        // Basis
        let base = secs[0];
        spec += if base.starts_with('#') {
            100
        } else if base.starts_with('.') {
            10
        } else {
            1
        };
        spec += (secs.len() as u32).saturating_sub(1);
    }
    spec
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

