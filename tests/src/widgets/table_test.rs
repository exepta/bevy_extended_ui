#[cfg(test)]
mod tests {
    use crate::ExtendedUiConfiguration;
    use crate::styles::TagName;
    use crate::widgets::table::TableWidget;
    use crate::widgets::{Table, TableCell, WidgetId, WidgetKind};
    use bevy::camera::visibility::RenderLayers;
    use bevy::prelude::*;

    /// Builds a headless App with only the real `TableWidget` plugin (init system
    /// in `Update`, grid system in `PostUpdate`). No render-heavy `StyleService`: a
    /// freshly spawned entity's `Node` already matches the post-style default state
    /// (empty grid template, `GridPlacement::DEFAULT`), so `apply_table_grid_system`
    /// exercises the same branch it would after the style pass.
    fn widget_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(ExtendedUiConfiguration::default());
        app.add_plugins(TableWidget);
        app
    }

    /// Runs enough frames that both the `Update` init system and the `PostUpdate`
    /// grid system execute against the spawned entities.
    fn run_frames(app: &mut App, frames: usize) {
        for _ in 0..frames {
            app.update();
        }
    }

    #[test]
    fn table_init_attaches_base_components_and_tag() {
        let mut app = widget_app();
        let table = app
            .world_mut()
            .spawn(Table {
                columns: 2,
                ..default()
            })
            .id();
        run_frames(&mut app, 2);

        let world = app.world();
        // Base styling components — these come from
        // `internal_node_creation_system`, NOT `spawn_with_meta`, so CSS can target them.
        assert!(world.get::<BackgroundColor>(table).is_some());
        assert!(world.get::<BorderColor>(table).is_some());
        assert!(world.get::<ZIndex>(table).is_some());
        assert!(world.get::<Pickable>(table).is_some());
        assert!(world.get::<RenderLayers>(table).is_some());

        let tag = world.get::<TagName>(table).expect("table TagName");
        assert_eq!(tag.0, "table");

        let wid = world.get::<WidgetId>(table).expect("table WidgetId");
        assert!(matches!(wid.kind, WidgetKind::Table));
    }

    #[test]
    fn cell_tag_reflects_header_flag() {
        let mut app = widget_app();
        let header = app
            .world_mut()
            .spawn(TableCell {
                header: true,
                row: 0,
                col: 0,
                ..default()
            })
            .id();
        let data = app
            .world_mut()
            .spawn(TableCell {
                header: false,
                row: 0,
                col: 1,
                ..default()
            })
            .id();
        run_frames(&mut app, 2);

        let world = app.world();
        assert_eq!(world.get::<TagName>(header).unwrap().0, "th");
        assert_eq!(world.get::<TagName>(data).unwrap().0, "td");
        assert!(matches!(
            world.get::<WidgetId>(data).unwrap().kind,
            WidgetKind::TableCell
        ));
    }

    #[test]
    fn grid_system_fills_template_and_cell_placement() {
        let mut app = widget_app();
        let table = app
            .world_mut()
            .spawn(Table {
                columns: 3,
                ..default()
            })
            .id();
        let cell = app
            .world_mut()
            .spawn(TableCell {
                header: false,
                row: 1,
                col: 2,
                ..default()
            })
            .id();
        run_frames(&mut app, 2);

        let world = app.world();
        let table_node = world.get::<Node>(table).expect("table Node");
        assert!(
            !table_node.grid_template_columns.is_empty(),
            "auto column template must be applied when author left it unset"
        );

        let cell_node = world.get::<Node>(cell).expect("cell Node");
        // 1-based: row 1 -> start(2), col 2 -> start(3).
        assert_eq!(cell_node.grid_row, GridPlacement::start(2));
        assert_eq!(cell_node.grid_column, GridPlacement::start(3));
    }

    #[test]
    fn empty_table_gets_single_track_without_panic() {
        let mut app = widget_app();
        let table = app
            .world_mut()
            .spawn(Table {
                columns: 0,
                ..default()
            })
            .id();
        run_frames(&mut app, 2);

        // `columns.max(1)` keeps a zero-column table a valid single-track grid.
        let node = app.world().get::<Node>(table).expect("table Node");
        assert_eq!(node.grid_template_columns.len(), 1);
    }

    #[test]
    fn author_template_is_not_overwritten_by_grid_system() {
        // Author CSS path: the post-style `Node` already carries a non-empty
        // `grid_template_columns`. The grid system must see it non-empty and skip
        // the auto `repeat(N, 1fr)` default so the author value wins.
        let mut app = widget_app();
        let table = app
            .world_mut()
            .spawn(Table {
                columns: 3,
                ..default()
            })
            .id();
        // Let the init system attach the base Node first.
        app.update();
        // Simulate the resolved author style: 2 explicit tracks on the Node.
        {
            let mut node = app.world_mut().get_mut::<Node>(table).unwrap();
            node.grid_template_columns = vec![
                RepeatedGridTrack::px(1, 80.0),
                RepeatedGridTrack::flex(1, 1.0),
            ];
        }
        app.update();

        let node = app.world().get::<Node>(table).unwrap();
        assert_eq!(
            node.grid_template_columns.len(),
            2,
            "author template (2 tracks) survives; auto repeat(3) is skipped"
        );
    }

    #[test]
    fn grid_is_reapplied_after_a_node_reset() {
        // Clobber regression (the bug review caught): the style pass rewrites every
        // `Node` field via `unwrap_or_default()` each dirty frame, erasing the grid.
        // Because `apply_table_grid_system` runs AFTER the style pass every frame, a
        // reset `Node` must be re-patched on the next update.
        let mut app = widget_app();
        let table = app
            .world_mut()
            .spawn(Table {
                columns: 2,
                ..default()
            })
            .id();
        let cell = app
            .world_mut()
            .spawn(TableCell {
                row: 0,
                col: 1,
                ..default()
            })
            .id();
        app.update();
        app.update();
        assert!(
            !app.world()
                .get::<Node>(table)
                .unwrap()
                .grid_template_columns
                .is_empty()
        );

        // Reproduce `apply_style_to_node`'s clobber: reset the grid fields to their
        // post-style defaults (empty template, DEFAULT placement).
        {
            let mut t_node = app.world_mut().get_mut::<Node>(table).unwrap();
            t_node.grid_template_columns = Vec::new();
        }
        {
            let mut c_node = app.world_mut().get_mut::<Node>(cell).unwrap();
            c_node.grid_column = GridPlacement::DEFAULT;
            c_node.grid_row = GridPlacement::DEFAULT;
        }
        app.update();

        let world = app.world();
        assert!(
            !world
                .get::<Node>(table)
                .unwrap()
                .grid_template_columns
                .is_empty(),
            "table grid template restored after reset"
        );
        assert_eq!(
            world.get::<Node>(cell).unwrap().grid_column,
            GridPlacement::start(2),
            "cell placement restored after reset"
        );
    }

    #[test]
    fn ragged_cells_keep_their_own_column_indices() {
        let mut app = widget_app();
        // Row 1 of a ragged table: two cells at col 0 and col 1; col 2 stays empty.
        let c0 = app
            .world_mut()
            .spawn(TableCell {
                row: 1,
                col: 0,
                ..default()
            })
            .id();
        let c1 = app
            .world_mut()
            .spawn(TableCell {
                row: 1,
                col: 1,
                ..default()
            })
            .id();
        run_frames(&mut app, 2);

        let world = app.world();
        assert_eq!(
            world.get::<Node>(c0).unwrap().grid_column,
            GridPlacement::start(1)
        );
        assert_eq!(
            world.get::<Node>(c1).unwrap().grid_column,
            GridPlacement::start(2)
        );
    }
}
