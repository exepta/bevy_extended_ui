#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy_extended_ui::ImageCache;
    use bevy_extended_ui::io::CssAsset;
    use bevy_extended_ui::services::css_service::CssService;
    use bevy_extended_ui::services::style_service::{
        StyleTransition, apply_calc_styles_system, update_widget_styles_system,
    };
    use bevy_extended_ui::styles::components::UiStyle;
    use bevy_extended_ui::styles::{
        CalcExpr, CalcUnit, CalcValue, CssClass, CssSource, TagName, TransitionSpec,
    };

    #[derive(Resource, Default)]
    struct ChangedNodes(Vec<Entity>);

    fn record_changes(query: Query<Entity, Changed<Node>>, mut changed: ResMut<ChangedNodes>) {
        changed.0 = query.iter().collect();
    }

    fn app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default(), CssService));
        app.init_asset::<CssAsset>().init_asset::<Image>();
        app.init_resource::<ImageCache>()
            .init_resource::<ChangedNodes>();
        app.add_systems(
            PostUpdate,
            (
                update_widget_styles_system,
                apply_calc_styles_system,
                record_changes,
            )
                .chain(),
        );
        app
    }

    fn sheet(app: &mut App, text: &str) -> Handle<CssAsset> {
        app.world_mut()
            .resource_mut::<Assets<CssAsset>>()
            .add(CssAsset { text: text.into() })
    }

    fn node(app: &mut App, sources: Vec<Handle<CssAsset>>) -> Entity {
        app.world_mut()
            .spawn((
                Node::default(),
                CssSource(sources),
                CssClass(vec!["item".into()]),
                TagName("div".into()),
            ))
            .id()
    }

    fn settle(app: &mut App) {
        for _ in 0..4 {
            app.update();
        }
    }

    #[test]
    fn unchanged_calc_does_not_invalidate_layout_but_parent_resize_does() {
        let mut app = app();
        let css = sheet(
            &mut app,
            ".item { width: calc(50% - 10px); height: calc(25% + 2px); min-width: calc(10% + 1px); max-width: calc(90% - 1px); min-height: calc(10% + 1px); max-height: calc(90% - 1px); left: calc(1% + 1px); right: calc(2% + 1px); top: calc(3% + 1px); bottom: calc(4% + 1px); flex-basis: calc(10% + 2px); gap: calc(1% + 3px); row-gap: calc(2% + 3px); }",
        );
        let parent = app
            .world_mut()
            .spawn(ComputedNode {
                size: Vec2::new(800.0, 600.0),
                inverse_scale_factor: 1.0,
                ..default()
            })
            .id();
        let entity = node(&mut app, vec![css]);
        app.world_mut().entity_mut(parent).add_child(entity);
        settle(&mut app);
        let before = app.world().get::<Node>(entity).unwrap().clone();
        assert_eq!(before.width, Val::Px(390.0));
        assert_eq!(before.height, Val::Px(152.0));
        assert_eq!(before.row_gap, Val::Px(19.0));
        assert_eq!(before.column_gap, Val::Px(11.0));
        assert!(app.world().resource::<ChangedNodes>().0.is_empty());

        app.world_mut()
            .get_mut::<ComputedNode>(parent)
            .unwrap()
            .size = Vec2::new(1000.0, 800.0);
        app.update();
        let resized = app.world().get::<Node>(entity).unwrap();
        assert_eq!(resized.width, Val::Px(490.0));
        assert_eq!(resized.height, Val::Px(202.0));
        assert!(app.world().resource::<ChangedNodes>().0.contains(&entity));
        app.update();
        assert!(app.world().resource::<ChangedNodes>().0.is_empty());
    }

    #[test]
    fn calc_observes_transition_values_and_reparenting() {
        let mut app = app();
        let css = sheet(&mut app, ".item { width: calc(50% - 10px); }");
        let entity = node(&mut app, vec![css]);
        settle(&mut app);
        let mut current = app
            .world()
            .get::<UiStyle>(entity)
            .unwrap()
            .active_style
            .clone()
            .unwrap();
        current.width_calc = Some(CalcExpr::Value(CalcValue::new(123.0, CalcUnit::Px)));
        app.world_mut().entity_mut(entity).insert(StyleTransition {
            from: current.clone(),
            to: current.clone(),
            current_style: Some(current),
            start_time: 0.0,
            spec: TransitionSpec::default(),
            from_transform: None,
            to_transform: None,
        });
        app.update();
        assert_eq!(
            app.world().get::<Node>(entity).unwrap().width,
            Val::Px(123.0)
        );
        app.update();
        assert!(app.world().resource::<ChangedNodes>().0.is_empty());
        app.world_mut()
            .entity_mut(entity)
            .remove::<StyleTransition>();
        let parent = app
            .world_mut()
            .spawn(ComputedNode {
                size: Vec2::splat(600.0),
                inverse_scale_factor: 1.0,
                ..default()
            })
            .id();
        app.world_mut().entity_mut(parent).add_child(entity);
        app.update();
        assert_eq!(
            app.world().get::<Node>(entity).unwrap().width,
            Val::Px(290.0)
        );
    }

    #[test]
    fn css_cache_is_isolated_between_apps() {
        let mut first = app();
        let mut second = app();
        let red = sheet(&mut first, ".item { width: 100px; }");
        let blue = sheet(&mut second, ".item { width: 200px; }");
        assert_eq!(red.id(), blue.id(), "exercise colliding asset indices");
        let a = node(&mut first, vec![red]);
        let b = node(&mut second, vec![blue]);
        settle(&mut first);
        settle(&mut second);
        first
            .world_mut()
            .entity_mut(a)
            .insert(bevy_extended_ui::html::reload::CssDirty);
        first.update();
        assert_eq!(first.world().get::<Node>(a).unwrap().width, Val::Px(100.0));
        assert_eq!(second.world().get::<Node>(b).unwrap().width, Val::Px(200.0));
    }

    #[test]
    fn css_reload_updates_cross_sheet_variables_and_removal() {
        let mut app = app();
        let vars = sheet(&mut app, ":root { --size: 100px; }");
        let rules = sheet(&mut app, ".item { width: var(--size, 25px); }");
        let entity = node(&mut app, vec![vars.clone(), rules]);
        settle(&mut app);
        assert_eq!(
            app.world().get::<Node>(entity).unwrap().width,
            Val::Px(100.0)
        );
        app.world_mut()
            .resource_mut::<Assets<CssAsset>>()
            .get_mut(&vars)
            .unwrap()
            .text = ":root { --size: 200px; }".into();
        settle(&mut app);
        assert_eq!(
            app.world().get::<Node>(entity).unwrap().width,
            Val::Px(200.0)
        );
        app.world_mut()
            .resource_mut::<Assets<CssAsset>>()
            .remove(vars.id());
        settle(&mut app);
        assert_eq!(
            app.world().get::<Node>(entity).unwrap().width,
            Val::Px(25.0)
        );
    }
}
