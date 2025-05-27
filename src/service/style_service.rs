use bevy::prelude::*;
use crate::html::HtmlStyle;
use crate::service::state_service::update_widget_states;
use crate::styling::Style;
use crate::styling::system::WidgetStyle;
use crate::UIWidgetState;

pub struct StyleService;

impl Plugin for StyleService {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate,
            update_widget_styles_system
                .after(update_widget_states)
                .after(TransformSystem::TransformPropagate)
        );
    }
}

fn update_widget_styles_system(
    mut query: Query<(
        Entity,
        Option<&UIWidgetState>,
        Option<&HtmlStyle>,
        &mut WidgetStyle,
    ), Or<(Changed<WidgetStyle>, Changed<HtmlStyle>, Changed<UIWidgetState>)>>,
    mut style_query: Query<(
        Option<&mut Node>,
        Option<&mut BackgroundColor>,
        Option<&mut BorderColor>,
        Option<&mut BorderRadius>,
        Option<&mut BoxShadow>,
        Option<&mut TextColor>,
        Option<&mut TextFont>,
        Option<&mut TextLayout>,
        Option<&mut ImageNode>,
        Option<&mut ZIndex>
    )>,
) {
    for (entity, state_opt, html_style_opt, mut widget_style) in query.iter_mut() {
        let state = state_opt.cloned().unwrap_or_default();

        let mut base_styles: Vec<(&String, u32)> = vec![];
        let mut pseudo_styles: Vec<(&String, u32)> = vec![];
        
        for sel in widget_style.styles.keys() {
            if selector_matches_state(sel, &state) {
                let specificity = selector_specificity(sel);
                if sel.contains(":") {
                    pseudo_styles.push((sel, specificity));
                } else {
                    base_styles.push((sel, specificity));
                }
            }
        }
        
        base_styles.sort_by_key(|&(_, spec)| spec);
        pseudo_styles.sort_by_key(|&(_, spec)| spec);

        let mut final_style = Style::default();
        
        for (sel, _) in &base_styles {
            final_style.merge(&widget_style.styles[*sel]);
        }

        if let Some(html_style) = html_style_opt {
            final_style.merge(&html_style.0);
        }
        
        for (sel, _) in &pseudo_styles {
            final_style.merge(&widget_style.styles[*sel]);
        }

        if widget_style.active_style.as_ref() != Some(&final_style) {
            widget_style.active_style = Some(final_style.clone());
        }
        
        if let Ok((
                      node, background, border_color, border_radius, box_shadow,
                      text_color, text_font, text_layout, image_node, z_index
                  )) = style_query.get_mut(entity)
        {
            apply_style_to_node(&final_style, node);

            if let Some(mut bg) = background {
                bg.0 = final_style.background.map(|b| b.color).unwrap_or(Color::NONE);
            }

            if let Some(mut br) = border_radius {
                if let Some(radius) = final_style.border_radius.clone() {
                    br.top_left = radius.top_left;
                    br.top_right = radius.top_right;
                    br.bottom_left = radius.bottom_left;
                    br.bottom_right = radius.bottom_right;
                } else {
                    br.top_left = Val::ZERO;
                    br.top_right = Val::ZERO;
                    br.bottom_left = Val::ZERO;
                    br.bottom_right = Val::ZERO;
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

            if let Some(mut text_layout) = text_layout {
                if let Some(text_wrap) = final_style.text_wrap {
                    text_layout.linebreak = text_wrap;
                }
            }

            if let Some(mut bs) = box_shadow {
                bs.0 = final_style.box_shadow.unwrap_or_default().0;
            }

            if let Some(mut index) = z_index {
                index.0 = final_style.z_index.unwrap_or(0);
            }
        }
    }
}
fn selector_matches_state(selector: &str, state: &UIWidgetState) -> bool {
    for part in selector.split_whitespace() {
        let segments: Vec<&str> = part.split(':').collect();
        for pseudo in &segments[1..] {
            match *pseudo {
                "read-only" if !state.readonly => return false,
                "disabled" if !state.disabled => return false,
                "checked" if !state.checked => return false,
                "focus" if !state.focused => return false,
                "hover" if !state.hovered => return false,
                _ => {}
            }
        }
    }
    true
}

fn selector_specificity(selector: &str) -> u32 {
    let mut spec = 0;
    for part in selector.split_whitespace() {
        let segments: Vec<&str> = part.split(':').collect();
        let base = segments[0];

        spec += if base.starts_with('#') {
            100
        } else if base.starts_with('.') {
            10
        } else {
            1
        };

        spec += segments.len().saturating_sub(1) as u32;
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
        node.overflow = style.overflow.unwrap_or_default();

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
        
        
        node.flex_grow = style.flex_grow.unwrap_or_default();
    }
}

