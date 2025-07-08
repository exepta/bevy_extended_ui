use bevy::ecs::relationship::RelatedSpawnerCommands;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use crate::styling::convert::{CssClass, CssSource, TagName};
use crate::{BindToID, CurrentWidgetState, ExtendedUiConfiguration, IgnoreParentState, ImageCache, UIGenID, UIWidgetState};
use crate::service::image_cache_service::{get_or_load_image, DEFAULT_CHOICE_BOX_KEY};
use crate::styling::FontVal;
use crate::styling::paint::Colored;
use crate::styling::system::WidgetStyle;
use crate::widgets::{ChoiceBox, ChoiceOption, WidgetId, WidgetKind};

#[derive(Component)]
struct ChoiceBase;

#[derive(Component)]
struct ChoiceOptionBase;

#[derive(Component)]
struct SelectedOptionBase;

#[derive(Component)]
struct DropBase;

#[derive(Component)]
struct OverlayLabel;

#[derive(Component)]
struct ChoiceLayoutBoxBase;

pub struct ChoiceBoxWidget;

impl Plugin for ChoiceBoxWidget {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            update_content_box_visibility,
            internal_node_creation_system,
            handle_scroll_events,
            handle_overlay_label
        ).chain());
    }
}

/// System that initializes internal UI nodes for [`ChoiceBox`] components (select/dropdown widgets).
///
/// This system constructs the full visual structure of a dropdown menu, including
/// - Main select box
/// - Overlay label
/// - Currently selected option node (icon + text)
/// - Dropdown content container with all selectable options
///
/// Each `ChoiceBox` entity that hasn't yet been initialized (i.e., is missing [`ChoiceBase`])
/// gets the following structure:
///
/// ```text
/// ChoiceBox (Tag: <select>)
/// ├── SelectLabel (optional static label)
/// ├── SelectedOption (currently selected visual representation)
/// └── ContentBox (dropdown menu, hidden by default)
///     ├── Option 1 (with optional icon and text)
///     ├── Option 2 ...
///     └── ...
/// ```
///
/// # Behavior
/// - Observers are attached to the main box and each option to respond to hover and click.
/// - Styling is inherited from [`CssSource`] and modified using classes like `choice-content-box`, `choice-option`, etc.
/// - The dropdown menu is hidden by default and must be revealed through UI interaction.
///
/// # Parameters
/// - `commands`: Used to build entities and assign components.
/// - `query`: Selects uninitialized `ChoiceBox` widgets.
/// - `config`: UI configuration including rendering layers.
/// - `asset_server`: Loads icon images if used in options.
/// - `image_cache`: Prevents redundant image loading.
/// - `images`: Asset container for Bevy `Image`s.
fn internal_node_creation_system(
    mut commands: Commands,
    query: Query<(Entity, &UIGenID, &ChoiceBox, Option<&CssSource>), (With<ChoiceBox>, Without<ChoiceBase>)>,
    config: Res<ExtendedUiConfiguration>,
    asset_server: Res<AssetServer>,
    mut image_cache: ResMut<ImageCache>,
    mut images: ResMut<Assets<Image>>,
) {
    let layer = config.render_layers.first().unwrap_or(&1);
    for (entity, id, choice_box, source_opt) in query.iter() {
        let mut css_source = CssSource::default();
        if let Some(source) = source_opt {
            css_source = source.clone();
        }

        commands.entity(entity).insert((
            Name::new(format!("Choice-Box-{}", choice_box.w_count)),
            Node::default(),
            WidgetId {
                id: choice_box.w_count,
                kind: WidgetKind::ChoiceBox
            },
            BackgroundColor::default(),
            ImageNode::default(),
            BorderColor::default(),
            BorderRadius::default(),
            BoxShadow::new(Colored::TRANSPARENT, Val::Px(0.), Val::Px(0.), Val::Px(0.), Val::Px(0.)),
            ZIndex::default(),
            Pickable::default(),
            css_source.clone(),
            TagName("select".to_string()),
            RenderLayers::layer(*layer),
            ChoiceBase
        )).observe(on_internal_click)
            .observe(on_internal_cursor_entered)
            .observe(on_internal_cursor_leave)
            .with_children(|builder| {

                // Overlay label
                builder.spawn((
                    Name::new(format!("Select-Label-{}", choice_box.w_count)),
                    Node::default(),
                    Text::new(choice_box.label.clone()),
                    TextColor::default(),
                    TextLayout::default(),
                    TextFont::default(),
                    ZIndex::default(),
                    UIWidgetState::default(),
                    css_source.clone(),
                    CssClass(vec!["select-label".to_string()]),
                    Pickable::IGNORE,
                    RenderLayers::layer(*layer),
                    OverlayLabel,
                    BindToID(id.0)
                ));
                
                generate_child_selected_option(builder, &css_source.clone(), choice_box, layer, &id.0, &mut *image_cache, &mut images, &asset_server);
                
                builder.spawn((
                    Name::new(format!("Choice-Content-{}", choice_box.w_count)),
                    Node::default(),
                    BackgroundColor::default(),
                    ImageNode::default(),
                    BorderColor::default(),
                    BorderRadius::default(),
                    BoxShadow::new(Colored::TRANSPARENT, Val::Px(0.), Val::Px(0.), Val::Px(0.), Val::Px(0.)),
                    ZIndex::default(),
                    UIWidgetState::default(),
                    css_source.clone(),
                    CssClass(vec![String::from("choice-content-box")]),
                    RenderLayers::layer(*layer),
                    Visibility::Hidden,
                    ChoiceLayoutBoxBase,
                    BindToID(id.0)
                )).with_children(|builder| {
                    for option in choice_box.options.iter() {
                        
                        let state;
                        if choice_box.value.internal_value.eq(&option.internal_value) {
                            state = UIWidgetState {
                                checked: true,
                                ..default()
                            }
                        } else {
                            state = UIWidgetState::default();
                        }
                        
                        builder.spawn((
                            Name::new(format!("Option-{}", choice_box.w_count)),
                            Node::default(),
                            BackgroundColor::default(),
                            ImageNode::default(),
                            BorderColor::default(),
                            BorderRadius::default(),
                            ZIndex::default(),
                            state.clone(),
                            IgnoreParentState,
                            option.clone(),
                            css_source.clone(),
                            CssClass(vec![String::from("choice-option")]),
                            RenderLayers::layer(*layer),
                            ChoiceOptionBase,
                            BindToID(id.0)
                        )).observe(on_internal_option_click)
                            .observe(on_internal_option_cursor_entered)
                            .observe(on_internal_option_cursor_leave)
                            .with_children(|builder| {
                                
                                if let Some(icon_path) = option.icon_path.as_deref() {
                                    let handle = get_or_load_image(
                                        icon_path,
                                        &mut image_cache,
                                        &mut images,
                                        &asset_server,
                                    );

                                    builder.spawn((
                                        Name::new(format!("Option-Icon-{}", choice_box.w_count)),
                                        ImageNode {
                                            image: handle,
                                            ..default()
                                        },
                                        ZIndex::default(),
                                        state.clone(),
                                        IgnoreParentState,
                                        css_source.clone(),
                                        CssClass(vec![String::from("option-icon"), String::from("option-text")]),
                                        Pickable::IGNORE,
                                        RenderLayers::layer(*layer),
                                        BindToID(id.0)
                                    ));
                                }
                                
                                let text;
                                if option.text.trim().is_empty() {
                                    text = Text::new("Select...");
                                } else {
                                    text = Text::new(option.text.clone());
                                }
                                
                                builder.spawn((
                                    Name::new(format!("Option-Text-{}", choice_box.w_count)),
                                    text,
                                    TextColor::default(),
                                    TextFont::default(),
                                    TextLayout::default(),
                                    ZIndex::default(),
                                    state.clone(),
                                    IgnoreParentState,
                                    css_source.clone(),
                                    CssClass(vec![String::from("option-text")]),
                                    Pickable::IGNORE,
                                    RenderLayers::layer(*layer),
                                    BindToID(id.0)
                                ));
                        });
                    }
                });

        });
    }
}

// ===============================================
//             Intern Functions
// ===============================================

/// Adjusts the position and size of the overlay label in a [`ChoiceBox`] widget.
///
/// This system runs for all `ChoiceBox` components and finds their associated
/// overlay labels (nodes marked with [`OverlayLabel`] and [`BindToID`]).
///
/// - If the `ChoiceBox` is focused or has a selected value/icon, the label moves up
///   and shrinks (floating label style).
/// - Otherwise, the label remains in its original position (centered).
///
/// This system synchronizes both the `Node::top` and the corresponding [`WidgetStyle`] values
/// for runtime CSS-based animation consistency.
///
/// # Affects:
/// - `Node::top`
/// - `TextFont::font_size`
/// - `WidgetStyle::top`
/// - `WidgetStyle::font_size`
fn handle_overlay_label(
    query: Query<(&UIWidgetState, &UIGenID, &ChoiceBox, &Children), With<ChoiceBox>>,
    mut label_query: Query<(&BindToID, &mut Node, &mut TextFont, &mut WidgetStyle), With<OverlayLabel>>,
) {
    for (state, gen_id, choice_box, children) in query.iter() {
        for child in children.iter() {
            if let Ok((bind_to, mut node, mut text_font, mut styles)) = label_query.get_mut(child) {
                if bind_to.0 != gen_id.0 {
                    continue;
                }

                if state.focused {
                    node.top = Val::Px(5.);
                    text_font.font_size = 10.;
                } else {
                    if choice_box.value.text.is_empty() && choice_box.value.icon_path.is_none() {
                        node.top = Val::Px(19.5);
                        text_font.font_size = 14.;
                    } else {
                        node.top = Val::Px(5.);
                        text_font.font_size = 10.;
                    }
                }

                for (_, style) in styles.styles.iter_mut() {
                    style.top = Some(node.top);
                    style.font_size = Some(FontVal::Px(text_font.font_size));
                }
            }
        }
    }
}

/// Controls the visibility of the dropdown menu (`ChoiceLayoutBoxBase`) for each [`ChoiceBox`].
///
/// If a `ChoiceBox` loses focus, its menu is automatically closed (`open = false`).
/// This system synchronizes the `UIWidgetState::open` field with the actual visibility
/// of the child content box.
///
/// # Behavior:
/// - Sets `Visibility::Visible` if `UIWidgetState::open` is true.
/// - Otherwise, hides the dropdown menu.
///
/// # Affects:
/// - `UIWidgetState::open`
/// - `Visibility`
fn update_content_box_visibility(
    mut query: Query<(&mut UIWidgetState, &UIGenID), (With<ChoiceBox>, Changed<UIWidgetState>)>,
    mut content_query: Query<(&mut Visibility, &BindToID), With<ChoiceLayoutBoxBase>>,
) {
    for (mut state, gen_id) in query.iter_mut() {
        for (mut visibility, bind_to_id) in content_query.iter_mut() {
            if bind_to_id.0 != gen_id.0 {
                continue;
            }

            if !state.focused {
                state.open = false;
            }

            if state.open {
                *visibility = Visibility::Inherited;
            } else {
                *visibility = Visibility::Hidden;
            }
        }
    }
}

/// Enables mouse wheel scrolling for the options inside a [`ChoiceBox`] dropdown.
///
/// This system reads [`MouseWheel`] input and scrolls the child options inside
/// the visible [`ChoiceLayoutBoxBase`]. It applies clamped and smoothed scrolling
/// to maintain a fluid UI experience.
///
/// # Logic:
/// - Only applies scrolling to dropdowns that are currently visible.
/// - Computes scroll offset per frame using an exponential smoothing function.
/// - Applies the offset to each `Node::top` and updates the matching [`WidgetStyle`] styles.
///
/// # Assumptions:
/// - A visible dropdown has at least 3 items; otherwise, scrolling isn't necessary.
/// - Each option is placed vertically with a fixed height (e.g., 50px).
///
/// # Affects:
/// - `Node::top` on [`ChoiceOptionBase`] children
/// - `WidgetStyle::top`
fn handle_scroll_events(
    mut scroll_events: EventReader<MouseWheel>,
    mut layout_query: Query<(Entity, &Visibility, &Children), With<ChoiceLayoutBoxBase>>,
    mut option_query: Query<(&mut Node, &mut WidgetStyle, &ChildOf), With<ChoiceOptionBase>>,
    time: Res<Time>,
) {
    let mut max_scroll = -0.0;
    let min_scroll = 0.0;

    let smooth_factor = 30.;

    for event in scroll_events.read() {
        for (layout_entity, visibility, children) in layout_query.iter_mut() {
            if *visibility != Visibility::Visible || *visibility != Visibility::Inherited {
                continue;
            }

            if children.len() > 3 {
                max_scroll = -50.0 * (children.len() - 3) as f32;
            }

            let scroll_amount = match event.unit {
                MouseScrollUnit::Line => event.y * 25.0,
                MouseScrollUnit::Pixel => event.y,
            };

            let inverted_scroll_amount = scroll_amount;

            for (mut style, mut widget_style, parent) in option_query.iter_mut() {
                if parent.parent() != layout_entity {
                    continue;
                }

                let current_offset = match style.top {
                    Val::Px(val) => val,
                    _ => 0.0,
                };

                let target_offset = (current_offset + inverted_scroll_amount)
                    .clamp(max_scroll, min_scroll);

                let smoothed_offset = current_offset + (target_offset - current_offset) * smooth_factor * time.delta_secs();

                style.top = Val::Px(smoothed_offset);
                for (_, styles) in widget_style.styles.iter_mut() {
                    styles.top = Some(Val::Px(smoothed_offset));
                }
            }
        }
    }
}

// ===============================================
//                   Intern Events
// ===============================================

// Main Component

/// Handles click events on the [`ChoiceBox`] base widget.
///
/// When the user clicks the widget:
/// - It toggles the open/closed state of the dropdown.
/// - Sets the widget as focused.
/// - Updates [`CurrentWidgetState`] to reflect the focused widget's ID.
///
/// # Triggered By:
/// - `Trigger<Pointer<Click>>`
///
/// # Affects:
/// - `UIWidgetState::focused`
/// - `UIWidgetState::open`
/// - `CurrentWidgetState::widget_id`
fn on_internal_click(
    trigger: Trigger<Pointer<Click>>,
    mut query: Query<(&mut UIWidgetState, &UIGenID), With<ChoiceBox>>,
    mut current_widget_state: ResMut<CurrentWidgetState>,
) {
    if let Ok((mut state, gen_id)) = query.get_mut(trigger.target) {
        state.focused = true;
        state.open = !state.open;
        current_widget_state.widget_id = gen_id.0;
    }
}

/// Sets `hovered = true` on a [`ChoiceBox`] when the cursor enters its area.
///
/// This system is used to update visual states or styles that react to hovering.
///
/// # Triggered By:
/// - `Trigger<Pointer<Over>>`
///
/// # Affects:
/// - `UIWidgetState::hovered`
fn on_internal_cursor_entered(
    trigger: Trigger<Pointer<Over>>,
    mut query: Query<&mut UIWidgetState, With<ChoiceBox>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.target) {
        state.hovered = true;
    }
}

/// Sets `hovered = false` on a [`ChoiceBox`] when the cursor exits its area.
///
/// # Triggered By:
/// - `Trigger<Pointer<Out>>`
///
/// # Affects:
/// - `UIWidgetState::hovered`
fn on_internal_cursor_leave(
    trigger: Trigger<Pointer<Out>>,
    mut query: Query<&mut UIWidgetState, With<ChoiceBox>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.target) {
        state.hovered = false;
    }
}

// Option Component

/// Handles selection logic when a [`ChoiceOptionBase`] is clicked.
///
/// This system:
/// - Updates the selected state across all sibling options (only one is `checked = true`).
/// - Updates the parent [`ChoiceBox`] value with the clicked option's text/icon.
/// - Closes the dropdown (`open = false`).
/// - Updates any [`SelectedOptionBase`] display widgets with the new value.
/// - Optionally removes focus from the clicked option if no value was selected.
///
/// # Triggered By:
/// - `Trigger<Pointer<Click>>` on a `ChoiceOptionBase`
///
/// # Affects:
/// - `UIWidgetState::checked` on options
/// - `ChoiceBox::value`
/// - `UIWidgetState::open`
/// - `Text` in `SelectedOptionBase`
/// - Optional: `UIWidgetState::focused`
fn on_internal_option_click(
    trigger: Trigger<Pointer<Click>>,
    mut option_query: Query<(Entity, &mut UIWidgetState, &ChoiceOption, &BindToID, &Children), (With<ChoiceOptionBase>, Without<ChoiceBox>)>,
    mut parent_query: Query<(Entity, &mut UIWidgetState, &UIGenID, &mut ChoiceBox), (With<ChoiceBox>, Without<ChoiceOptionBase>)>,
    mut selected_query: Query<(&BindToID, &Children), With<SelectedOptionBase>>,
    mut text_query: Query<&mut Text>,
    mut inner_query: Query<&mut UIWidgetState, (Without<ChoiceOptionBase>,  Without<ChoiceBox>)>,
) {
    let clicked_entity = trigger.target;


    let (clicked_parent_id, clicked_option_text, clicked_option_icon) =
        if let Ok((_, _, option, bind_id, _)) = option_query.get(clicked_entity) {
            (bind_id.0.clone(), option.text.clone(), option.icon_path.clone())
        } else {
            return;
        };

    for (entity, mut state, _, bind_id, children) in option_query.iter_mut() {
        if bind_id.0 == clicked_parent_id {
            state.checked = entity == clicked_entity;

            for child in children.iter() {
                if let Ok(mut inner_state) = inner_query.get_mut(child) {
                    inner_state.checked = state.checked;
                }
            }
        }
    }

    for (_, mut parent_state, id, mut choice_box) in parent_query.iter_mut() {
        if id.0 == clicked_parent_id {
            choice_box.value.text = clicked_option_text.clone();
            choice_box.value.icon_path = clicked_option_icon.clone();
            parent_state.open = false;


            if clicked_option_text.is_empty() && clicked_option_icon.is_none() {
                if let Ok((_, mut state, _, _, _)) = option_query.get_mut(clicked_entity) {
                    state.focused = false;
                }
            }

            for (bind_id, selected_children) in selected_query.iter_mut() {
                if bind_id.0 == clicked_parent_id {
                    for child in selected_children.iter() {
                        if let Ok(mut text) = text_query.get_mut(child) {
                            text.0 = clicked_option_text.clone();
                        }
                    }
                }
            }
        }
    }
}

/// Sets `hovered = true` on a [`ChoiceOptionBase`] and its visual children when hovered.
///
/// Used for hover effects like highlighting or animations.
///
/// # Triggered By:
/// - `Trigger<Pointer<Over>>`
///
/// # Affects:
/// - `UIWidgetState::hovered` on option and visual children
fn on_internal_option_cursor_entered(
    trigger: Trigger<Pointer<Over>>,
    mut query: Query<(&mut UIWidgetState, &Children), With<ChoiceOptionBase>>,
    mut inner_query: Query<&mut UIWidgetState, Without<ChoiceOptionBase>>,
) {
    if let Ok((mut state, children)) = query.get_mut(trigger.target) {
        state.hovered = true;
        
        for child in children.iter() {
            if let Ok(mut inner_state) = inner_query.get_mut(child) {
                inner_state.hovered = true;
            }
        }
    }
}

/// Sets `hovered = false` on a [`ChoiceOptionBase`] and its visual children when unhovered.
///
/// # Triggered By:
/// - `Trigger<Pointer<Out>>`
///
/// # Affects:
/// - `UIWidgetState::hovered` on option and visual children
fn on_internal_option_cursor_leave(
    trigger: Trigger<Pointer<Out>>,
    mut query: Query<(&mut UIWidgetState, &Children), With<ChoiceOptionBase>>,
    mut inner_query: Query<&mut UIWidgetState, Without<ChoiceOptionBase>>,
) {
    if let Ok((mut state, children)) = query.get_mut(trigger.target) {
        state.hovered = false;

        for child in children.iter() {
            if let Ok(mut inner_state) = inner_query.get_mut(child) {
                inner_state.hovered = false;
            }
        }
    }
}

// ===============================================
//                   Child Builder
// ===============================================

/// Spawns the visible child UI elements that represent the currently selected option
/// inside a [`ChoiceBox`] widget.
///
/// This function creates two primary child nodes:
/// 1. **SelectedOptionBase**: A container displaying the selected text.
/// 2. **DropBase**: A container holding the dropdown arrow icon.
///
/// These nodes are styled via CSS classes (`"option-selected"`, `"option-sel-text"`, and `"option-drop-box"`)
/// and tied to the owning widget via [`BindToID`]. They are rendered on the same [`RenderLayers`] level
/// as the parent `ChoiceBox` and ignore pointer interaction via [`Pickable::IGNORE`].
///
/// # Parameters
/// - `builder`: The [`RelatedSpawnerCommands`] to create children in the current entity hierarchy.
/// - `css_source`: A [`CssSource`] reference for applying consistent widget styling.
/// - `choice_box`: The [`ChoiceBox`] from which to derive current selected value and icon.
/// - `layer`: The render layer index for correct UI layering.
/// - `id`: The widget ID used to bind the children to their parent.
/// - `image_cache`: A local image cache to avoid redundant asset loading.
/// - `images`: The global Bevy [`Assets<Image>`] map for dynamically managed textures.
/// - `asset_server`: The [`AssetServer`] for loading icon assets as needed.
///
/// # Spawns:
/// - `SelectedOptionBase`:
///   - Text displaying the selected value (`choice_box.value.text`)
///
/// - `DropBase`:
///   - Icon image from `choice_box.icon_path` or a default fallback
///
/// # Components Added:
/// - [`Name`], [`Node`], [`BackgroundColor`], [`ImageNode`], [`BorderColor`], [`BorderRadius`]
/// - [`UIWidgetState`], [`CssSource`], [`CssClass`], [`RenderLayers`], [`Pickable`], [`BindToID`]
/// - Marker: [`SelectedOptionBase`], [`DropBase`]
fn generate_child_selected_option(
    builder: &mut RelatedSpawnerCommands<ChildOf>,
    css_source: &CssSource,
    choice_box: &ChoiceBox,
    layer: &usize, id: &usize,
    image_cache: &mut ImageCache,
    images: &mut ResMut<Assets<Image>>,
    asset_server: &Res<AssetServer>,
) {
    
    // Selected Container
    builder.spawn((
        Name::new(format!("Option-Selected-{}", choice_box.w_count)),
        Node::default(),
        BackgroundColor::default(),
        ImageNode::default(),
        BorderColor::default(),
        BorderRadius::default(),
        UIWidgetState::default(),
        css_source.clone(),
        CssClass(vec![String::from("option-selected")]),
        RenderLayers::layer(*layer),
        Pickable::IGNORE,
        BindToID(*id),
        SelectedOptionBase
    )).with_children(|builder| {
        
            // Selected Text
            builder.spawn((
                Name::new(format!("Option-Sel-Text-{}", choice_box.w_count)),
                Text::new(choice_box.value.text.clone()),
                TextColor::default(),
                TextFont::default(),
                TextLayout::default(),
                ZIndex::default(),
                UIWidgetState::default(),
                IgnoreParentState,
                css_source.clone(),
                CssClass(vec![String::from("option-sel-text")]),
                Pickable::IGNORE,
                RenderLayers::layer(*layer),
                BindToID(*id)
            ));
    });
    
    builder.spawn((
        Name::new(format!("Arrow-Box-{}", choice_box.w_count)),
        Node::default(),
        BackgroundColor::default(),
        ImageNode::default(),
        BorderColor::default(),
        BorderRadius::default(),
        UIWidgetState::default(),
        css_source.clone(),
        CssClass(vec![String::from("option-drop-box")]),
        RenderLayers::layer(*layer),
        Pickable::IGNORE,
        BindToID(*id),
        DropBase
    )).with_children(|builder| {

        let handle = get_or_load_image(
            choice_box.icon_path.as_deref().unwrap_or(DEFAULT_CHOICE_BOX_KEY),
            image_cache,
            images,
            &asset_server,
        );
            
            builder.spawn((
                Name::new(format!("Drop-Icon-{}", choice_box.w_count)),
                ImageNode {
                    image: handle,
                    ..default()
                },
                ZIndex::default(),
                UIWidgetState::default(),
                css_source.clone(),
                CssClass(vec![String::from("option-drop-icon")]),
                Pickable::IGNORE,
                RenderLayers::layer(*layer),
                BindToID(*id)
            ));
        
    });
}
