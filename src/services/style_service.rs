use crate::ImageCache;
use crate::html::HtmlStyle;
use crate::services::image_service::get_or_load_image;
use crate::services::state_service::update_widget_states;
use crate::styles::{FontWeight, Style};
use crate::styles::components::UiStyle;
use crate::widgets::UIWidgetState;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;

pub struct StyleService;

impl Plugin for StyleService {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            update_widget_styles_system.after(update_widget_states),
        );
    }
}

pub fn update_widget_styles_system(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            Option<&UIWidgetState>,
            Option<&HtmlStyle>,
            &mut UiStyle,
        ),
        Or<(Changed<UiStyle>, Changed<HtmlStyle>, Changed<UIWidgetState>)>,
    >,
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
        Option<&mut ZIndex>,
        Option<&mut FocusPolicy>,
    )>,
    asset_server: Res<AssetServer>,
    mut image_cache: ResMut<ImageCache>,
    mut images: ResMut<Assets<Image>>,
) {
    for (entity, state_opt, html_style_opt, mut ui_style) in query.iter_mut() {
        let state = state_opt.cloned().unwrap_or_default();

        let mut base_styles: Vec<(&String, u32)> = vec![];
        let mut pseudo_styles: Vec<(&String, u32)> = vec![];

        for sel in ui_style.styles.keys() {
            if selector_matches_state(sel, &state) {
                let specificity = selector_specificity(sel);
                if sel.contains(':') {
                    pseudo_styles.push((sel, specificity));
                } else {
                    base_styles.push((sel, specificity));
                }
            }
        }

        base_styles.sort_by_key(|&(_, spec)| spec);
        pseudo_styles.sort_by_key(|&(_, spec)| spec);

        let mut final_style = Style::default();

        // 1) base normal
        for (sel, _) in &base_styles {
            if let Some(pair) = ui_style.styles.get(*sel) {
                final_style.merge(&pair.normal);
            }
        }

        // 2) base important
        for (sel, _) in &base_styles {
            if let Some(pair) = ui_style.styles.get(*sel) {
                final_style.merge(&pair.important);
            }
        }

        // 3) inline html
        if let Some(html_style) = html_style_opt {
            final_style.merge(&html_style.0);
        }

        // 4) pseudo normal
        for (sel, _) in &pseudo_styles {
            if let Some(pair) = ui_style.styles.get(*sel) {
                final_style.merge(&pair.normal);
            }
        }

        // 5) pseudo important
        for (sel, _) in &pseudo_styles {
            if let Some(pair) = ui_style.styles.get(*sel) {
                final_style.merge(&pair.important);
            }
        }

        if ui_style.active_style.as_ref() != Some(&final_style) {
            ui_style.active_style = Some(final_style.clone());
        }

        if let Ok((
                      node,
                      background,
                      border_color,
                      border_radius,
                      box_shadow,
                      text_color,
                      text_font,
                      text_layout,
                      image_node,
                      z_index,
                      focus_policy,
                  )) = style_query.get_mut(entity)
        {
            apply_style_to_node(&final_style, node);

            if let Some(mut bg) = background {
                bg.0 = final_style
                    .background
                    .clone()
                    .map(|b| b.color)
                    .unwrap_or(Color::NONE);
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
                bc.set_all(final_style.border_color.unwrap_or(Color::NONE));
            }

            if let Some(mut tc) = text_color {
                tc.0 = final_style.color.unwrap_or(Color::WHITE);
            }

            if let Some(mut image_node) = image_node {
                image_node.color = final_style.color.unwrap_or(Color::WHITE);
                if let Some(background) = final_style.background.clone() {
                    if let Some(path) = background.image {
                        let handle = get_or_load_image(
                            path.as_str(),
                            &mut image_cache,
                            &mut images,
                            &asset_server,
                        );
                        image_node.image = handle;
                    }
                }
            }

            if let Some(mut tf) = text_font {
                if let Some(font_size) = final_style.font_size.clone() {
                    tf.font_size = font_size.get(None);
                }

                if let Some(font_family) = final_style.font_family.clone() {
                    let font_path_str = font_family.0.to_string();
                    if font_path_str.eq_ignore_ascii_case("default") {
                        tf.font = Default::default();
                    } else if font_path_str.ends_with(".ttf") {
                        tf.font = asset_server.load(font_path_str);
                    } else {
                        let folder = font_path_str.trim().trim_matches('"').trim_matches('\'');

                        if folder.is_empty() {
                            tf.font = Default::default();
                        } else {
                            let weight_opt = final_style.font_weight.clone(); // Option<FontWeight>
                            tf.font = load_weighted_font_from_folder(&asset_server, folder, weight_opt);
                        }
                    }
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

            if let Some(mut fp) = focus_policy {
                let old = *fp;
                *fp = match final_style.pointer_events {
                    Some(pe) if pe == FocusPolicy::Pass => FocusPolicy::Pass, // pointer-events: none
                    Some(_) => FocusPolicy::Block,                           // pointer-events: auto
                    None => old,                                             // keep previous
                };
            } else {
                // If an entity doesn't have a FocusPolicy yet, insert one when relevant.
                if let Some(pe) = &final_style.pointer_events {
                    let fp = if *pe == FocusPolicy::Pass {
                        FocusPolicy::Pass
                    } else {
                        FocusPolicy::Block
                    };
                    commands.entity(entity).insert(fp);
                }
            }
        }
    }
}

/// Checks whether a CSS selector matches the widget's current UI state.
///
/// # Parameters
/// - `selector`: The full selector string, including pseudo-classes (e.g. `.btn:hover:focus`).
/// - `state`: The current UI state of the widget.
///
/// # Returns
/// - `true` if the state satisfies all pseudo-classes in the selector.
/// - `false` if any pseudo-class does not match.
///
/// # Supported Pseudo-Classes
/// - `:read-only`
/// - `:disabled`
/// - `:checked`
/// - `:focus`
/// - `:hover`
fn selector_matches_state(selector: &str, state: &UIWidgetState) -> bool {
    for part in selector.split_whitespace() {
        let segments: Vec<&str> = part.split(':').collect();
        for pseudo in &segments[1..] {
            match *pseudo {
                "read-only" if !state.readonly => return false,
                "disabled" if !state.disabled => return false,
                "checked" if state.disabled || !state.checked => return false,
                "focus" if state.disabled || !state.focused => return false,
                "hover" if state.disabled || !state.hovered => return false,
                _ => {}
            }
        }
    }
    true
}

/// Calculates the specificity of a CSS selector for sorting style precedence.
///
/// # Parameters
/// - `selector`: The full selector string (e.g. `#login:hover`, `.button`, `input:focus`).
///
/// # Returns
/// A numeric specificity score:
/// - ID selectors: +100
/// - Class selectors: +10
/// - Tag selectors: +1
/// - Pseudo-classes: +1 per occurrence
///
/// # Example
/// - `#main` => 100
/// - `.btn:focus` => 11
/// - `input` => 1
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

/// Applies the resolved [`Style`] to a Bevy [`Node`] UI component.
///
/// # Parameters
/// - `style`: The computed widget style.
/// - `node`: A mutable reference to the Bevy [`Node`] component (if it exists).
///
/// # Behavior
/// - Transfers values like width, height, margin, padding, flex/grid properties, etc.
/// - Handles direction-aware gap logic (column/row).
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
        node.flex_basis = style.flex_basis.unwrap_or_default();
        node.flex_shrink = style.flex_shrink.unwrap_or_default();
        node.flex_wrap = style.flex_wrap.unwrap_or_default();

        node.grid_row = style.grid_row.unwrap_or_default();
        node.grid_column = style.grid_column.unwrap_or_default();
        node.grid_auto_flow = style.grid_auto_flow.unwrap_or_default();
        node.grid_template_rows = style.grid_template_rows.clone().unwrap_or_default();
        node.grid_template_columns = style.grid_template_columns.clone().unwrap_or_default();
        node.grid_auto_columns = style.grid_auto_columns.clone().unwrap_or_default();
        node.grid_auto_rows = style.grid_auto_rows.clone().unwrap_or_default();
    }
}

fn load_weighted_font_from_folder(
    asset_server: &AssetServer,
    folder: &str,
    weight: Option<FontWeight>,
) -> Handle<Font> {
    let folder = folder.trim().trim_matches('"').trim_matches('\'').trim_end_matches('/');
    if folder.is_empty() {
        return Default::default();
    }

    let family = folder_basename(folder);

    let w = weight.unwrap_or(FontWeight::Normal);

    let token = weight_token_exact(w);
    let path_primary = format!("{folder}/{family}-{token}.ttf");

    asset_server.load::<Font>(path_primary)
}

fn weight_token_exact(weight: FontWeight) -> &'static str {
    match weight {
        FontWeight::Thin       => "Thin",
        FontWeight::ExtraLight => "ExtraLight",
        FontWeight::Light      => "Light",
        FontWeight::Normal     => "Regular",
        FontWeight::Medium     => "Medium",
        FontWeight::SemiBold   => "SemiBold",
        FontWeight::Bold       => "Bold",
        FontWeight::ExtraBold  => "ExtraBold",
        FontWeight::Black      => "Black",
    }
}

fn folder_basename(folder: &str) -> &str {
    folder.trim_end_matches('/').rsplit('/').next().unwrap_or(folder)
}
