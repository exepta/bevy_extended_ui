#[cfg(target_arch = "wasm32")]
use js_sys::Date;
#[cfg(not(target_arch = "wasm32"))]
use std::time::{SystemTime, UNIX_EPOCH};

use crate::services::image_service::{DEFAULT_CHOICE_BOX_KEY, get_or_load_image};
use crate::styles::components::UiStyle;
use crate::styles::paint::Colored;
use crate::styles::{CssClass, CssID, CssSource, TagName};
use crate::widgets::controls::input::InputUserChanged;
use crate::widgets::widget_util::wheel_delta_y;
use crate::widgets::{
    BindToID, DateFormat, DatePicker, IgnoreParentState, InputField, InputType, InputValue,
    UIGenID, UIWidgetState, WidgetId, WidgetKind,
};
use crate::{CurrentWidgetState, ExtendedUiConfiguration, ImageCache};
use bevy::camera::visibility::RenderLayers;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::ui::{RelativeCursorPosition, UiGlobalTransform, UiScale};
use bevy::window::PrimaryWindow;

/// Marker component for initialized date picker widgets.
#[derive(Component)]
struct DatePickerBase;

/// Marker component for the date picker field click area.
#[derive(Component)]
struct DatePickerField;

/// Marker component for the floating label text.
#[derive(Component)]
struct DatePickerLabel;

/// Marker component for the visible selected/placeholder value.
#[derive(Component)]
struct DatePickerValueText;

/// Marker component for the calendar popover container.
#[derive(Component)]
struct DatePickerPopover;

/// Marker component for the month/year headline.
#[derive(Component)]
struct DatePickerHeaderLabel;

/// Marker component for the header month toggle button.
#[derive(Component)]
struct DatePickerHeaderMonthButton;

/// Marker component for the header year toggle button.
#[derive(Component)]
struct DatePickerHeaderYearButton;

/// Marker component for the header year text.
#[derive(Component)]
struct DatePickerHeaderYearText;

/// Marker component for the weekday row container.
#[derive(Component)]
struct DatePickerWeekdays;

/// Marker component for the day-grid container.
#[derive(Component)]
struct DatePickerGrid;

/// Marker component for the year picker list container.
#[derive(Component)]
struct DatePickerYearList;

/// Marker component for the month picker list container.
#[derive(Component)]
struct DatePickerMonthList;

/// Marker component for one year option entry.
#[derive(Component)]
struct DatePickerYearOption {
    year: i32,
}

/// Marker component for one month option entry.
#[derive(Component)]
struct DatePickerMonthOption {
    month: u32,
}

/// Marker component for previous-month button.
#[derive(Component)]
struct DatePickerPrevButton;

/// Marker component for next-month button.
#[derive(Component)]
struct DatePickerNextButton;

/// Marker component for day cells.
#[derive(Component)]
struct DatePickerDayButton {
    index: usize,
}

/// Marker component for day cell text entities.
#[derive(Component)]
struct DatePickerDayText {
    index: usize,
}

const DATE_PICKER_OVERLAY_Z: i32 = 40_000;
const DATE_PICKER_FALLBACK_WIDTH: f32 = 320.0;
const DATE_PICKER_FALLBACK_HEIGHT: f32 = 344.0;

fn set_if_changed<T: PartialEq>(target: &mut T, value: T) {
    if *target != value {
        *target = value;
    }
}

/// Stores the previous z-index of a bound input while its date picker is open.
#[derive(Component, Clone, Copy, Debug)]
struct DatePickerLiftedZ {
    previous: i32,
}

/// Runtime date picker state (calendar view + parsed constraints).
#[derive(Component, Clone, Debug)]
struct DatePickerState {
    selected: Option<SimpleDate>,
    range_start: Option<SimpleDate>,
    range_end: Option<SimpleDate>,
    min: Option<SimpleDate>,
    max: Option<SimpleDate>,
    view_year: i32,
    view_month: u32,
    month_list_open: bool,
    year_list_open: bool,
    year_list_centered: bool,
    pending_bound_write_back: bool,
    year_start: i32,
    year_end: i32,
}

/// Tracks the last visual state applied to a date picker.
#[derive(Component, Clone, Debug, PartialEq)]
struct DatePickerVisualSnapshot {
    value: String,
    input_value: String,
    label: String,
    placeholder: String,
    min: Option<String>,
    max: Option<String>,
    format_pattern: Option<String>,
    format: DateFormat,
    for_id: Option<String>,
    bound_input_type: Option<InputType>,
    bound_date_format: Option<String>,
    selected: Option<SimpleDate>,
    range_start: Option<SimpleDate>,
    range_end: Option<SimpleDate>,
    view_year: i32,
    view_month: u32,
    open: bool,
    focused: bool,
    disabled: bool,
}

/// Tracks the last month/year panel state applied to a date picker.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
struct DatePickerPanelSnapshot {
    view_year: i32,
    view_month: u32,
    year_start: i32,
    year_end: i32,
    month_list_open: bool,
    year_list_open: bool,
    year_list_centered: bool,
    open: bool,
    disabled: bool,
}

/// Compact calendar date type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct SimpleDate {
    year: i32,
    month: u32,
    day: u32,
}

/// Defines the available `PickerSelectionMode` variants for this part of the UI runtime.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PickerSelectionMode {
    /// Variant `Single`.
    Single,
    /// Variant `Range`.
    Range,
}

/// Defines the available `DateFieldOrder` variants for this part of the UI runtime.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DateFieldOrder {
    /// Variant `MonthDayYear`.
    MonthDayYear,
    /// Variant `DayMonthYear`.
    DayMonthYear,
    /// Variant `YearMonthDay`.
    YearMonthDay,
}

/// Represents the `DatePattern` data structure used by the extended UI system.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct DatePattern {
    order: DateFieldOrder,
    separator: char,
}

/// Represents the `CalendarCell` data structure used by the extended UI system.
#[derive(Clone, Copy)]
struct CalendarCell {
    date: SimpleDate,
    in_current_month: bool,
}

/// Plugin that registers date picker widget behavior.
pub struct DatePickerWidget;

impl Plugin for DatePickerWidget {
    /// Registers systems for date picker setup and interaction.
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                internal_node_creation_system,
                sync_bound_date_picker_targets,
                sync_date_picker_visuals,
                sync_year_picker_panel,
                update_date_picker_popover_position,
                handle_year_scroll_events,
                close_unfocused_date_pickers,
            )
                .chain(),
        );
        // Run a late visual sync after click observers to avoid one-frame stale day states
        // (e.g. off-month cells briefly using current-month colors).
        app.add_systems(
            Last,
            (
                sync_date_picker_visuals,
                sync_year_picker_panel,
                update_date_picker_popover_position,
            )
                .chain(),
        );
    }
}

/// Creates UI nodes for each date picker entity.
fn internal_node_creation_system(
    mut commands: Commands,
    query: Query<
        (Entity, &UIGenID, &DatePicker, Option<&CssSource>),
        (With<DatePicker>, Without<DatePickerBase>),
    >,
    input_targets: Query<(&CssID, &InputField), With<InputField>>,
    config: Res<ExtendedUiConfiguration>,
    asset_server: Res<AssetServer>,
    mut image_cache: ResMut<ImageCache>,
    mut images: ResMut<Assets<Image>>,
) {
    let layer = config.render_layers.first().copied().unwrap_or(1);

    for (entity, id, picker, source_opt) in query.iter() {
        if let Some(for_id) = picker.for_id.as_ref() {
            match input_targets
                .iter()
                .find(|(css_id, _)| css_id.0.as_str() == for_id.as_str())
            {
                Some((_css_id, input)) => {
                    if !input_supports_date_picker(input.input_type) {
                        warn!(
                            "DatePicker '{}' requires target input '#{}' to use type='date' or type='range'",
                            picker.entry, for_id
                        );
                    }
                }
                None => {
                    warn!(
                        "DatePicker '{}' could not resolve target input '#{}'",
                        picker.entry, for_id
                    );
                }
            }
        }

        let css_source = source_opt.cloned().unwrap_or_default();
        let bound_input = picker.for_id.as_ref().and_then(|for_id| {
            input_targets
                .iter()
                .find(|(css_id, _)| css_id.0.as_str() == for_id.as_str())
                .map(|(_, input)| input)
        });
        let date_pattern = resolve_date_pattern(picker, bound_input);
        let selection_mode = resolve_selection_mode(bound_input);
        let (range_start, range_end, selected) = match selection_mode {
            PickerSelectionMode::Single => {
                let selected = parse_picker_date(&picker.value, date_pattern);
                (None, None, selected)
            }
            PickerSelectionMode::Range => {
                let parsed = parse_picker_range(&picker.value, date_pattern);
                let start = parsed.map(|(start, _)| start);
                let end = parsed.and_then(|(_, end)| end);
                let selected = end.or(start);
                (start, end, selected)
            }
        };
        let min = picker
            .min
            .as_deref()
            .and_then(|value| parse_picker_date(value, date_pattern));
        let max = picker
            .max
            .as_deref()
            .and_then(|value| parse_picker_date(value, date_pattern));
        let today = today_utc_date();
        let start = selected.unwrap_or(today);
        let (year_start, year_end) = resolve_year_range(min, max, start.year);
        let drop_icon = if picker.for_id.is_none() {
            Some(get_or_load_image(
                DEFAULT_CHOICE_BOX_KEY,
                &mut image_cache,
                &mut images,
                &asset_server,
            ))
        } else {
            None
        };

        commands
            .entity(entity)
            .insert((
                Name::new(format!("DatePicker-{}", picker.entry)),
                Node::default(),
                WidgetId {
                    id: picker.entry,
                    kind: WidgetKind::DatePicker,
                },
                BackgroundColor::default(),
                ImageNode::default(),
                BorderColor::default(),
                BoxShadow::new(
                    Colored::TRANSPARENT,
                    Val::Px(0.),
                    Val::Px(0.),
                    Val::Px(0.),
                    Val::Px(0.),
                ),
                ZIndex::default(),
                Pickable::default(),
                css_source.clone(),
                TagName(String::from("date-picker")),
                RenderLayers::layer(layer),
                DatePickerState {
                    selected,
                    range_start,
                    range_end,
                    min,
                    max,
                    view_year: start.year,
                    view_month: start.month,
                    month_list_open: false,
                    year_list_open: false,
                    year_list_centered: false,
                    pending_bound_write_back: false,
                    year_start,
                    year_end,
                },
                DatePickerBase,
                InputValue(match selection_mode {
                    PickerSelectionMode::Single => selected
                        .map(|date| format_for_display(date, date_pattern))
                        .unwrap_or_default(),
                    PickerSelectionMode::Range => {
                        format_range_for_display(range_start, range_end, date_pattern)
                    }
                }),
            ))
            .insert(GlobalZIndex::default())
            .observe(on_internal_cursor_entered)
            .observe(on_internal_cursor_leave)
            .with_children(|builder| {
                if picker.for_id.is_none() {
                    builder
                        .spawn((
                            Name::new(format!("DatePicker-Field-{}", picker.entry)),
                            Node::default(),
                            UIWidgetState::default(),
                            css_source.clone(),
                            CssClass(vec!["date-picker-field".to_string()]),
                            RenderLayers::layer(layer),
                            Pickable::default(),
                            DatePickerField,
                            BindToID(id.get()),
                        ))
                        .observe(on_field_click)
                        .observe(on_field_cursor_entered)
                        .observe(on_field_cursor_leave)
                        .with_children(|field| {
                            field.spawn((
                                Name::new(format!("DatePicker-Label-{}", picker.entry)),
                                Node::default(),
                                Text::new(picker.label.clone()),
                                TextColor::default(),
                                TextFont::default(),
                                TextLayout::default(),
                                UIWidgetState::default(),
                                css_source.clone(),
                                CssClass(vec!["date-picker-label".to_string()]),
                                RenderLayers::layer(layer),
                                Pickable::IGNORE,
                                DatePickerLabel,
                                BindToID(id.get()),
                            ));

                            field.spawn((
                                Name::new(format!("DatePicker-Value-{}", picker.entry)),
                                Node::default(),
                                Text::new(""),
                                TextColor::default(),
                                TextFont::default(),
                                TextLayout::default(),
                                UIWidgetState::default(),
                                css_source.clone(),
                                CssClass(vec!["date-picker-value".to_string()]),
                                RenderLayers::layer(layer),
                                Pickable::IGNORE,
                                DatePickerValueText,
                                BindToID(id.get()),
                            ));

                            if let Some(drop_icon) = drop_icon.clone() {
                                field.spawn((
                                    Name::new(format!("DatePicker-Icon-{}", picker.entry)),
                                    Node::default(),
                                    ImageNode::new(drop_icon),
                                    UIWidgetState::default(),
                                    css_source.clone(),
                                    CssClass(vec!["date-picker-icon".to_string()]),
                                    RenderLayers::layer(layer),
                                    Pickable::IGNORE,
                                    BindToID(id.get()),
                                ));
                            }
                        });
                }

                builder
                    .spawn((
                        Name::new(format!("DatePicker-Popover-{}", picker.entry)),
                        Node::default(),
                        BackgroundColor::default(),
                        BorderColor::default(),
                        ImageNode::default(),
                        BoxShadow::new(
                            Colored::TRANSPARENT,
                            Val::Px(0.),
                            Val::Px(0.),
                            Val::Px(0.),
                            Val::Px(0.),
                        ),
                        ZIndex::default(),
                        UIWidgetState::default(),
                        css_source.clone(),
                        CssClass(vec!["date-picker-popover".to_string()]),
                        RenderLayers::layer(layer),
                        Visibility::Hidden,
                        Pickable::IGNORE,
                        DatePickerPopover,
                        BindToID(id.get()),
                    ))
                    .insert(GlobalZIndex::default())
                    .with_children(|popover| {
                        popover
                            .spawn((
                                Name::new(format!("DatePicker-Header-{}", picker.entry)),
                                Node::default(),
                                UIWidgetState::default(),
                                css_source.clone(),
                                CssClass(vec!["date-picker-header".to_string()]),
                                RenderLayers::layer(layer),
                                Pickable::IGNORE,
                                BindToID(id.get()),
                            ))
                            .with_children(|header| {
                                header
                                    .spawn((
                                        Name::new(format!("DatePicker-Prev-{}", picker.entry)),
                                        Node::default(),
                                        UIWidgetState::default(),
                                        IgnoreParentState,
                                        css_source.clone(),
                                        CssClass(vec![
                                            "date-picker-nav".to_string(),
                                            "date-picker-prev".to_string(),
                                        ]),
                                        RenderLayers::layer(layer),
                                        Pickable::default(),
                                        DatePickerPrevButton,
                                        BindToID(id.get()),
                                    ))
                                    .with_children(|button| {
                                        button.spawn((
                                            Name::new(format!(
                                                "DatePicker-Prev-Text-{}",
                                                picker.entry
                                            )),
                                            Text::new("<"),
                                            TextColor::default(),
                                            TextFont::default(),
                                            TextLayout::justify(Justify::Center).with_no_wrap(),
                                            UIWidgetState::default(),
                                            css_source.clone(),
                                            CssClass(vec!["date-picker-nav-text".to_string()]),
                                            RenderLayers::layer(layer),
                                            Pickable::IGNORE,
                                            BindToID(id.get()),
                                        ));
                                    })
                                    .observe(on_prev_click)
                                    .observe(on_nav_cursor_entered)
                                    .observe(on_nav_cursor_leave);

                                header
                                    .spawn((
                                        Name::new(format!(
                                            "DatePicker-Header-Center-{}",
                                            picker.entry
                                        )),
                                        Node::default(),
                                        UIWidgetState::default(),
                                        css_source.clone(),
                                        CssClass(vec!["date-picker-header-center".to_string()]),
                                        RenderLayers::layer(layer),
                                        Pickable::IGNORE,
                                        BindToID(id.get()),
                                    ))
                                    .with_children(|center| {
                                        center
                                            .spawn((
                                                Name::new(format!(
                                                    "DatePicker-Month-Button-{}",
                                                    picker.entry
                                                )),
                                                Node::default(),
                                                UIWidgetState::default(),
                                                IgnoreParentState,
                                                css_source.clone(),
                                                CssClass(vec![
                                                    "date-picker-month-button".to_string(),
                                                ]),
                                                RenderLayers::layer(layer),
                                                Pickable::default(),
                                                DatePickerHeaderMonthButton,
                                                BindToID(id.get()),
                                            ))
                                            .with_children(|button| {
                                                button.spawn((
                                                    Name::new(format!(
                                                        "DatePicker-Month-{}",
                                                        picker.entry
                                                    )),
                                                    Node::default(),
                                                    Text::new(""),
                                                    TextColor::default(),
                                                    TextFont::default(),
                                                    TextLayout::default(),
                                                    UIWidgetState::default(),
                                                    css_source.clone(),
                                                    CssClass(vec![
                                                        "date-picker-month-label".to_string(),
                                                    ]),
                                                    RenderLayers::layer(layer),
                                                    Pickable::IGNORE,
                                                    DatePickerHeaderLabel,
                                                    BindToID(id.get()),
                                                ));
                                            })
                                            .observe(on_month_toggle_click)
                                            .observe(on_nav_cursor_entered)
                                            .observe(on_nav_cursor_leave);

                                        center
                                            .spawn((
                                                Name::new(format!(
                                                    "DatePicker-Year-{}",
                                                    picker.entry
                                                )),
                                                Node::default(),
                                                UIWidgetState::default(),
                                                IgnoreParentState,
                                                css_source.clone(),
                                                CssClass(vec![
                                                    "date-picker-year-button".to_string(),
                                                ]),
                                                RenderLayers::layer(layer),
                                                Pickable::default(),
                                                DatePickerHeaderYearButton,
                                                BindToID(id.get()),
                                            ))
                                            .with_children(|button| {
                                                button.spawn((
                                                    Name::new(format!(
                                                        "DatePicker-Year-Text-{}",
                                                        picker.entry
                                                    )),
                                                    Text::new(""),
                                                    TextColor::default(),
                                                    TextFont::default(),
                                                    TextLayout::justify(Justify::Center)
                                                        .with_no_wrap(),
                                                    UIWidgetState::default(),
                                                    css_source.clone(),
                                                    CssClass(vec![
                                                        "date-picker-year-text".to_string(),
                                                    ]),
                                                    RenderLayers::layer(layer),
                                                    Pickable::IGNORE,
                                                    DatePickerHeaderYearText,
                                                    BindToID(id.get()),
                                                ));
                                            })
                                            .observe(on_year_toggle_click)
                                            .observe(on_nav_cursor_entered)
                                            .observe(on_nav_cursor_leave);
                                    });

                                header
                                    .spawn((
                                        Name::new(format!("DatePicker-Next-{}", picker.entry)),
                                        Node::default(),
                                        UIWidgetState::default(),
                                        IgnoreParentState,
                                        css_source.clone(),
                                        CssClass(vec![
                                            "date-picker-nav".to_string(),
                                            "date-picker-next".to_string(),
                                        ]),
                                        RenderLayers::layer(layer),
                                        Pickable::default(),
                                        DatePickerNextButton,
                                        BindToID(id.get()),
                                    ))
                                    .with_children(|button| {
                                        button.spawn((
                                            Name::new(format!(
                                                "DatePicker-Next-Text-{}",
                                                picker.entry
                                            )),
                                            Text::new(">"),
                                            TextColor::default(),
                                            TextFont::default(),
                                            TextLayout::justify(Justify::Center).with_no_wrap(),
                                            UIWidgetState::default(),
                                            css_source.clone(),
                                            CssClass(vec!["date-picker-nav-text".to_string()]),
                                            RenderLayers::layer(layer),
                                            Pickable::IGNORE,
                                            BindToID(id.get()),
                                        ));
                                    })
                                    .observe(on_next_click)
                                    .observe(on_nav_cursor_entered)
                                    .observe(on_nav_cursor_leave);
                            });

                        popover
                            .spawn((
                                Name::new(format!("DatePicker-Weekdays-{}", picker.entry)),
                                Node::default(),
                                UIWidgetState::default(),
                                css_source.clone(),
                                CssClass(vec!["date-picker-weekdays".to_string()]),
                                RenderLayers::layer(layer),
                                Pickable::IGNORE,
                                DatePickerWeekdays,
                                BindToID(id.get()),
                            ))
                            .with_children(|weekday_row| {
                                for day in ["Mo", "Di", "Mi", "Do", "Fr", "Sa", "So"] {
                                    weekday_row
                                        .spawn((
                                            Name::new(format!(
                                                "DatePicker-Weekday-{day}-{}",
                                                picker.entry
                                            )),
                                            Node::default(),
                                            UIWidgetState::default(),
                                            css_source.clone(),
                                            CssClass(vec!["date-picker-weekday".to_string()]),
                                            RenderLayers::layer(layer),
                                            Pickable::IGNORE,
                                            BindToID(id.get()),
                                        ))
                                        .with_children(|weekday| {
                                            weekday.spawn((
                                                Name::new(format!(
                                                    "DatePicker-Weekday-Text-{day}-{}",
                                                    picker.entry
                                                )),
                                                Text::new(day),
                                                TextColor::default(),
                                                TextFont::default(),
                                                TextLayout::justify(Justify::Center).with_no_wrap(),
                                                UIWidgetState::default(),
                                                css_source.clone(),
                                                CssClass(vec![
                                                    "date-picker-weekday-text".to_string(),
                                                ]),
                                                RenderLayers::layer(layer),
                                                Pickable::IGNORE,
                                                BindToID(id.get()),
                                            ));
                                        });
                                }
                            });

                        popover
                            .spawn((
                                Name::new(format!("DatePicker-Grid-{}", picker.entry)),
                                Node::default(),
                                UIWidgetState::default(),
                                css_source.clone(),
                                CssClass(vec!["date-picker-grid".to_string()]),
                                RenderLayers::layer(layer),
                                Pickable::IGNORE,
                                DatePickerGrid,
                                BindToID(id.get()),
                            ))
                            .with_children(|grid| {
                                for index in 0..42 {
                                    grid.spawn((
                                        Name::new(format!(
                                            "DatePicker-Day-{}-{}",
                                            picker.entry, index
                                        )),
                                        Node::default(),
                                        UIWidgetState::default(),
                                        IgnoreParentState,
                                        css_source.clone(),
                                        CssClass(vec!["date-picker-day".to_string()]),
                                        RenderLayers::layer(layer),
                                        Pickable::default(),
                                        DatePickerDayButton { index },
                                        BindToID(id.get()),
                                    ))
                                    .with_children(|day| {
                                        day.spawn((
                                            Name::new(format!(
                                                "DatePicker-Day-Text-{}-{}",
                                                picker.entry, index
                                            )),
                                            Text::new(""),
                                            TextColor(Color::srgb(0.84, 0.86, 0.96)),
                                            TextFont::default(),
                                            TextLayout::justify(Justify::Center).with_no_wrap(),
                                            UIWidgetState::default(),
                                            css_source.clone(),
                                            CssClass(vec!["date-picker-day-text".to_string()]),
                                            RenderLayers::layer(layer),
                                            Pickable::IGNORE,
                                            DatePickerDayText { index },
                                            BindToID(id.get()),
                                        ));
                                    })
                                    .observe(on_day_click)
                                    .observe(on_day_cursor_entered)
                                    .observe(on_day_cursor_leave);
                                }
                            });

                        popover
                            .spawn((
                                Name::new(format!("DatePicker-Years-{}", picker.entry)),
                                Node::default(),
                                UIWidgetState::default(),
                                css_source.clone(),
                                CssClass(vec!["date-picker-years".to_string()]),
                                RenderLayers::layer(layer),
                                Visibility::Hidden,
                                Pickable::default(),
                                DatePickerYearList,
                                BindToID(id.get()),
                            ))
                            .insert(ScrollPosition::default())
                            .observe(on_year_list_click)
                            .with_children(|years| {
                                for year in year_start..=year_end {
                                    years
                                        .spawn((
                                            Name::new(format!(
                                                "DatePicker-Year-Option-{}-{}",
                                                picker.entry, year
                                            )),
                                            Node::default(),
                                            RelativeCursorPosition::default(),
                                            UIWidgetState::default(),
                                            IgnoreParentState,
                                            css_source.clone(),
                                            CssClass(vec!["date-picker-year-option".to_string()]),
                                            RenderLayers::layer(layer),
                                            Pickable::default(),
                                            DatePickerYearOption { year },
                                            BindToID(id.get()),
                                        ))
                                        .with_children(|option| {
                                            option.spawn((
                                                Name::new(format!(
                                                    "DatePicker-Year-Option-Text-{}-{}",
                                                    picker.entry, year
                                                )),
                                                Text::new(year.to_string()),
                                                TextColor::default(),
                                                TextFont::default(),
                                                TextLayout::justify(Justify::Center).with_no_wrap(),
                                                UIWidgetState::default(),
                                                css_source.clone(),
                                                CssClass(vec![
                                                    "date-picker-year-option-text".to_string(),
                                                ]),
                                                RenderLayers::layer(layer),
                                                Pickable::IGNORE,
                                                BindToID(id.get()),
                                            ));
                                        })
                                        .observe(on_year_click)
                                        .observe(on_year_cursor_entered)
                                        .observe(on_year_cursor_leave);
                                }
                            });

                        popover
                            .spawn((
                                Name::new(format!("DatePicker-Months-{}", picker.entry)),
                                Node::default(),
                                UIWidgetState::default(),
                                css_source.clone(),
                                CssClass(vec!["date-picker-months".to_string()]),
                                RenderLayers::layer(layer),
                                Visibility::Hidden,
                                Pickable::default(),
                                DatePickerMonthList,
                                BindToID(id.get()),
                            ))
                            .insert(ScrollPosition::default())
                            .observe(on_month_list_click)
                            .with_children(|months| {
                                for month in 1..=12 {
                                    months
                                        .spawn((
                                            Name::new(format!(
                                                "DatePicker-Month-Option-{}-{}",
                                                picker.entry, month
                                            )),
                                            Node::default(),
                                            RelativeCursorPosition::default(),
                                            UIWidgetState::default(),
                                            IgnoreParentState,
                                            css_source.clone(),
                                            CssClass(vec!["date-picker-month-option".to_string()]),
                                            RenderLayers::layer(layer),
                                            Pickable::default(),
                                            DatePickerMonthOption { month },
                                            BindToID(id.get()),
                                        ))
                                        .with_children(|option| {
                                            option.spawn((
                                                Name::new(format!(
                                                    "DatePicker-Month-Option-Text-{}-{}",
                                                    picker.entry, month
                                                )),
                                                Text::new(month_short_name(month).to_string()),
                                                TextColor::default(),
                                                TextFont::default(),
                                                TextLayout::justify(Justify::Center).with_no_wrap(),
                                                UIWidgetState::default(),
                                                css_source.clone(),
                                                CssClass(vec![
                                                    "date-picker-month-option-text".to_string(),
                                                ]),
                                                RenderLayers::layer(layer),
                                                Pickable::IGNORE,
                                                BindToID(id.get()),
                                            ));
                                        })
                                        .observe(on_month_click)
                                        .observe(on_month_cursor_entered)
                                        .observe(on_month_cursor_leave);
                                }
                            });
                    });
            });
    }
}

/// Synchronizes bound input target state for `for="..."` date pickers.
fn sync_bound_date_picker_targets(
    mut commands: Commands,
    current_widget_state: Res<CurrentWidgetState>,
    mut picker_query: Query<
        (
            Entity,
            &mut DatePicker,
            &mut DatePickerState,
            &mut InputValue,
            &mut UIWidgetState,
            &UIGenID,
        ),
        (
            With<DatePickerBase>,
            Without<DatePickerPopover>,
            Without<DatePickerDayButton>,
            Without<DatePickerDayText>,
            Without<InputField>,
        ),
    >,
    mut input_query: Query<
        (
            Entity,
            &UIGenID,
            &CssID,
            &mut InputField,
            &mut InputValue,
            &mut ZIndex,
            Option<&DatePickerLiftedZ>,
        ),
        (
            With<InputField>,
            Without<DatePickerBase>,
            Without<DatePickerDayButton>,
            Without<DatePickerDayText>,
        ),
    >,
    parent_query: Query<&ChildOf>,
) {
    for (picker_entity, mut picker, mut state, mut input_value, mut ui_state, ui_id) in
        picker_query.iter_mut()
    {
        let Some(for_id) = picker.for_id.as_ref() else {
            continue;
        };
        let mut effective_pattern = resolve_date_pattern(&picker, None);
        let mut selection_mode = PickerSelectionMode::Single;

        let mut resolved_target = false;
        let mut valid_target = false;
        for (target_entity, target_id, css_id, mut input, mut target_value, mut target_z, lifted) in
            input_query.iter_mut()
        {
            if css_id.0 != *for_id {
                continue;
            }
            resolved_target = true;

            if !input_supports_date_picker(input.input_type) {
                if let Some(previous) = lifted {
                    target_z.0 = previous.previous;
                    commands.entity(target_entity).remove::<DatePickerLiftedZ>();
                }
                ui_state.open = false;
                ui_state.checked = false;
                ui_state.focused = false;
                state.month_list_open = false;
                state.year_list_open = false;
                state.year_list_centered = false;
                break;
            }
            valid_target = true;
            selection_mode = resolve_selection_mode(Some(&input));
            effective_pattern = resolve_date_pattern(&picker, Some(&input));

            // Anchor the hidden date-picker host to the input field, so popover positioning follows it.
            let needs_reparent = parent_query
                .get(picker_entity)
                .map(|parent| parent.parent() != target_entity)
                .unwrap_or(true);
            if needs_reparent {
                commands.entity(target_entity).add_child(picker_entity);
            }

            let target_is_active = current_widget_state.widget_id == target_id.get();
            let picker_is_active = current_widget_state.widget_id == ui_id.get();
            let should_elevate_target = ui_state.open
                || picker_is_active
                || target_is_active
                || state.month_list_open
                || state.year_list_open;
            if should_elevate_target {
                if lifted.is_none() {
                    commands.entity(target_entity).insert(DatePickerLiftedZ {
                        previous: target_z.0,
                    });
                }
                if target_z.0 < DATE_PICKER_OVERLAY_Z {
                    target_z.0 = DATE_PICKER_OVERLAY_Z;
                }
            } else if let Some(previous) = lifted {
                target_z.0 = previous.previous;
                commands.entity(target_entity).remove::<DatePickerLiftedZ>();
            }

            if state.pending_bound_write_back {
                let picker_bound_value = match selection_mode {
                    PickerSelectionMode::Single => {
                        if let Some(selected) = state.selected {
                            format_for_display(selected, effective_pattern)
                        } else if let Some(parsed) =
                            parse_picker_date(&picker.value, effective_pattern)
                        {
                            format_for_display(parsed, effective_pattern)
                        } else if !picker.value.trim().is_empty() {
                            picker.value.clone()
                        } else {
                            String::new()
                        }
                    }
                    PickerSelectionMode::Range => {
                        if state.range_start.is_some() {
                            format_range_for_display(
                                state.range_start,
                                state.range_end,
                                effective_pattern,
                            )
                        } else if let Some((start, end)) =
                            parse_picker_range(&picker.value, effective_pattern)
                        {
                            format_range_for_display(Some(start), end, effective_pattern)
                        } else if !picker.value.trim().is_empty() {
                            picker.value.clone()
                        } else {
                            String::new()
                        }
                    }
                };

                if input.text != picker_bound_value {
                    input.text = picker_bound_value.clone();
                }
                if target_value.0 != picker_bound_value {
                    target_value.0 = picker_bound_value;
                }

                state.pending_bound_write_back = false;
                ui_state.focused = target_is_active || picker_is_active;
            } else if target_is_active {
                ui_state.focused = true;
                ui_state.open = true;
                ui_state.checked = true;
                let bound_value = if !target_value.0.trim().is_empty() {
                    target_value.0.clone()
                } else {
                    input.text.clone()
                };
                let normalized =
                    normalize_bound_value_for_mode(&bound_value, effective_pattern, selection_mode);
                if picker.value != normalized {
                    picker.value = normalized;
                }
            } else if picker_is_active {
                ui_state.focused = true;
                let picker_bound_value = match selection_mode {
                    PickerSelectionMode::Single => {
                        if let Some(selected) = state.selected {
                            format_for_display(selected, effective_pattern)
                        } else if let Some(parsed) =
                            parse_picker_date(&picker.value, effective_pattern)
                        {
                            format_for_display(parsed, effective_pattern)
                        } else if !picker.value.trim().is_empty() {
                            picker.value.clone()
                        } else {
                            String::new()
                        }
                    }
                    PickerSelectionMode::Range => {
                        if state.range_start.is_some() {
                            format_range_for_display(
                                state.range_start,
                                state.range_end,
                                effective_pattern,
                            )
                        } else if let Some((start, end)) =
                            parse_picker_range(&picker.value, effective_pattern)
                        {
                            format_range_for_display(Some(start), end, effective_pattern)
                        } else if !picker.value.trim().is_empty() {
                            picker.value.clone()
                        } else {
                            String::new()
                        }
                    }
                };
                if !picker_bound_value.is_empty() {
                    if input.text != picker_bound_value {
                        input.text = picker_bound_value.clone();
                    }
                    if target_value.0 != picker_bound_value {
                        target_value.0 = picker_bound_value;
                    }
                }
            } else {
                ui_state.focused = false;
                let bound_value = if !target_value.0.trim().is_empty() {
                    target_value.0.clone()
                } else {
                    input.text.clone()
                };
                let normalized =
                    normalize_bound_value_for_mode(&bound_value, effective_pattern, selection_mode);
                if picker.value != normalized {
                    picker.value = normalized;
                }
            }
            break;
        }

        if !resolved_target {
            ui_state.open = false;
            ui_state.checked = false;
            ui_state.focused = false;
            state.month_list_open = false;
            state.year_list_open = false;
            state.year_list_centered = false;
            continue;
        }
        if !valid_target {
            continue;
        }

        match selection_mode {
            PickerSelectionMode::Single => {
                let parsed_value = parse_picker_date(&picker.value, effective_pattern);
                if parsed_value != state.selected {
                    state.selected = parsed_value;
                    state.range_start = None;
                    state.range_end = None;
                    if let Some(selected) = state.selected {
                        state.view_year = selected.year;
                        state.view_month = selected.month;
                    }
                }
            }
            PickerSelectionMode::Range => {
                let parsed_range = parse_picker_range(&picker.value, effective_pattern);
                let parsed_start = parsed_range.map(|(start, _)| start);
                let parsed_end = parsed_range.and_then(|(_, end)| end);
                let parsed_selected = parsed_end.or(parsed_start);
                if parsed_start != state.range_start
                    || parsed_end != state.range_end
                    || parsed_selected != state.selected
                {
                    state.range_start = parsed_start;
                    state.range_end = parsed_end;
                    state.selected = parsed_selected;
                    if let Some(selected) = state.selected {
                        state.view_year = selected.year;
                        state.view_month = selected.month;
                    }
                }
            }
        }

        let formatted = match selection_mode {
            PickerSelectionMode::Single => state
                .selected
                .map(|date| format_for_display(date, effective_pattern))
                .unwrap_or_default(),
            PickerSelectionMode::Range => {
                format_range_for_display(state.range_start, state.range_end, effective_pattern)
            }
        };
        if input_value.0 != formatted {
            input_value.0 = formatted;
        }
    }
}

/// Synchronizes labels, value text, calendar header, and day button states.
fn sync_date_picker_visuals(
    mut commands: Commands,
    mut picker_query: Query<
        (
            Entity,
            &mut DatePicker,
            &mut DatePickerState,
            &mut InputValue,
            &mut UIWidgetState,
            &mut ZIndex,
            &mut GlobalZIndex,
            &UIGenID,
            Option<&DatePickerVisualSnapshot>,
        ),
        (
            With<DatePickerBase>,
            Without<DatePickerDayButton>,
            Without<DatePickerDayText>,
            Without<InputField>,
        ),
    >,
    input_targets: Query<(&CssID, &InputField), (With<InputField>, Without<DatePickerBase>)>,
    mut params: ParamSet<(
        Query<
            (
                &mut Text,
                &mut Node,
                &mut TextFont,
                &BindToID,
                Option<&mut UiStyle>,
            ),
            With<DatePickerLabel>,
        >,
        Query<(&mut Text, &mut TextColor, &BindToID), With<DatePickerValueText>>,
        Query<
            (&mut Visibility, &mut GlobalZIndex, &BindToID),
            (With<DatePickerPopover>, Without<DatePickerBase>),
        >,
        Query<(&mut Text, &BindToID), With<DatePickerHeaderLabel>>,
        Query<
            (
                &mut UIWidgetState,
                &DatePickerDayButton,
                &BindToID,
                &mut Node,
                &mut Visibility,
            ),
            (
                With<DatePickerDayButton>,
                Without<DatePickerBase>,
                Without<DatePickerDayText>,
                Without<InputField>,
            ),
        >,
        Query<
            (
                &mut Text,
                &mut TextColor,
                &mut UIWidgetState,
                &DatePickerDayText,
                &BindToID,
                &mut Visibility,
            ),
            (
                With<DatePickerDayText>,
                Without<DatePickerDayButton>,
                Without<DatePickerBase>,
                Without<InputField>,
            ),
        >,
    )>,
) {
    for (
        entity,
        mut picker,
        mut state,
        mut input_value,
        ui_state,
        mut root_z,
        mut root_global_z,
        ui_id,
        snapshot,
    ) in picker_query.iter_mut()
    {
        let bound_input = picker.for_id.as_ref().and_then(|for_id| {
            input_targets
                .iter()
                .find(|(css_id, _)| css_id.0.as_str() == for_id.as_str())
                .map(|(_, input)| input)
        });
        let selection_mode = resolve_selection_mode(bound_input);
        let effective_pattern = resolve_date_pattern(&picker, bound_input);
        let next_snapshot = DatePickerVisualSnapshot {
            value: picker.value.clone(),
            input_value: input_value.0.clone(),
            label: picker.label.clone(),
            placeholder: picker.placeholder.clone(),
            min: picker.min.clone(),
            max: picker.max.clone(),
            format_pattern: picker.format_pattern.clone(),
            format: picker.format,
            for_id: picker.for_id.clone(),
            bound_input_type: bound_input.map(|input| input.input_type),
            bound_date_format: bound_input.and_then(|input| input.date_format.clone()),
            selected: state.selected,
            range_start: state.range_start,
            range_end: state.range_end,
            view_year: state.view_year,
            view_month: state.view_month,
            open: ui_state.open,
            focused: ui_state.focused,
            disabled: ui_state.disabled,
        };
        if snapshot.is_some_and(|snapshot| *snapshot == next_snapshot) {
            continue;
        }

        if ui_state.open && !ui_state.disabled {
            set_if_changed(&mut root_z.0, DATE_PICKER_OVERLAY_Z);
            set_if_changed(&mut root_global_z.0, DATE_PICKER_OVERLAY_Z);
        } else {
            set_if_changed(&mut root_z.0, 0);
            set_if_changed(&mut root_global_z.0, 0);
        }

        let min = picker
            .min
            .as_deref()
            .and_then(|value| parse_picker_date(value, effective_pattern));
        let max = picker
            .max
            .as_deref()
            .and_then(|value| parse_picker_date(value, effective_pattern));
        set_if_changed(&mut state.min, min);
        set_if_changed(&mut state.max, max);
        let (year_start, year_end) = resolve_year_range(state.min, state.max, state.view_year);
        set_if_changed(&mut state.year_start, year_start);
        set_if_changed(&mut state.year_end, year_end);

        match selection_mode {
            PickerSelectionMode::Single => {
                let parsed_value = parse_picker_date(&picker.value, effective_pattern);
                if parsed_value != state.selected {
                    state.selected = parsed_value;
                    state.range_start = None;
                    state.range_end = None;
                    if let Some(selected) = state.selected {
                        state.view_year = selected.year;
                        state.view_month = selected.month;
                    }
                }
            }
            PickerSelectionMode::Range => {
                let parsed_range = parse_picker_range(&picker.value, effective_pattern);
                let parsed_start = parsed_range.map(|(start, _)| start);
                let parsed_end = parsed_range.and_then(|(_, end)| end);
                let parsed_selected = parsed_end.or(parsed_start);
                if parsed_start != state.range_start
                    || parsed_end != state.range_end
                    || parsed_selected != state.selected
                {
                    state.range_start = parsed_start;
                    state.range_end = parsed_end;
                    state.selected = parsed_selected;
                    if let Some(selected) = state.selected {
                        state.view_year = selected.year;
                        state.view_month = selected.month;
                    }
                }
            }
        }

        let formatted = match selection_mode {
            PickerSelectionMode::Single => state
                .selected
                .map(|date| format_for_display(date, effective_pattern))
                .unwrap_or_default(),
            PickerSelectionMode::Range => {
                format_range_for_display(state.range_start, state.range_end, effective_pattern)
            }
        };
        if input_value.0 != formatted {
            input_value.0 = formatted.clone();
        }
        if picker.value != formatted {
            picker.value = formatted;
        }

        let has_value = match selection_mode {
            PickerSelectionMode::Single => state.selected.is_some(),
            PickerSelectionMode::Range => state.range_start.is_some(),
        };
        let float_label = ui_state.focused || ui_state.open || has_value;

        {
            let mut label_query = params.p0();
            for (mut label_text, mut label_node, mut label_font, bind_id, style_opt) in
                label_query.iter_mut()
            {
                if bind_id.0 != ui_id.get() {
                    continue;
                }

                set_if_changed(&mut label_text.0, picker.label.clone());
                if float_label {
                    set_if_changed(&mut label_node.top, Val::Px(7.0));
                    set_if_changed(&mut label_font.font_size, FontSize::Px(11.0));
                } else {
                    set_if_changed(&mut label_node.top, Val::Px(19.0));
                    set_if_changed(&mut label_font.font_size, FontSize::Px(16.0));
                }

                if let Some(mut style) = style_opt {
                    for (_, rules) in style.styles.iter_mut() {
                        rules.normal.top = Some(label_node.top);
                        rules.normal.font_size = Some(label_font.font_size);
                    }
                }
            }
        }

        let placeholder = if picker.placeholder.trim().is_empty() {
            match selection_mode {
                PickerSelectionMode::Single => default_placeholder_for_format(effective_pattern),
                PickerSelectionMode::Range => {
                    let single = default_placeholder_for_format(effective_pattern);
                    format!("{single} - {single}")
                }
            }
        } else {
            picker.placeholder.clone()
        };

        {
            let mut value_query = params.p1();
            for (mut value_text, mut value_color, bind_id) in value_query.iter_mut() {
                if bind_id.0 != ui_id.get() {
                    continue;
                }

                if has_value {
                    set_if_changed(
                        &mut value_text.0,
                        match selection_mode {
                            PickerSelectionMode::Single => state
                                .selected
                                .map(|selected| format_for_display(selected, effective_pattern))
                                .unwrap_or_default(),
                            PickerSelectionMode::Range => format_range_for_display(
                                state.range_start,
                                state.range_end,
                                effective_pattern,
                            ),
                        },
                    );
                    set_if_changed(&mut value_color.0, Color::srgb(0.96, 0.97, 1.0));
                } else if float_label {
                    set_if_changed(&mut value_text.0, placeholder.clone());
                    set_if_changed(&mut value_color.0, Color::srgba(0.69, 0.72, 0.82, 0.98));
                } else {
                    // Keep placeholder hidden while the label is not floated to avoid overlap.
                    if !value_text.0.is_empty() {
                        value_text.0.clear();
                    }
                }
            }
        }

        {
            let mut popover_query = params.p2();
            for (mut visibility, mut popover_global_z, bind_id) in popover_query.iter_mut() {
                if bind_id.0 != ui_id.get() {
                    continue;
                }
                if ui_state.open && !ui_state.disabled {
                    set_if_changed(&mut *visibility, Visibility::Inherited);
                    set_if_changed(&mut popover_global_z.0, DATE_PICKER_OVERLAY_Z + 1);
                } else {
                    set_if_changed(&mut *visibility, Visibility::Hidden);
                    set_if_changed(&mut popover_global_z.0, 0);
                }
            }
        }

        {
            let mut month_query = params.p3();
            for (mut month_text, bind_id) in month_query.iter_mut() {
                if bind_id.0 != ui_id.get() {
                    continue;
                }
                set_if_changed(&mut month_text.0, month_name(state.view_month).to_string());
            }
        }

        let cells = build_calendar_cells(state.view_year, state.view_month);
        let visible_rows = visible_calendar_row_count(state.view_year, state.view_month);
        let visible_cell_count = visible_rows * 7;
        {
            let mut day_query = params.p4();
            for (mut day_state, button, bind_id, mut day_node, mut day_visibility) in
                day_query.iter_mut()
            {
                if bind_id.0 != ui_id.get() {
                    continue;
                }

                let hidden = button.index >= visible_cell_count;
                if hidden {
                    set_if_changed(&mut day_node.display, Display::None);
                    set_if_changed(&mut *day_visibility, Visibility::Hidden);
                    set_if_changed(&mut day_state.checked, false);
                    set_if_changed(&mut day_state.readonly, true);
                    set_if_changed(&mut day_state.disabled, true);
                    set_if_changed(&mut day_state.hovered, false);
                    set_if_changed(&mut day_state.focused, false);
                    set_if_changed(&mut day_state.invalid, false);
                    continue;
                }

                set_if_changed(&mut day_node.display, Display::Flex);
                set_if_changed(&mut *day_visibility, Visibility::Inherited);

                let Some(cell) = cells.get(button.index) else {
                    continue;
                };

                let is_endpoint = match selection_mode {
                    PickerSelectionMode::Single => state.selected == Some(cell.date),
                    PickerSelectionMode::Range => {
                        state.range_start == Some(cell.date) || state.range_end == Some(cell.date)
                    }
                };
                let is_range_start = selection_mode == PickerSelectionMode::Range
                    && state.range_start == Some(cell.date);
                let is_range_end = selection_mode == PickerSelectionMode::Range
                    && state.range_end == Some(cell.date);
                let same_start_end = is_range_start && is_range_end;
                let in_range = match selection_mode {
                    PickerSelectionMode::Single => false,
                    PickerSelectionMode::Range => {
                        in_selected_range(cell.date, state.range_start, state.range_end)
                    }
                };
                let readonly = !cell.in_current_month;
                let disabled =
                    ui_state.disabled || !is_date_allowed(cell.date, state.min, state.max);
                let highlight_range = in_range && !is_endpoint && !readonly && !disabled;
                let decorate_range_caps = selection_mode == PickerSelectionMode::Range
                    && state.range_start.is_some()
                    && state.range_end.is_some()
                    && !same_start_end
                    && !readonly
                    && !disabled;
                let range_start_cap = decorate_range_caps && is_range_start;
                let range_end_cap = decorate_range_caps && is_range_end;

                set_if_changed(&mut day_state.checked, is_endpoint);
                set_if_changed(&mut day_state.readonly, readonly);
                set_if_changed(&mut day_state.disabled, disabled);
                set_if_changed(&mut day_state.focused, highlight_range || range_end_cap);
                set_if_changed(&mut day_state.invalid, range_start_cap);
                if day_state.disabled || day_state.readonly {
                    set_if_changed(&mut day_state.hovered, false);
                }
            }
        }

        {
            let mut day_text_query = params.p5();
            for (mut text, mut text_color, mut text_state, text_info, bind_id, mut visibility) in
                day_text_query.iter_mut()
            {
                if bind_id.0 != ui_id.get() {
                    continue;
                }

                let hidden = text_info.index >= visible_cell_count;
                if hidden {
                    if !text.0.is_empty() {
                        text.0.clear();
                    }
                    set_if_changed(&mut text_color.0, Color::srgb(0.84, 0.86, 0.96));
                    set_if_changed(&mut *visibility, Visibility::Hidden);
                    set_if_changed(&mut text_state.checked, false);
                    set_if_changed(&mut text_state.readonly, true);
                    set_if_changed(&mut text_state.disabled, true);
                    set_if_changed(&mut text_state.hovered, false);
                    set_if_changed(&mut text_state.focused, false);
                    set_if_changed(&mut text_state.invalid, false);
                    continue;
                }

                set_if_changed(&mut *visibility, Visibility::Inherited);

                let Some(cell) = cells.get(text_info.index) else {
                    continue;
                };

                set_if_changed(&mut text.0, cell.date.day.to_string());
                let selected = match selection_mode {
                    PickerSelectionMode::Single => state.selected == Some(cell.date),
                    PickerSelectionMode::Range => {
                        state.range_start == Some(cell.date) || state.range_end == Some(cell.date)
                    }
                };
                let is_range_start = selection_mode == PickerSelectionMode::Range
                    && state.range_start == Some(cell.date);
                let is_range_end = selection_mode == PickerSelectionMode::Range
                    && state.range_end == Some(cell.date);
                let same_start_end = is_range_start && is_range_end;
                let in_range = match selection_mode {
                    PickerSelectionMode::Single => false,
                    PickerSelectionMode::Range => {
                        in_selected_range(cell.date, state.range_start, state.range_end)
                    }
                };
                let readonly = !cell.in_current_month;
                let disabled =
                    ui_state.disabled || !is_date_allowed(cell.date, state.min, state.max);
                let highlight_range = in_range && !selected && !readonly && !disabled;
                let decorate_range_caps = selection_mode == PickerSelectionMode::Range
                    && state.range_start.is_some()
                    && state.range_end.is_some()
                    && !same_start_end
                    && !readonly
                    && !disabled;
                let range_start_cap = decorate_range_caps && is_range_start;
                let range_end_cap = decorate_range_caps && is_range_end;

                set_if_changed(&mut text_state.checked, selected);
                set_if_changed(&mut text_state.readonly, readonly);
                set_if_changed(&mut text_state.disabled, disabled);
                set_if_changed(&mut text_state.focused, highlight_range || range_end_cap);
                set_if_changed(&mut text_state.invalid, range_start_cap);
                if disabled {
                    set_if_changed(&mut text_state.hovered, false);
                }

                set_if_changed(
                    &mut text_color.0,
                    if selected {
                        Color::WHITE
                    } else if highlight_range {
                        Color::srgb(0.88, 0.9, 0.98)
                    } else if disabled {
                        Color::srgb(0.31, 0.33, 0.41)
                    } else if readonly {
                        Color::srgb(0.29, 0.31, 0.41)
                    } else {
                        Color::srgb(0.84, 0.86, 0.96)
                    },
                );
            }
        }

        commands.entity(entity).insert(next_snapshot);
    }
}

fn update_date_picker_popover_position(
    window_q: Query<&Window, With<PrimaryWindow>>,
    ui_scale: Res<UiScale>,
    picker_q: Query<
        (
            &UIGenID,
            &DatePicker,
            &UIWidgetState,
            &ComputedNode,
            &UiGlobalTransform,
        ),
        (
            With<DatePickerBase>,
            Without<DatePickerPopover>,
            Without<DatePickerField>,
            Without<InputField>,
        ),
    >,
    field_q: Query<
        (&BindToID, &ComputedNode, &UiGlobalTransform),
        (With<DatePickerField>, Without<DatePickerBase>),
    >,
    input_q: Query<
        (&CssID, &ComputedNode, &UiGlobalTransform),
        (With<InputField>, Without<DatePickerBase>),
    >,
    mut popover_q: Query<
        (&mut Node, &BindToID, &ComputedNode, Option<&mut UiStyle>),
        (With<DatePickerPopover>, Without<DatePickerBase>),
    >,
) {
    let Ok(window) = window_q.single() else {
        return;
    };

    let gap = 6.0;
    let margin = 8.0;
    let scale = (window.scale_factor() * ui_scale.0).max(f32::EPSILON);
    let viewport_size = Vec2::new(window.width(), window.height());

    for (id, picker, state, root_node, root_transform) in picker_q.iter() {
        if !state.open || state.disabled {
            continue;
        }

        let Some((anchor_node, anchor_transform)) =
            resolve_date_picker_anchor(id, picker, &field_q, &input_q)
        else {
            continue;
        };

        let Some((mut popover_node, popover_computed, maybe_styles)) = popover_q
            .iter_mut()
            .find(|(_, bind, _, _)| bind.0 == id.0)
            .map(|(node, _, computed, styles)| (node, computed, styles))
        else {
            continue;
        };

        let anchor_size = logical_size(anchor_node);
        let popover_size = logical_size(popover_computed);
        let root_top_left = top_left_ui(root_node, root_transform, scale);
        let anchor_top_left = top_left_ui(anchor_node, anchor_transform, scale);

        let anchor_w = anchor_size.x.max(1.0);
        let anchor_h = anchor_size.y.max(1.0);
        let raw_popover_w = if popover_size.x.is_finite() && popover_size.x > 8.0 {
            popover_size.x
        } else {
            DATE_PICKER_FALLBACK_WIDTH
        };
        let raw_popover_h = if popover_size.y.is_finite() && popover_size.y > 8.0 {
            popover_size.y
        } else {
            DATE_PICKER_FALLBACK_HEIGHT
        };

        let popover_w = raw_popover_w.min((viewport_size.x - margin * 2.0).max(160.0));
        let popover_h = raw_popover_h.min((viewport_size.y - margin * 2.0).max(180.0));
        let ideal_x = anchor_top_left.x + (anchor_w - popover_w) * 0.5;
        let max_x = (viewport_size.x - popover_w - margin).max(margin);
        let absolute_x = ideal_x.clamp(margin, max_x);

        let below_y = anchor_top_left.y + anchor_h + gap;
        let above_y = anchor_top_left.y - popover_h - gap;
        let absolute_y = if below_y + popover_h <= viewport_size.y - margin || above_y < margin {
            below_y
        } else {
            above_y
        }
        .clamp(margin, (viewport_size.y - popover_h - margin).max(margin));

        let local_left = Val::Px(absolute_x - root_top_left.x);
        let local_top = Val::Px(absolute_y - root_top_left.y);
        set_if_changed(&mut popover_node.left, local_left);
        set_if_changed(&mut popover_node.top, local_top);

        if let Some(mut styles) = maybe_styles {
            let mut changed = false;
            {
                let styles = styles.bypass_change_detection();
                if let Some(active) = styles.active_style.as_mut() {
                    if active.left != Some(local_left) {
                        active.left = Some(local_left);
                        changed = true;
                    }
                    if active.top != Some(local_top) {
                        active.top = Some(local_top);
                        changed = true;
                    }
                }
            }
            if changed {
                styles.set_changed();
            }
        }
    }
}

fn resolve_date_picker_anchor<'a>(
    id: &UIGenID,
    picker: &DatePicker,
    field_q: &'a Query<
        (&BindToID, &ComputedNode, &UiGlobalTransform),
        (With<DatePickerField>, Without<DatePickerBase>),
    >,
    input_q: &'a Query<
        (&CssID, &ComputedNode, &UiGlobalTransform),
        (With<InputField>, Without<DatePickerBase>),
    >,
) -> Option<(&'a ComputedNode, &'a UiGlobalTransform)> {
    if let Some(for_id) = picker.for_id.as_deref() {
        let normalized = for_id.trim().trim_start_matches('#');
        for (css_id, node, transform) in input_q.iter() {
            if css_id.0.trim().trim_start_matches('#') == normalized {
                return Some((node, transform));
            }
        }
    }

    field_q
        .iter()
        .find(|(bind, _, _)| bind.0 == id.0)
        .map(|(_, node, transform)| (node, transform))
}

fn logical_size(node: &ComputedNode) -> Vec2 {
    let inv = node.inverse_scale_factor.max(f32::EPSILON);
    node.size() * inv
}

fn top_left_ui(node: &ComputedNode, transform: &UiGlobalTransform, scale: f32) -> Vec2 {
    let half = node.size() * 0.5;
    transform.affine().transform_point2(-half) / scale
}

/// Synchronizes month/year panel visibility and option states.
fn sync_year_picker_panel(
    mut commands: Commands,
    mut picker_query: Query<
        (
            Entity,
            &mut DatePickerState,
            &UIWidgetState,
            &UIGenID,
            Option<&DatePickerPanelSnapshot>,
        ),
        With<DatePickerBase>,
    >,
    mut month_year_params: ParamSet<(
        Query<(&mut Text, &BindToID), (With<DatePickerHeaderYearText>, Without<DatePickerBase>)>,
        Query<
            (
                &mut UIWidgetState,
                &BindToID,
                Option<&DatePickerHeaderYearButton>,
                Option<&DatePickerHeaderMonthButton>,
            ),
            (
                Or<(
                    With<DatePickerHeaderYearButton>,
                    With<DatePickerHeaderMonthButton>,
                )>,
                Without<DatePickerBase>,
            ),
        >,
        Query<(&mut Visibility, &mut Node, &BindToID), With<DatePickerWeekdays>>,
        Query<(&mut Visibility, &mut Node, &BindToID), With<DatePickerGrid>>,
        Query<
            (
                Entity,
                &mut Visibility,
                &mut Node,
                &mut ScrollPosition,
                &ComputedNode,
                &BindToID,
            ),
            With<DatePickerYearList>,
        >,
        Query<
            (
                &mut UIWidgetState,
                &DatePickerYearOption,
                &BindToID,
                &ComputedNode,
                &ChildOf,
            ),
            (With<DatePickerYearOption>, Without<DatePickerBase>),
        >,
        Query<(&mut Visibility, &mut Node, &BindToID), With<DatePickerMonthList>>,
        Query<
            (&mut UIWidgetState, &DatePickerMonthOption, &BindToID),
            (With<DatePickerMonthOption>, Without<DatePickerBase>),
        >,
    )>,
) {
    for (entity, mut state, picker_ui, picker_id, snapshot) in picker_query.iter_mut() {
        let next_snapshot = DatePickerPanelSnapshot {
            view_year: state.view_year,
            view_month: state.view_month,
            year_start: state.year_start,
            year_end: state.year_end,
            month_list_open: state.month_list_open,
            year_list_open: state.year_list_open,
            year_list_centered: state.year_list_centered,
            open: picker_ui.open,
            disabled: picker_ui.disabled,
        };
        if snapshot.is_some_and(|snapshot| *snapshot == next_snapshot) {
            continue;
        }

        let show_month_list =
            picker_ui.open && state.month_list_open && !state.year_list_open && !picker_ui.disabled;
        let show_year_list =
            picker_ui.open && state.year_list_open && !state.month_list_open && !picker_ui.disabled;
        let hide_calendar = show_year_list || show_month_list;

        {
            let mut year_label_query = month_year_params.p0();
            for (mut year_text, bind_id) in year_label_query.iter_mut() {
                if bind_id.0 != picker_id.get() {
                    continue;
                }
                set_if_changed(&mut year_text.0, state.view_year.to_string());
            }
        }

        {
            let mut button_query = month_year_params.p1();
            for (mut button_state, bind_id, is_year, is_month) in button_query.iter_mut() {
                if bind_id.0 != picker_id.get() {
                    continue;
                }

                if is_year.is_some() {
                    set_if_changed(&mut button_state.checked, show_year_list);
                    if !show_year_list {
                        set_if_changed(&mut button_state.hovered, false);
                    }
                } else if is_month.is_some() {
                    set_if_changed(&mut button_state.checked, show_month_list);
                    if !show_month_list {
                        set_if_changed(&mut button_state.hovered, false);
                    }
                }
            }
        }

        {
            let mut weekdays_query = month_year_params.p2();
            for (mut visibility, mut node, bind_id) in weekdays_query.iter_mut() {
                if bind_id.0 != picker_id.get() {
                    continue;
                }
                if hide_calendar {
                    set_if_changed(&mut *visibility, Visibility::Hidden);
                    set_if_changed(&mut node.display, Display::None);
                } else {
                    set_if_changed(&mut *visibility, Visibility::Inherited);
                    set_if_changed(&mut node.display, Display::Flex);
                }
            }
        }

        {
            let mut grid_query = month_year_params.p3();
            for (mut visibility, mut node, bind_id) in grid_query.iter_mut() {
                if bind_id.0 != picker_id.get() {
                    continue;
                }
                if hide_calendar {
                    set_if_changed(&mut *visibility, Visibility::Hidden);
                    set_if_changed(&mut node.display, Display::None);
                } else {
                    set_if_changed(&mut *visibility, Visibility::Inherited);
                    set_if_changed(&mut node.display, Display::Flex);
                }
            }
        }

        {
            let mut years_query = month_year_params.p4();
            for (_list_entity, mut visibility, mut node, mut scroll, list_computed, bind_id) in
                years_query.iter_mut()
            {
                if bind_id.0 != picker_id.get() {
                    continue;
                }
                if show_year_list {
                    set_if_changed(&mut *visibility, Visibility::Inherited);
                    set_if_changed(&mut node.display, Display::Flex);
                    if !state.year_list_centered {
                        // One year row is 34px high with 2px vertical gap in CSS.
                        let option_h = 36.0;
                        let inv_sf = list_computed.inverse_scale_factor.max(f32::EPSILON);
                        let viewport_h = (list_computed.size().y * inv_sf).max(1.0);
                        let total_items = (state.year_end - state.year_start + 1).max(1) as f32;
                        let content_h = total_items * option_h;
                        let max_scroll = (content_h - viewport_h).max(0.0);
                        // During the first frame after showing, computed height can still be invalid.
                        // Wait until the viewport is plausibly laid out before locking centering.
                        if viewport_h > option_h * 2.0 {
                            let index = (state.view_year - state.year_start).max(0) as f32;
                            let target = (index * option_h - (viewport_h * 0.5) + (option_h * 0.5))
                                .clamp(0.0, max_scroll);
                            set_if_changed(&mut scroll.y, target);
                            set_if_changed(&mut state.year_list_centered, true);
                        }
                    }
                } else {
                    set_if_changed(&mut *visibility, Visibility::Hidden);
                    set_if_changed(&mut node.display, Display::None);
                    set_if_changed(&mut state.year_list_centered, false);
                }
            }
        }

        {
            let mut option_query = month_year_params.p5();
            for (mut option_state, option, bind_id, _opt_computed, _parent) in
                option_query.iter_mut()
            {
                if bind_id.0 != picker_id.get() {
                    continue;
                }

                let disabled = option.year < state.year_start || option.year > state.year_end;
                set_if_changed(&mut option_state.checked, option.year == state.view_year);
                set_if_changed(&mut option_state.disabled, disabled);
                if disabled || !show_year_list {
                    set_if_changed(&mut option_state.hovered, false);
                }
            }
        }

        {
            let mut months_query = month_year_params.p6();
            for (mut visibility, mut node, bind_id) in months_query.iter_mut() {
                if bind_id.0 != picker_id.get() {
                    continue;
                }
                if show_month_list {
                    set_if_changed(&mut *visibility, Visibility::Inherited);
                    set_if_changed(&mut node.display, Display::Flex);
                } else {
                    set_if_changed(&mut *visibility, Visibility::Hidden);
                    set_if_changed(&mut node.display, Display::None);
                }
            }
        }

        {
            let mut option_query = month_year_params.p7();
            for (mut option_state, option, bind_id) in option_query.iter_mut() {
                if bind_id.0 != picker_id.get() {
                    continue;
                }

                let disabled = picker_ui.disabled
                    || !is_month_allowed(state.view_year, option.month, state.min, state.max);
                set_if_changed(&mut option_state.checked, option.month == state.view_month);
                set_if_changed(&mut option_state.disabled, disabled);
                if disabled || !show_month_list {
                    set_if_changed(&mut option_state.hovered, false);
                }
            }
        }

        commands.entity(entity).insert(DatePickerPanelSnapshot {
            view_year: state.view_year,
            view_month: state.view_month,
            year_start: state.year_start,
            year_end: state.year_end,
            month_list_open: state.month_list_open,
            year_list_open: state.year_list_open,
            year_list_centered: state.year_list_centered,
            open: picker_ui.open,
            disabled: picker_ui.disabled,
        });
    }
}

/// Handles mouse-wheel scrolling for the month/year picker lists.
fn handle_year_scroll_events(
    mut scroll_events: MessageReader<MouseWheel>,
    time: Res<Time>,
    mut year_list_query: Query<
        (
            Entity,
            &Visibility,
            &Children,
            &mut ScrollPosition,
            &ComputedNode,
        ),
        Or<(With<DatePickerYearList>, With<DatePickerMonthList>)>,
    >,
    option_query: Query<
        (&ComputedNode, &ChildOf),
        Or<(With<DatePickerYearOption>, With<DatePickerMonthOption>)>,
    >,
) {
    if scroll_events.is_empty() {
        return;
    }

    let smooth_factor = 24.0;

    for event in scroll_events.read() {
        for (list_entity, visibility, children, mut scroll, list_computed) in
            year_list_query.iter_mut()
        {
            let is_visible = matches!(*visibility, Visibility::Visible | Visibility::Inherited);
            if !is_visible {
                continue;
            }

            let inv_sf = list_computed.inverse_scale_factor.max(f32::EPSILON);
            let delta = -wheel_delta_y(event, inv_sf);

            if children.is_empty() {
                scroll.y = 0.0;
                continue;
            }

            let mut option_height = None;
            for (opt_computed, parent) in option_query.iter() {
                if parent.parent() == list_entity {
                    let opt_inv_sf = opt_computed.inverse_scale_factor.max(f32::EPSILON);
                    option_height = Some((opt_computed.size().y * opt_inv_sf).max(1.0));
                    break;
                }
            }

            let option_h = option_height.unwrap_or(34.0);
            let viewport_h = (list_computed.size().y * inv_sf).max(1.0);
            let content_h = children.len() as f32 * option_h;
            let max_scroll = (content_h - viewport_h).max(0.0);

            let target = (scroll.y + delta).clamp(0.0, max_scroll);
            let smoothed = scroll.y + (target - scroll.y) * smooth_factor * time.delta_secs();
            scroll.y = smoothed.clamp(0.0, max_scroll);
        }
    }
}

/// Closes popovers once the picker loses focus.
fn close_unfocused_date_pickers(
    mut query: Query<(&mut UIWidgetState, &mut DatePickerState), With<DatePickerBase>>,
) {
    for (mut state, mut picker_state) in query.iter_mut() {
        if !state.focused && state.open {
            state.open = false;
            state.checked = false;
            picker_state.month_list_open = false;
            picker_state.year_list_open = false;
            picker_state.year_list_centered = false;
        }
    }
}

/// Toggles the popover when clicking the date picker field.
fn on_field_click(
    mut trigger: On<Pointer<Click>>,
    field_query: Query<&BindToID, With<DatePickerField>>,
    mut picker_query: Query<
        (&UIGenID, &mut DatePickerState, &mut UIWidgetState),
        With<DatePickerBase>,
    >,
    mut current_widget_state: ResMut<CurrentWidgetState>,
) {
    let Ok(bind) = field_query.get(trigger.entity) else {
        return;
    };

    let mut was_open = false;
    let mut disabled = false;
    for (id, _picker_state, state) in picker_query.iter_mut() {
        if id.get() != bind.0 {
            continue;
        }
        was_open = state.open;
        disabled = state.disabled;
        break;
    }

    if disabled {
        trigger.propagate(false);
        return;
    }

    let new_open = !was_open;
    for (id, mut picker_state, mut state) in picker_query.iter_mut() {
        if id.get() == bind.0 {
            state.focused = true;
            state.open = new_open;
            state.checked = new_open;
            if !new_open {
                picker_state.month_list_open = false;
                picker_state.year_list_open = false;
                picker_state.year_list_centered = false;
            }
            current_widget_state.widget_id = id.get();
        } else {
            state.open = false;
            state.checked = false;
            picker_state.month_list_open = false;
            picker_state.year_list_open = false;
            picker_state.year_list_centered = false;
        }
    }

    trigger.propagate(false);
}

/// Moves calendar view one month backwards.
fn on_prev_click(
    mut trigger: On<Pointer<Click>>,
    bind_query: Query<&BindToID, With<DatePickerPrevButton>>,
    mut picker_query: Query<
        (&UIGenID, &mut DatePickerState, &mut UIWidgetState),
        With<DatePickerBase>,
    >,
    mut current_widget_state: ResMut<CurrentWidgetState>,
) {
    let Ok(bind) = bind_query.get(trigger.entity) else {
        return;
    };

    for (id, mut state, mut ui_state) in picker_query.iter_mut() {
        if id.get() != bind.0 {
            continue;
        }
        if ui_state.disabled {
            trigger.propagate(false);
            return;
        }

        let (year, month) = shift_month(state.view_year, state.view_month, -1);
        state.view_year = year;
        state.view_month = month;
        state.month_list_open = false;
        state.year_list_open = false;
        state.year_list_centered = false;
        ui_state.focused = true;
        ui_state.open = true;
        ui_state.checked = true;
        current_widget_state.widget_id = id.get();
        break;
    }

    trigger.propagate(false);
}

/// Moves calendar view one month forwards.
fn on_next_click(
    mut trigger: On<Pointer<Click>>,
    bind_query: Query<&BindToID, With<DatePickerNextButton>>,
    mut picker_query: Query<
        (&UIGenID, &mut DatePickerState, &mut UIWidgetState),
        With<DatePickerBase>,
    >,
    mut current_widget_state: ResMut<CurrentWidgetState>,
) {
    let Ok(bind) = bind_query.get(trigger.entity) else {
        return;
    };

    for (id, mut state, mut ui_state) in picker_query.iter_mut() {
        if id.get() != bind.0 {
            continue;
        }
        if ui_state.disabled {
            trigger.propagate(false);
            return;
        }

        let (year, month) = shift_month(state.view_year, state.view_month, 1);
        state.view_year = year;
        state.view_month = month;
        state.month_list_open = false;
        state.year_list_open = false;
        state.year_list_centered = false;
        ui_state.focused = true;
        ui_state.open = true;
        ui_state.checked = true;
        current_widget_state.widget_id = id.get();
        break;
    }

    trigger.propagate(false);
}

/// Toggles the month picker list.
fn on_month_toggle_click(
    mut trigger: On<Pointer<Click>>,
    bind_query: Query<&BindToID, With<DatePickerHeaderMonthButton>>,
    mut picker_query: Query<
        (&UIGenID, &mut DatePickerState, &mut UIWidgetState),
        With<DatePickerBase>,
    >,
    mut current_widget_state: ResMut<CurrentWidgetState>,
) {
    let Ok(bind) = bind_query.get(trigger.entity) else {
        return;
    };

    for (id, mut state, mut ui_state) in picker_query.iter_mut() {
        if id.get() != bind.0 {
            continue;
        }
        if ui_state.disabled {
            trigger.propagate(false);
            return;
        }

        state.month_list_open = !state.month_list_open;
        state.year_list_open = false;
        state.year_list_centered = false;
        ui_state.focused = true;
        ui_state.open = true;
        ui_state.checked = true;
        current_widget_state.widget_id = id.get();
        break;
    }

    trigger.propagate(false);
}

/// Toggles the scrollable year picker list.
fn on_year_toggle_click(
    mut trigger: On<Pointer<Click>>,
    bind_query: Query<&BindToID, With<DatePickerHeaderYearButton>>,
    mut picker_query: Query<
        (&UIGenID, &mut DatePickerState, &mut UIWidgetState),
        With<DatePickerBase>,
    >,
    mut current_widget_state: ResMut<CurrentWidgetState>,
) {
    let Ok(bind) = bind_query.get(trigger.entity) else {
        return;
    };

    for (id, mut state, mut ui_state) in picker_query.iter_mut() {
        if id.get() != bind.0 {
            continue;
        }
        if ui_state.disabled {
            trigger.propagate(false);
            return;
        }

        state.year_list_open = !state.year_list_open;
        state.month_list_open = false;
        if state.year_list_open {
            state.year_list_centered = false;
        } else {
            state.year_list_centered = false;
        }
        ui_state.focused = true;
        ui_state.open = true;
        ui_state.checked = true;
        current_widget_state.widget_id = id.get();
        break;
    }

    trigger.propagate(false);
}

/// Selects a month from the month list.
fn on_month_click(
    mut trigger: On<Pointer<Click>>,
    month_query: Query<(&BindToID, &DatePickerMonthOption), With<DatePickerMonthOption>>,
    parent_query: Query<&ChildOf>,
    mut picker_query: Query<
        (
            &UIGenID,
            &mut DatePicker,
            &mut DatePickerState,
            &mut UIWidgetState,
            &mut InputValue,
        ),
        (With<DatePickerBase>, Without<InputField>),
    >,
    mut input_query: Query<
        (&CssID, &UIGenID, &mut InputField, &mut InputValue),
        (With<InputField>, Without<DatePickerBase>),
    >,
    mut current_widget_state: ResMut<CurrentWidgetState>,
) {
    let Some((bind_id, month_value)) =
        resolve_month_option_from_entity(trigger.entity, &month_query, &parent_query)
    else {
        return;
    };

    for (id, mut picker, mut state, mut ui_state, mut picker_value) in picker_query.iter_mut() {
        if id.get() != bind_id {
            continue;
        }
        if ui_state.disabled {
            trigger.propagate(false);
            return;
        }
        if !(1..=12).contains(&month_value) {
            trigger.propagate(false);
            return;
        }
        if !is_month_allowed(state.view_year, month_value, state.min, state.max) {
            trigger.propagate(false);
            return;
        }

        state.view_month = month_value;
        state.month_list_open = false;
        state.year_list_open = false;
        state.year_list_centered = false;

        let mut candidate = if let Some(selected) = state.selected {
            SimpleDate {
                year: state.view_year,
                month: month_value,
                day: selected.day,
            }
        } else {
            SimpleDate {
                year: state.view_year,
                month: month_value,
                day: 1,
            }
        };
        candidate.day = candidate
            .day
            .min(days_in_month(candidate.year, candidate.month));
        let updated_date = clamp_date_to_bounds(candidate, state.min, state.max);
        let date = updated_date.unwrap_or(candidate);

        let mut selection_mode = PickerSelectionMode::Single;
        let mut effective_pattern = resolve_date_pattern(&picker, None);
        if let Some(for_id) = picker.for_id.as_ref() {
            for (css_id, _target_id, field, _field_value) in input_query.iter_mut() {
                if css_id.0 != *for_id {
                    continue;
                }
                if input_supports_date_picker(field.input_type) {
                    effective_pattern = resolve_date_pattern(&picker, Some(&field));
                    selection_mode = resolve_selection_mode(Some(&field));
                }
                break;
            }
        }

        if selection_mode == PickerSelectionMode::Range {
            ui_state.focused = true;
            ui_state.open = true;
            ui_state.checked = true;
            current_widget_state.widget_id = id.get();
            break;
        }

        state.selected = Some(date);
        state.view_year = date.year;
        state.view_month = date.month;
        let formatted = format_for_display(date, effective_pattern);
        picker.value = formatted.clone();
        picker_value.0 = formatted.clone();
        state.pending_bound_write_back = true;

        let mut bound_target_widget_id: Option<usize> = None;
        if let Some(for_id) = picker.for_id.as_ref() {
            for (css_id, target_id, mut field, mut field_value) in input_query.iter_mut() {
                if css_id.0 != *for_id {
                    continue;
                }
                if !input_supports_date_picker(field.input_type) {
                    break;
                }
                field.text = formatted.clone();
                field_value.0 = formatted.clone();
                bound_target_widget_id = Some(target_id.get());
                break;
            }
        }

        ui_state.focused = true;
        ui_state.open = true;
        ui_state.checked = true;
        current_widget_state.widget_id = bound_target_widget_id.unwrap_or(id.get());
        break;
    }

    trigger.propagate(false);
}

/// Selects a year from the scrollable year list.
fn on_year_click(
    mut trigger: On<Pointer<Click>>,
    year_query: Query<(&BindToID, &DatePickerYearOption), With<DatePickerYearOption>>,
    parent_query: Query<&ChildOf>,
    mut picker_query: Query<
        (
            &UIGenID,
            &mut DatePicker,
            &mut DatePickerState,
            &mut UIWidgetState,
            &mut InputValue,
        ),
        (With<DatePickerBase>, Without<InputField>),
    >,
    mut input_query: Query<
        (&CssID, &UIGenID, &mut InputField, &mut InputValue),
        (With<InputField>, Without<DatePickerBase>),
    >,
    mut current_widget_state: ResMut<CurrentWidgetState>,
) {
    let Some((bind_id, year_value)) =
        resolve_year_option_from_entity(trigger.entity, &year_query, &parent_query)
    else {
        return;
    };

    for (id, mut picker, mut state, mut ui_state, mut picker_value) in picker_query.iter_mut() {
        if id.get() != bind_id {
            continue;
        }
        if ui_state.disabled {
            trigger.propagate(false);
            return;
        }
        if year_value < state.year_start || year_value > state.year_end {
            trigger.propagate(false);
            return;
        }

        state.view_year = year_value;
        state.month_list_open = false;
        state.year_list_open = false;
        state.year_list_centered = false;

        let mut candidate = if let Some(selected) = state.selected {
            SimpleDate {
                year: year_value,
                month: selected.month,
                day: selected.day,
            }
        } else {
            SimpleDate {
                year: year_value,
                month: state.view_month,
                day: 1,
            }
        };
        candidate.day = candidate
            .day
            .min(days_in_month(candidate.year, candidate.month));
        let updated_date = clamp_date_to_bounds(candidate, state.min, state.max);

        let date = updated_date.unwrap_or(candidate);
        let mut selection_mode = PickerSelectionMode::Single;
        let mut effective_pattern = resolve_date_pattern(&picker, None);
        if let Some(for_id) = picker.for_id.as_ref() {
            for (css_id, _target_id, field, _field_value) in input_query.iter_mut() {
                if css_id.0 != *for_id {
                    continue;
                }
                if input_supports_date_picker(field.input_type) {
                    effective_pattern = resolve_date_pattern(&picker, Some(&field));
                    selection_mode = resolve_selection_mode(Some(&field));
                }
                break;
            }
        }

        if selection_mode == PickerSelectionMode::Range {
            ui_state.focused = true;
            ui_state.open = true;
            ui_state.checked = true;
            current_widget_state.widget_id = id.get();
            break;
        }

        state.selected = Some(date);
        state.view_year = date.year;
        state.view_month = date.month;
        let formatted = format_for_display(date, effective_pattern);
        picker.value = formatted.clone();
        picker_value.0 = formatted.clone();
        state.pending_bound_write_back = true;

        let mut bound_target_widget_id: Option<usize> = None;
        if let Some(for_id) = picker.for_id.as_ref() {
            for (css_id, target_id, mut field, mut field_value) in input_query.iter_mut() {
                if css_id.0 != *for_id {
                    continue;
                }
                if !input_supports_date_picker(field.input_type) {
                    break;
                }
                field.text = formatted.clone();
                field_value.0 = formatted.clone();
                bound_target_widget_id = Some(target_id.get());
                break;
            }
        }

        ui_state.focused = true;
        ui_state.open = true;
        ui_state.checked = true;
        current_widget_state.widget_id = bound_target_widget_id.unwrap_or(id.get());
        break;
    }

    trigger.propagate(false);
}

/// Fallback month selection when the click lands on the month-list container.
fn on_month_list_click(
    mut trigger: On<Pointer<Click>>,
    month_list_query: Query<&BindToID, With<DatePickerMonthList>>,
    month_query: Query<(&BindToID, &DatePickerMonthOption), With<DatePickerMonthOption>>,
    parent_query: Query<&ChildOf>,
    month_option_query: Query<(&DatePickerMonthOption, &BindToID, &RelativeCursorPosition)>,
    mut picker_query: Query<
        (
            &UIGenID,
            &mut DatePicker,
            &mut DatePickerState,
            &mut UIWidgetState,
            &mut InputValue,
        ),
        (With<DatePickerBase>, Without<InputField>),
    >,
    mut input_query: Query<
        (&CssID, &UIGenID, &mut InputField, &mut InputValue),
        (With<InputField>, Without<DatePickerBase>),
    >,
    mut current_widget_state: ResMut<CurrentWidgetState>,
) {
    if resolve_month_option_from_entity(trigger.event_target(), &month_query, &parent_query)
        .is_some()
    {
        trigger.propagate(false);
        return;
    }

    let Ok(bind) = month_list_query.get(trigger.entity) else {
        return;
    };

    let mut hovered_month: Option<u32> = None;
    for (option, option_bind, rel) in month_option_query.iter() {
        if option_bind.0 != bind.0 {
            continue;
        }
        if rel.cursor_over() {
            hovered_month = Some(option.month);
            break;
        }
    }

    let Some(month_value) = hovered_month else {
        trigger.propagate(false);
        return;
    };

    for (id, mut picker, mut state, mut ui_state, mut picker_value) in picker_query.iter_mut() {
        if id.get() != bind.0 {
            continue;
        }
        if ui_state.disabled || !state.month_list_open {
            trigger.propagate(false);
            return;
        }

        let month_value = month_value.clamp(1, 12);
        if !is_month_allowed(state.view_year, month_value, state.min, state.max) {
            trigger.propagate(false);
            return;
        }
        state.view_month = month_value;
        state.month_list_open = false;
        state.year_list_open = false;
        state.year_list_centered = false;

        let mut candidate = if let Some(selected) = state.selected {
            SimpleDate {
                year: state.view_year,
                month: month_value,
                day: selected.day,
            }
        } else {
            SimpleDate {
                year: state.view_year,
                month: month_value,
                day: 1,
            }
        };
        candidate.day = candidate
            .day
            .min(days_in_month(candidate.year, candidate.month));
        let updated_date = clamp_date_to_bounds(candidate, state.min, state.max);
        let date = updated_date.unwrap_or(candidate);

        let mut selection_mode = PickerSelectionMode::Single;
        let mut effective_pattern = resolve_date_pattern(&picker, None);
        if let Some(for_id) = picker.for_id.as_ref() {
            for (css_id, _target_id, field, _field_value) in input_query.iter_mut() {
                if css_id.0 != *for_id {
                    continue;
                }
                if input_supports_date_picker(field.input_type) {
                    effective_pattern = resolve_date_pattern(&picker, Some(&field));
                    selection_mode = resolve_selection_mode(Some(&field));
                }
                break;
            }
        }

        if selection_mode == PickerSelectionMode::Range {
            ui_state.focused = true;
            ui_state.open = true;
            ui_state.checked = true;
            current_widget_state.widget_id = id.get();
            break;
        }

        state.selected = Some(date);
        state.view_year = date.year;
        state.view_month = date.month;
        let formatted = format_for_display(date, effective_pattern);
        picker.value = formatted.clone();
        picker_value.0 = formatted.clone();
        state.pending_bound_write_back = true;

        let mut bound_target_widget_id: Option<usize> = None;
        if let Some(for_id) = picker.for_id.as_ref() {
            for (css_id, target_id, mut field, mut field_value) in input_query.iter_mut() {
                if css_id.0 != *for_id {
                    continue;
                }
                if !input_supports_date_picker(field.input_type) {
                    break;
                }
                field.text = formatted.clone();
                field_value.0 = formatted.clone();
                bound_target_widget_id = Some(target_id.get());
                break;
            }
        }

        ui_state.focused = true;
        ui_state.open = true;
        ui_state.checked = true;
        current_widget_state.widget_id = bound_target_widget_id.unwrap_or(id.get());
        break;
    }

    trigger.propagate(false);
}

/// Fallback year selection when the click lands on the year-list container.
fn on_year_list_click(
    mut trigger: On<Pointer<Click>>,
    year_list_query: Query<&BindToID, With<DatePickerYearList>>,
    year_query: Query<(&BindToID, &DatePickerYearOption), With<DatePickerYearOption>>,
    parent_query: Query<&ChildOf>,
    year_option_query: Query<(&DatePickerYearOption, &BindToID, &RelativeCursorPosition)>,
    mut picker_query: Query<
        (
            &UIGenID,
            &mut DatePicker,
            &mut DatePickerState,
            &mut UIWidgetState,
            &mut InputValue,
        ),
        (With<DatePickerBase>, Without<InputField>),
    >,
    mut input_query: Query<
        (&CssID, &UIGenID, &mut InputField, &mut InputValue),
        (With<InputField>, Without<DatePickerBase>),
    >,
    mut current_widget_state: ResMut<CurrentWidgetState>,
) {
    if resolve_year_option_from_entity(trigger.event_target(), &year_query, &parent_query).is_some()
    {
        trigger.propagate(false);
        return;
    }

    let Ok(bind) = year_list_query.get(trigger.entity) else {
        return;
    };

    let mut hovered_year: Option<i32> = None;
    for (option, option_bind, rel) in year_option_query.iter() {
        if option_bind.0 != bind.0 {
            continue;
        }
        if rel.cursor_over() {
            hovered_year = Some(option.year);
            break;
        }
    }

    let Some(year_value) = hovered_year else {
        trigger.propagate(false);
        return;
    };

    for (id, mut picker, mut state, mut ui_state, mut picker_value) in picker_query.iter_mut() {
        if id.get() != bind.0 {
            continue;
        }
        if ui_state.disabled || !state.year_list_open {
            trigger.propagate(false);
            return;
        }

        let year_value = year_value.clamp(state.year_start, state.year_end);

        state.view_year = year_value;
        state.month_list_open = false;
        state.year_list_open = false;
        state.year_list_centered = false;

        let mut candidate = if let Some(selected) = state.selected {
            SimpleDate {
                year: year_value,
                month: selected.month,
                day: selected.day,
            }
        } else {
            SimpleDate {
                year: year_value,
                month: state.view_month,
                day: 1,
            }
        };
        candidate.day = candidate
            .day
            .min(days_in_month(candidate.year, candidate.month));
        let updated_date = clamp_date_to_bounds(candidate, state.min, state.max);
        let date = updated_date.unwrap_or(candidate);
        let mut selection_mode = PickerSelectionMode::Single;
        let mut effective_pattern = resolve_date_pattern(&picker, None);
        if let Some(for_id) = picker.for_id.as_ref() {
            for (css_id, _target_id, field, _field_value) in input_query.iter_mut() {
                if css_id.0 != *for_id {
                    continue;
                }
                if input_supports_date_picker(field.input_type) {
                    effective_pattern = resolve_date_pattern(&picker, Some(&field));
                    selection_mode = resolve_selection_mode(Some(&field));
                }
                break;
            }
        }

        if selection_mode == PickerSelectionMode::Range {
            ui_state.focused = true;
            ui_state.open = true;
            ui_state.checked = true;
            current_widget_state.widget_id = id.get();
            break;
        }

        state.selected = Some(date);
        state.view_year = date.year;
        state.view_month = date.month;
        let formatted = format_for_display(date, effective_pattern);
        picker.value = formatted.clone();
        picker_value.0 = formatted.clone();
        state.pending_bound_write_back = true;

        let mut bound_target_widget_id: Option<usize> = None;
        if let Some(for_id) = picker.for_id.as_ref() {
            for (css_id, target_id, mut field, mut field_value) in input_query.iter_mut() {
                if css_id.0 != *for_id {
                    continue;
                }
                if !input_supports_date_picker(field.input_type) {
                    break;
                }
                field.text = formatted.clone();
                field_value.0 = formatted.clone();
                bound_target_widget_id = Some(target_id.get());
                break;
            }
        }

        ui_state.focused = true;
        ui_state.open = true;
        ui_state.checked = true;
        current_widget_state.widget_id = bound_target_widget_id.unwrap_or(id.get());
        break;
    }

    trigger.propagate(false);
}

/// Selects a day from the visible calendar grid.
fn on_day_click(
    mut trigger: On<Pointer<Click>>,
    mut commands: Commands,
    day_query: Query<(&BindToID, &DatePickerDayButton), With<DatePickerDayButton>>,
    mut picker_query: Query<
        (
            &UIGenID,
            &mut DatePicker,
            &mut DatePickerState,
            &mut UIWidgetState,
            &mut InputValue,
        ),
        With<DatePickerBase>,
    >,
    mut input_query: Query<
        (Entity, &CssID, &UIGenID, &mut InputField, &mut InputValue),
        (With<InputField>, Without<DatePickerBase>),
    >,
    mut current_widget_state: ResMut<CurrentWidgetState>,
) {
    let Ok((bind, day_button)) = day_query.get(trigger.entity) else {
        return;
    };

    for (id, mut picker, mut state, mut ui_state, mut input_value) in picker_query.iter_mut() {
        if id.get() != bind.0 {
            continue;
        }
        if ui_state.disabled {
            trigger.propagate(false);
            return;
        }

        let cells = build_calendar_cells(state.view_year, state.view_month);
        let visible_rows = visible_calendar_row_count(state.view_year, state.view_month);
        let visible_cell_count = visible_rows * 7;
        if day_button.index >= visible_cell_count {
            trigger.propagate(false);
            return;
        }
        let Some(cell) = cells.get(day_button.index).copied() else {
            break;
        };

        if !is_date_allowed(cell.date, state.min, state.max) {
            trigger.propagate(false);
            return;
        }

        let mut selection_mode = PickerSelectionMode::Single;
        let mut effective_pattern = resolve_date_pattern(&picker, None);
        if let Some(for_id) = picker.for_id.as_ref() {
            for (_, css_id, _target_id, field, _field_value) in input_query.iter_mut() {
                if css_id.0 != *for_id {
                    continue;
                }
                if input_supports_date_picker(field.input_type) {
                    effective_pattern = resolve_date_pattern(&picker, Some(&field));
                    selection_mode = resolve_selection_mode(Some(&field));
                }
                break;
            }
        }

        state.view_year = cell.date.year;
        state.view_month = cell.date.month;
        state.month_list_open = false;
        state.year_list_open = false;
        state.year_list_centered = false;

        let mut keep_open = false;
        let formatted = match selection_mode {
            PickerSelectionMode::Single => {
                state.selected = Some(cell.date);
                state.range_start = None;
                state.range_end = None;
                format_for_display(cell.date, effective_pattern)
            }
            PickerSelectionMode::Range => {
                keep_open = true;
                match (state.range_start, state.range_end) {
                    (None, _) | (Some(_), Some(_)) => {
                        state.range_start = Some(cell.date);
                        state.range_end = None;
                        state.selected = Some(cell.date);
                    }
                    (Some(start), None) => {
                        if cell.date < start {
                            state.range_start = Some(cell.date);
                            state.range_end = Some(start);
                            state.selected = Some(start);
                        } else {
                            state.range_start = Some(start);
                            state.range_end = Some(cell.date);
                            state.selected = Some(cell.date);
                        }
                    }
                }
                format_range_for_display(state.range_start, state.range_end, effective_pattern)
            }
        };
        picker.value = formatted.clone();
        input_value.0 = formatted.clone();
        state.pending_bound_write_back = true;

        let mut bound_target_widget_id: Option<usize> = None;
        if let Some(for_id) = picker.for_id.as_ref() {
            for (target_entity, css_id, target_id, mut field, mut field_value) in
                input_query.iter_mut()
            {
                if css_id.0 != *for_id {
                    continue;
                }
                if !input_supports_date_picker(field.input_type) {
                    break;
                }

                field.text = formatted.clone();
                field_value.0 = formatted.clone();
                commands.entity(target_entity).insert(InputUserChanged);
                bound_target_widget_id = Some(target_id.get());
                break;
            }
        }

        ui_state.focused = true;
        ui_state.open = keep_open;
        ui_state.checked = keep_open;
        current_widget_state.widget_id = bound_target_widget_id.unwrap_or(id.get());
        break;
    }

    trigger.propagate(false);
}

/// Sets hovered state when entering the date picker root.
fn on_internal_cursor_entered(
    mut trigger: On<Pointer<Over>>,
    mut query: Query<&mut UIWidgetState, With<DatePickerBase>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.entity) {
        state.hovered = true;
    }
    trigger.propagate(false);
}

/// Clears hovered state when leaving the date picker root.
fn on_internal_cursor_leave(
    mut trigger: On<Pointer<Out>>,
    mut query: Query<&mut UIWidgetState, With<DatePickerBase>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.entity) {
        state.hovered = false;
    }
    trigger.propagate(false);
}

/// Sets hovered state on the bound root while inside the field.
fn on_field_cursor_entered(
    mut trigger: On<Pointer<Over>>,
    field_query: Query<&BindToID, With<DatePickerField>>,
    mut picker_query: Query<(&UIGenID, &mut UIWidgetState), With<DatePickerBase>>,
) {
    let Ok(bind) = field_query.get(trigger.entity) else {
        return;
    };
    for (id, mut state) in picker_query.iter_mut() {
        if id.get() == bind.0 {
            state.hovered = true;
            break;
        }
    }
    trigger.propagate(false);
}

/// Clears hovered state on the bound root when leaving the field.
fn on_field_cursor_leave(
    mut trigger: On<Pointer<Out>>,
    field_query: Query<&BindToID, With<DatePickerField>>,
    mut picker_query: Query<(&UIGenID, &mut UIWidgetState), With<DatePickerBase>>,
) {
    let Ok(bind) = field_query.get(trigger.entity) else {
        return;
    };
    for (id, mut state) in picker_query.iter_mut() {
        if id.get() == bind.0 {
            state.hovered = false;
            break;
        }
    }
    trigger.propagate(false);
}

/// Sets hovered state when entering a nav button.
fn on_nav_cursor_entered(
    mut trigger: On<Pointer<Over>>,
    mut query: Query<
        &mut UIWidgetState,
        Or<(
            With<DatePickerPrevButton>,
            With<DatePickerNextButton>,
            With<DatePickerHeaderMonthButton>,
            With<DatePickerHeaderYearButton>,
        )>,
    >,
) {
    if let Ok(mut state) = query.get_mut(trigger.entity) {
        state.hovered = true;
    }
    trigger.propagate(false);
}

/// Clears hovered state when leaving a nav button.
fn on_nav_cursor_leave(
    mut trigger: On<Pointer<Out>>,
    mut query: Query<
        &mut UIWidgetState,
        Or<(
            With<DatePickerPrevButton>,
            With<DatePickerNextButton>,
            With<DatePickerHeaderMonthButton>,
            With<DatePickerHeaderYearButton>,
        )>,
    >,
) {
    if let Ok(mut state) = query.get_mut(trigger.entity) {
        state.hovered = false;
    }
    trigger.propagate(false);
}

/// Sets hovered state for year entries.
fn on_year_cursor_entered(
    mut trigger: On<Pointer<Over>>,
    mut query: Query<&mut UIWidgetState, With<DatePickerYearOption>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.entity) {
        if !state.disabled {
            state.hovered = true;
        }
    }
    trigger.propagate(false);
}

/// Clears hovered state for year entries.
fn on_year_cursor_leave(
    mut trigger: On<Pointer<Out>>,
    mut query: Query<&mut UIWidgetState, With<DatePickerYearOption>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.entity) {
        state.hovered = false;
    }
    trigger.propagate(false);
}

/// Sets hovered state for month entries.
fn on_month_cursor_entered(
    mut trigger: On<Pointer<Over>>,
    mut query: Query<&mut UIWidgetState, With<DatePickerMonthOption>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.entity) {
        if !state.disabled {
            state.hovered = true;
        }
    }
    trigger.propagate(false);
}

/// Clears hovered state for month entries.
fn on_month_cursor_leave(
    mut trigger: On<Pointer<Out>>,
    mut query: Query<&mut UIWidgetState, With<DatePickerMonthOption>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.entity) {
        state.hovered = false;
    }
    trigger.propagate(false);
}

/// Sets hovered state for day cells.
fn on_day_cursor_entered(
    mut trigger: On<Pointer<Over>>,
    mut query: Query<&mut UIWidgetState, With<DatePickerDayButton>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.entity) {
        if !state.disabled && !state.readonly {
            state.hovered = true;
        }
    }
    trigger.propagate(false);
}

/// Clears hovered state for day cells.
fn on_day_cursor_leave(
    mut trigger: On<Pointer<Out>>,
    mut query: Query<&mut UIWidgetState, With<DatePickerDayButton>>,
) {
    if let Ok(mut state) = query.get_mut(trigger.entity) {
        state.hovered = false;
    }
    trigger.propagate(false);
}

/// Handles `resolve_date_pattern` in the extended UI workflow.
fn resolve_date_pattern(picker: &DatePicker, bound_input: Option<&InputField>) -> DatePattern {
    if let Some(spec) = picker
        .format_pattern
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        if let Some(pattern) = parse_date_pattern(spec) {
            return pattern;
        }
    }

    if let Some(input) = bound_input {
        if input_supports_date_picker(input.input_type) {
            if let Some(spec) = input
                .date_format
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
            {
                if let Some(pattern) = parse_date_pattern(spec) {
                    return pattern;
                }
            }
        }
    }

    pattern_from_date_format(picker.format)
}

/// Handles `input_supports_date_picker` in the extended UI workflow.
fn input_supports_date_picker(input_type: InputType) -> bool {
    matches!(input_type, InputType::Date | InputType::Range)
}

/// Handles `resolve_selection_mode` in the extended UI workflow.
fn resolve_selection_mode(bound_input: Option<&InputField>) -> PickerSelectionMode {
    if bound_input.is_some_and(|input| input.input_type == InputType::Range) {
        PickerSelectionMode::Range
    } else {
        PickerSelectionMode::Single
    }
}

/// Handles `default_separator_for_order` in the extended UI workflow.
fn default_separator_for_order(order: DateFieldOrder) -> char {
    match order {
        DateFieldOrder::MonthDayYear | DateFieldOrder::DayMonthYear => '/',
        DateFieldOrder::YearMonthDay => '-',
    }
}

/// Handles `pattern_from_date_format` in the extended UI workflow.
fn pattern_from_date_format(format: DateFormat) -> DatePattern {
    match format {
        DateFormat::MonthDayYear => DatePattern {
            order: DateFieldOrder::MonthDayYear,
            separator: '/',
        },
        DateFormat::DayMonthYear => DatePattern {
            order: DateFieldOrder::DayMonthYear,
            separator: '/',
        },
        DateFormat::YearMonthDay => DatePattern {
            order: DateFieldOrder::YearMonthDay,
            separator: '-',
        },
    }
}

/// Handles `parse_date_pattern` in the extended UI workflow.
fn parse_date_pattern(spec: &str) -> Option<DatePattern> {
    let trimmed = spec.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Some(format) = DateFormat::from_str(trimmed) {
        let mut pattern = pattern_from_date_format(format);
        if let Some(separator) = detect_separator(trimmed) {
            pattern.separator = separator;
        }
        return Some(pattern);
    }

    let normalized = trimmed.to_ascii_lowercase();
    let mut parts: Vec<String> = Vec::new();
    let mut separators: Vec<char> = Vec::new();
    let mut current = String::new();

    for ch in normalized.chars() {
        if ch.is_ascii_alphabetic() {
            current.push(ch);
            continue;
        }

        if ch == '/' || ch == '-' || ch == '.' {
            if current.is_empty() {
                return None;
            }
            parts.push(std::mem::take(&mut current));
            separators.push(ch);
            continue;
        }

        if !ch.is_whitespace() {
            return None;
        }
    }

    if !current.is_empty() {
        parts.push(current);
    }

    if parts.len() != 3 {
        return None;
    }

    if !separators.is_empty() && !separators.iter().all(|sep| *sep == separators[0]) {
        return None;
    }

    let mut fields: [char; 3] = [' '; 3];
    for (idx, part) in parts.iter().enumerate() {
        let token = part.chars().next()?;
        if token != 'd' && token != 'm' && token != 'y' {
            return None;
        }
        fields[idx] = token;
    }

    if !(fields.contains(&'d') && fields.contains(&'m') && fields.contains(&'y')) {
        return None;
    }

    let order = match fields {
        ['m', 'd', 'y'] => DateFieldOrder::MonthDayYear,
        ['d', 'm', 'y'] => DateFieldOrder::DayMonthYear,
        ['y', 'm', 'd'] => DateFieldOrder::YearMonthDay,
        _ => return None,
    };

    Some(DatePattern {
        order,
        separator: separators
            .first()
            .copied()
            .unwrap_or_else(|| default_separator_for_order(order)),
    })
}

/// Handles `detect_separator` in the extended UI workflow.
fn detect_separator(value: &str) -> Option<char> {
    value
        .chars()
        .find(|ch| *ch == '/' || *ch == '-' || *ch == '.')
}

/// Handles `default_placeholder_for_format` in the extended UI workflow.
fn default_placeholder_for_format(pattern: DatePattern) -> String {
    let sep = pattern.separator;
    match pattern.order {
        DateFieldOrder::MonthDayYear => format!("MM{sep}DD{sep}YYYY"),
        DateFieldOrder::DayMonthYear => format!("DD{sep}MM{sep}YYYY"),
        DateFieldOrder::YearMonthDay => format!("YYYY{sep}MM{sep}DD"),
    }
}

/// Handles `format_for_display` in the extended UI workflow.
fn format_for_display(date: SimpleDate, pattern: DatePattern) -> String {
    let sep = pattern.separator;
    match pattern.order {
        DateFieldOrder::MonthDayYear => {
            format!("{:02}{sep}{:02}{sep}{:04}", date.month, date.day, date.year)
        }
        DateFieldOrder::DayMonthYear => {
            format!("{:02}{sep}{:02}{sep}{:04}", date.day, date.month, date.year)
        }
        DateFieldOrder::YearMonthDay => {
            format!("{:04}{sep}{:02}{sep}{:02}", date.year, date.month, date.day)
        }
    }
}

/// Handles `format_range_for_display` in the extended UI workflow.
fn format_range_for_display(
    start: Option<SimpleDate>,
    end: Option<SimpleDate>,
    pattern: DatePattern,
) -> String {
    match (start, end) {
        (Some(start), Some(end)) => {
            format!(
                "{} - {}",
                format_for_display(start, pattern),
                format_for_display(end, pattern)
            )
        }
        (Some(start), None) => format_for_display(start, pattern),
        _ => String::new(),
    }
}

/// Handles `parse_picker_range` in the extended UI workflow.
fn parse_picker_range(
    value: &str,
    pattern: DatePattern,
) -> Option<(SimpleDate, Option<SimpleDate>)> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Some((left, right)) = split_range_parts(trimmed) {
        let mut start = parse_picker_date(left, pattern)?;
        let mut end = if right.is_empty() {
            None
        } else {
            parse_picker_date(right, pattern)
        };
        if let Some(end_date) = end {
            if end_date < start {
                start = end_date;
                end = Some(parse_picker_date(left, pattern)?);
            }
        }
        return Some((start, end));
    }

    parse_picker_date(trimmed, pattern).map(|single| (single, None))
}

/// Handles `split_range_parts` in the extended UI workflow.
fn split_range_parts(value: &str) -> Option<(&str, &str)> {
    for delimiter in [" - ", " – ", " — "] {
        if let Some((left, right)) = value.split_once(delimiter) {
            return Some((left.trim(), right.trim()));
        }
    }
    None
}

/// Handles `normalize_bound_value_for_mode` in the extended UI workflow.
fn normalize_bound_value_for_mode(
    value: &str,
    pattern: DatePattern,
    mode: PickerSelectionMode,
) -> String {
    match mode {
        PickerSelectionMode::Single => parse_picker_date(value, pattern)
            .map(|date| format_for_display(date, pattern))
            .unwrap_or_else(|| value.to_string()),
        PickerSelectionMode::Range => parse_picker_range(value, pattern)
            .map(|(start, end)| format_range_for_display(Some(start), end, pattern))
            .unwrap_or_else(|| value.to_string()),
    }
}

/// Handles `in_selected_range` in the extended UI workflow.
fn in_selected_range(date: SimpleDate, start: Option<SimpleDate>, end: Option<SimpleDate>) -> bool {
    let Some(start) = start else {
        return false;
    };
    let Some(end) = end else {
        return false;
    };
    if start <= end {
        date >= start && date <= end
    } else {
        date >= end && date <= start
    }
}

/// Handles `parse_picker_date` in the extended UI workflow.
fn parse_picker_date(value: &str, pattern: DatePattern) -> Option<SimpleDate> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    parse_iso_date(trimmed)
        .or_else(|| parse_date_with_pattern(trimmed, pattern))
        .or_else(|| {
            parse_date_with_pattern(
                trimmed,
                DatePattern {
                    order: DateFieldOrder::MonthDayYear,
                    separator: detect_separator(trimmed).unwrap_or('/'),
                },
            )
        })
        .or_else(|| {
            parse_date_with_pattern(
                trimmed,
                DatePattern {
                    order: DateFieldOrder::DayMonthYear,
                    separator: detect_separator(trimmed).unwrap_or('/'),
                },
            )
        })
        .or_else(|| {
            parse_date_with_pattern(
                trimmed,
                DatePattern {
                    order: DateFieldOrder::YearMonthDay,
                    separator: detect_separator(trimmed).unwrap_or('-'),
                },
            )
        })
}

/// Handles `resolve_year_option_from_entity` in the extended UI workflow.
fn resolve_year_option_from_entity(
    start: Entity,
    year_query: &Query<(&BindToID, &DatePickerYearOption), With<DatePickerYearOption>>,
    parent_query: &Query<&ChildOf>,
) -> Option<(usize, i32)> {
    let mut current = Some(start);
    while let Some(entity) = current {
        if let Ok((bind, year_option)) = year_query.get(entity) {
            return Some((bind.0, year_option.year));
        }
        current = parent_query.get(entity).ok().map(|parent| parent.parent());
    }
    None
}

/// Handles `resolve_month_option_from_entity` in the extended UI workflow.
fn resolve_month_option_from_entity(
    start: Entity,
    month_query: &Query<(&BindToID, &DatePickerMonthOption), With<DatePickerMonthOption>>,
    parent_query: &Query<&ChildOf>,
) -> Option<(usize, u32)> {
    let mut current = Some(start);
    while let Some(entity) = current {
        if let Ok((bind, month_option)) = month_query.get(entity) {
            return Some((bind.0, month_option.month));
        }
        current = parent_query.get(entity).ok().map(|parent| parent.parent());
    }
    None
}

/// Handles `parse_iso_date` in the extended UI workflow.
fn parse_iso_date(value: &str) -> Option<SimpleDate> {
    let mut parts = value.trim().split('-');
    let year = parts.next()?.trim().parse::<i32>().ok()?;
    let month = parts.next()?.trim().parse::<u32>().ok()?;
    let day = parts.next()?.trim().parse::<u32>().ok()?;
    if parts.next().is_some() {
        return None;
    }
    make_date(year, month, day)
}

/// Handles `parse_date_with_pattern` in the extended UI workflow.
fn parse_date_with_pattern(value: &str, pattern: DatePattern) -> Option<SimpleDate> {
    let tokens: Vec<&str> = value
        .trim()
        .split(|c| c == '/' || c == '-' || c == '.')
        .collect();
    if tokens.len() != 3 {
        return None;
    }

    let p1 = tokens[0].trim();
    let p2 = tokens[1].trim();
    let p3 = tokens[2].trim();

    match pattern.order {
        DateFieldOrder::MonthDayYear => make_date(
            p3.parse::<i32>().ok()?,
            p1.parse::<u32>().ok()?,
            p2.parse::<u32>().ok()?,
        ),
        DateFieldOrder::DayMonthYear => make_date(
            p3.parse::<i32>().ok()?,
            p2.parse::<u32>().ok()?,
            p1.parse::<u32>().ok()?,
        ),
        DateFieldOrder::YearMonthDay => make_date(
            p1.parse::<i32>().ok()?,
            p2.parse::<u32>().ok()?,
            p3.parse::<u32>().ok()?,
        ),
    }
}

/// Handles `make_date` in the extended UI workflow.
fn make_date(year: i32, month: u32, day: u32) -> Option<SimpleDate> {
    if !(1..=12).contains(&month) {
        return None;
    }
    if day == 0 || day > days_in_month(year, month) {
        return None;
    }
    Some(SimpleDate { year, month, day })
}

/// Handles `is_leap_year` in the extended UI workflow.
fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// Handles `days_in_month` in the extended UI workflow.
fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 30,
    }
}

/// Handles `shift_month` in the extended UI workflow.
fn shift_month(year: i32, month: u32, delta: i32) -> (i32, u32) {
    let idx = year * 12 + (month as i32 - 1) + delta;
    let new_year = idx.div_euclid(12);
    let new_month = idx.rem_euclid(12) + 1;
    (new_year, new_month as u32)
}

/// Handles `day_of_week` in the extended UI workflow.
fn day_of_week(date: SimpleDate) -> u32 {
    let mut y = date.year;
    let mut m = date.month as i32;
    let d = date.day as i32;
    if m < 3 {
        y -= 1;
        m += 12;
    }
    let k = y % 100;
    let j = y / 100;
    let h = (d + ((13 * (m + 1)) / 5) + k + (k / 4) + (j / 4) + (5 * j)) % 7;
    ((h + 6) % 7) as u32
}

/// Handles `build_calendar_cells` in the extended UI workflow.
fn build_calendar_cells(year: i32, month: u32) -> Vec<CalendarCell> {
    let first_sunday_index = day_of_week(SimpleDate {
        year,
        month,
        day: 1,
    }) as usize;
    let first = (first_sunday_index + 6) % 7;
    let current_days = days_in_month(year, month) as usize;
    let (prev_year, prev_month) = shift_month(year, month, -1);
    let (next_year, next_month) = shift_month(year, month, 1);
    let prev_days = days_in_month(prev_year, prev_month) as usize;

    let mut cells = Vec::with_capacity(42);
    for idx in 0..42 {
        if idx < first {
            let day = prev_days - (first - idx) + 1;
            cells.push(CalendarCell {
                date: SimpleDate {
                    year: prev_year,
                    month: prev_month,
                    day: day as u32,
                },
                in_current_month: false,
            });
            continue;
        }

        let current_idx = idx - first;
        if current_idx < current_days {
            cells.push(CalendarCell {
                date: SimpleDate {
                    year,
                    month,
                    day: (current_idx + 1) as u32,
                },
                in_current_month: true,
            });
            continue;
        }

        let next_day = current_idx - current_days + 1;
        cells.push(CalendarCell {
            date: SimpleDate {
                year: next_year,
                month: next_month,
                day: next_day as u32,
            },
            in_current_month: false,
        });
    }

    cells
}

/// Handles `visible_calendar_row_count` in the extended UI workflow.
fn visible_calendar_row_count(year: i32, month: u32) -> usize {
    let first_sunday_index = day_of_week(SimpleDate {
        year,
        month,
        day: 1,
    }) as usize;
    let first = (first_sunday_index + 6) % 7;
    let current_days = days_in_month(year, month) as usize;
    let weeks = (first + current_days).div_ceil(7);
    weeks.max(5)
}

/// Handles `is_date_allowed` in the extended UI workflow.
fn is_date_allowed(date: SimpleDate, min: Option<SimpleDate>, max: Option<SimpleDate>) -> bool {
    if let Some(min) = min {
        if date < min {
            return false;
        }
    }
    if let Some(max) = max {
        if date > max {
            return false;
        }
    }
    true
}

/// Handles `clamp_date_to_bounds` in the extended UI workflow.
fn clamp_date_to_bounds(
    mut date: SimpleDate,
    min: Option<SimpleDate>,
    max: Option<SimpleDate>,
) -> Option<SimpleDate> {
    if let Some(min) = min {
        if date < min {
            date = min;
        }
    }
    if let Some(max) = max {
        if date > max {
            date = max;
        }
    }
    if is_date_allowed(date, min, max) {
        Some(date)
    } else {
        None
    }
}

/// Handles `month_name` in the extended UI workflow.
fn month_name(month: u32) -> &'static str {
    match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "Unknown",
    }
}

/// Handles `month_short_name` in the extended UI workflow.
fn month_short_name(month: u32) -> &'static str {
    match month {
        1 => "Jan",
        2 => "Feb",
        3 => "Mar",
        4 => "Apr",
        5 => "May",
        6 => "Jun",
        7 => "Jul",
        8 => "Aug",
        9 => "Sep",
        10 => "Oct",
        11 => "Nov",
        12 => "Dec",
        _ => "???",
    }
}

/// Handles `is_month_allowed` in the extended UI workflow.
fn is_month_allowed(
    year: i32,
    month: u32,
    min: Option<SimpleDate>,
    max: Option<SimpleDate>,
) -> bool {
    let Some(first) = make_date(year, month, 1) else {
        return false;
    };
    let Some(last) = make_date(year, month, days_in_month(year, month)) else {
        return false;
    };

    if let Some(min) = min {
        if last < min {
            return false;
        }
    }
    if let Some(max) = max {
        if first > max {
            return false;
        }
    }

    true
}

/// Handles `resolve_year_range` in the extended UI workflow.
fn resolve_year_range(
    min: Option<SimpleDate>,
    max: Option<SimpleDate>,
    anchor_year: i32,
) -> (i32, i32) {
    let mut start = min.map(|d| d.year).unwrap_or(1900);
    let mut end = max.map(|d| d.year).unwrap_or(2100);
    if start > end {
        std::mem::swap(&mut start, &mut end);
    }
    if anchor_year < start {
        start = anchor_year;
    }
    if anchor_year > end {
        end = anchor_year;
    }
    (start, end)
}

/// Handles `today_utc_date` in the extended UI workflow.
#[cfg(not(target_arch = "wasm32"))]
fn today_utc_date() -> SimpleDate {
    let Ok(duration) = SystemTime::now().duration_since(UNIX_EPOCH) else {
        return SimpleDate {
            year: 1970,
            month: 1,
            day: 1,
        };
    };

    let days_since_epoch = (duration.as_secs() / 86_400) as i64;
    let (year, month, day) = civil_from_days(days_since_epoch);
    SimpleDate { year, month, day }
}

/// Handles `today_utc_date` in the extended UI workflow.
#[cfg(target_arch = "wasm32")]
fn today_utc_date() -> SimpleDate {
    let ms = Date::now();
    if !ms.is_finite() {
        return SimpleDate {
            year: 1970,
            month: 1,
            day: 1,
        };
    }

    let secs = (ms / 1000.0).floor() as i64;
    let days_since_epoch = secs / 86_400;
    let (year, month, day) = civil_from_days(days_since_epoch);
    SimpleDate { year, month, day }
}

/// Handles `civil_from_days` in the extended UI workflow.
fn civil_from_days(days_since_epoch: i64) -> (i32, u32, u32) {
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let mut y = yoe as i32 + era as i32 * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + if mp < 10 { 3 } else { -9 };
    if m <= 2 {
        y += 1;
    }
    (y, m as u32, d as u32)
}
