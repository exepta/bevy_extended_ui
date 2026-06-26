use crate::ExtendedUiConfiguration;
use crate::html::{HtmlInnerContent, HtmlSystemSet};
use crate::old::registry::{TABLE_CELL_ID_POOL, TABLE_ID_POOL};
use crate::services::style_service::update_widget_styles_system;
use crate::styles::paint::Colored;
use crate::styles::{CssSource, TagName};
use crate::widgets::{UIGenID, UIWidgetState, Widget, WidgetId, WidgetKind};
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;

/// The `<table>` grid container.
///
/// `columns` is the auto-derived column count (the maximum number of cells in
/// any single row), used as the default grid template by `apply_table_grid_system`.
#[derive(Component, Reflect, Debug, Clone, PartialEq, Eq)]
#[reflect(Component)]
#[require(UIGenID, UIWidgetState, GlobalTransform, InheritedVisibility, Widget)]
pub struct Table {
    pub entry: usize,
    pub columns: usize,
}

impl Default for Table {
    fn default() -> Self {
        let entry = TABLE_ID_POOL.lock().unwrap().acquire();
        Self { entry, columns: 0 }
    }
}

/// The table section a cell belongs to (`<thead>`/`<tbody>`/`<tfoot>`).
///
/// Section wrappers are flattened at parse time, so a cell carries its origin
/// here instead. Each variant also maps to a CSS class (`thead`/`tbody`/`tfoot`)
/// stamped on the cell so author rules can target header/body/footer rows.
#[derive(Reflect, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TableSection {
    /// A cell that came from `<thead>`.
    Head,
    /// A cell that came from `<tbody>`, or from a bare `<tr>` with no section.
    #[default]
    Body,
    /// A cell that came from `<tfoot>`.
    Foot,
}

impl TableSection {
    /// The CSS class stamped on cells of this section.
    pub fn css_class(self) -> &'static str {
        match self {
            TableSection::Head => "thead",
            TableSection::Body => "tbody",
            TableSection::Foot => "tfoot",
        }
    }
}

/// A `<th>` or `<td>` cell. Its position in the parent grid is fixed at parse time.
#[derive(Component, Reflect, Debug, Clone, PartialEq, Eq)]
#[reflect(Component)]
#[require(UIGenID, UIWidgetState, GlobalTransform, InheritedVisibility, Widget)]
pub struct TableCell {
    pub entry: usize,
    pub header: bool,
    pub section: TableSection,
    pub row: usize,
    pub col: usize,
}

impl Default for TableCell {
    fn default() -> Self {
        let entry = TABLE_CELL_ID_POOL.lock().unwrap().acquire();
        Self {
            entry,
            header: false,
            section: TableSection::default(),
            row: 0,
            col: 0,
        }
    }
}

#[derive(Component)]
struct TableBase;

#[derive(Component)]
struct TableCellBase;

pub struct TableWidget;

impl Plugin for TableWidget {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, internal_node_creation_system);
        // Text cells are rendered after the build pass so a cell that hosts a
        // child widget (e.g. `<td><button>`) already has its `Children` and is
        // skipped — only childless text cells become `Text` nodes.
        app.add_systems(Update, render_cell_text.after(HtmlSystemSet::Build));
        // Grid values depend on the resolved style, so they must be written AFTER
        // the style pass (which fully owns `Node`). `update_widget_styles_system`
        // runs in `PostUpdate`; this system runs in the same schedule, after it.
        app.add_systems(
            PostUpdate,
            apply_table_grid_system.after(update_widget_styles_system),
        );
    }
}

/// Inserts the base node for table and cell widgets.
///
/// Grid layout is deliberately NOT set here — a one-shot write is erased by the
/// style pass. `display:grid` comes from `default_style.rs`; dynamic grid values
/// come from [`apply_table_grid_system`].
fn internal_node_creation_system(
    mut commands: Commands,
    table_query: Query<(Entity, &Table, Option<&CssSource>), (With<Table>, Without<TableBase>)>,
    cell_query: Query<
        (Entity, &TableCell, Option<&CssSource>),
        (With<TableCell>, Without<TableCellBase>),
    >,
    config: Res<ExtendedUiConfiguration>,
) {
    let layer = *config.render_layers.first().unwrap_or(&1);

    for (entity, table, source_opt) in table_query.iter() {
        let css_source = source_opt.cloned().unwrap_or_default();

        commands.entity(entity).insert((
            Name::new(format!("Table-{}", table.entry)),
            Node::default(),
            WidgetId {
                id: table.entry,
                kind: WidgetKind::Table,
            },
            ImageNode::default(),
            BackgroundColor::default(),
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
            css_source,
            TagName("table".to_string()),
            RenderLayers::layer(layer),
            TableBase,
            UIWidgetState::default(),
        ));
    }

    for (entity, cell, source_opt) in cell_query.iter() {
        let css_source = source_opt.cloned().unwrap_or_default();
        let tag = if cell.header { "th" } else { "td" };

        commands.entity(entity).insert((
            Name::new(format!("TableCell-{}", cell.entry)),
            Node::default(),
            WidgetId {
                id: cell.entry,
                kind: WidgetKind::TableCell,
            },
            ImageNode::default(),
            BackgroundColor::default(),
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
            css_source,
            TagName(tag.to_string()),
            RenderLayers::layer(layer),
            TableCellBase,
            UIWidgetState::default(),
        ));
    }
}

/// Renders a cell's inner text as a `Text` node.
///
/// A cell holds either nested widgets (e.g. `<td><button>`) or plain text. Only
/// the plain-text case is handled here: cells that already have child entities
/// are skipped (a `Text` entity cannot also lay out UI children), so a widget
/// cell keeps its widgets and a text cell gets its text. Runs after the build
/// pass so a widget cell's `Children` is already present.
fn render_cell_text(
    mut commands: Commands,
    query: Query<
        (Entity, &HtmlInnerContent),
        (With<TableCellBase>, Without<Text>, Without<Children>),
    >,
) {
    for (entity, content) in query.iter() {
        let text = content.inner_text().trim();
        if text.is_empty() {
            continue;
        }
        commands.entity(entity).insert((
            Text::new(text.to_string()),
            TextColor::default(),
            TextFont::default(),
            TextLayout::default(),
        ));
    }
}

/// Writes the dynamic grid values the style pass cannot know, only when the
/// author left them unset so author CSS always wins.
///
/// Runs in `PostUpdate` after `update_widget_styles_system`, which rewrites every
/// `Node` field from the resolved style. An author `grid-template-columns` leaves
/// `Node.grid_template_columns` non-empty after that pass → skip. An author cell
/// `grid-row`/`grid-column` leaves it `!= GridPlacement::DEFAULT` → skip.
fn apply_table_grid_system(
    mut table_query: Query<(&Table, &mut Node), With<TableBase>>,
    mut cell_query: Query<(&TableCell, &mut Node), (With<TableCellBase>, Without<Table>)>,
) {
    for (table, mut node) in table_query.iter_mut() {
        if node.grid_template_columns.is_empty() {
            // `columns.max(1)` keeps the empty-table case a valid single-track grid.
            node.grid_template_columns =
                vec![RepeatedGridTrack::flex(table.columns.max(1) as u16, 1.0)];
        }
    }

    for (cell, mut node) in cell_query.iter_mut() {
        // `+1`: grid lines are 1-indexed; `GridPlacement::start(0)` panics.
        if node.grid_column == GridPlacement::DEFAULT {
            node.grid_column = GridPlacement::start((cell.col + 1) as i16);
        }
        if node.grid_row == GridPlacement::DEFAULT {
            node.grid_row = GridPlacement::start((cell.row + 1) as i16);
        }
    }
}
