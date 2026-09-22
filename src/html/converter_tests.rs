use super::*;

fn binding_app() -> App {
    let mut app = App::new();
    app.init_resource::<UiLangVariables>()
        .init_resource::<UiSharedValues>()
        .init_resource::<HtmlBindingValueTracker>();
    app.add_systems(
        Update,
        (track_html_binding_value_changes, patch_html_text_bindings).chain(),
    );
    app
}

#[test]
fn idle_binding_tracking_keeps_snapshots_and_clears_notifications() {
    let mut app = binding_app();
    app.world_mut()
        .resource_mut::<UiLangVariables>()
        .vars
        .insert("name".into(), "Alice".into());
    app.world_mut()
        .resource_mut::<UiSharedValues>()
        .values
        .insert("model".into(), serde_json::json!({"count": 1}));
    app.update();
    let tracker = app.world().resource::<HtmlBindingValueTracker>();
    assert!(tracker.values_changed);
    let snapshot = tracker.vars["name"].as_ptr();
    app.update();
    let tracker = app.world().resource::<HtmlBindingValueTracker>();
    assert!(!tracker.values_changed);
    assert!(tracker.changed_roots.is_empty());
    assert_eq!(
        snapshot,
        tracker.vars["name"].as_ptr(),
        "idle frames must retain the snapshot allocation"
    );
    app.update();
    assert_eq!(
        snapshot,
        app.world().resource::<HtmlBindingValueTracker>().vars["name"].as_ptr()
    );

    app.world_mut()
        .resource_mut::<UiSharedValues>()
        .values
        .insert("model".into(), serde_json::json!({"count": 2}));
    app.update();
    assert!(
        app.world()
            .resource::<HtmlBindingValueTracker>()
            .changed_roots
            .contains("model")
    );
    app.world_mut()
        .resource_mut::<UiLangVariables>()
        .vars
        .remove("name");
    app.update();
    assert!(
        app.world()
            .resource::<HtmlBindingValueTracker>()
            .changed_roots
            .contains("name")
    );
    app.update();
    assert!(
        !app.world()
            .resource::<HtmlBindingValueTracker>()
            .values_changed
    );
}

#[test]
fn binding_changes_after_idle_still_patch_text() {
    let mut app = binding_app();
    app.world_mut()
        .resource_mut::<UiLangVariables>()
        .vars
        .insert("name".into(), "Alice".into());
    let entity = app
        .world_mut()
        .spawn((
            Text::new(""),
            HtmlTextBinding {
                template: "Hello {{name}}".into(),
                bindings: vec!["name".into()],
            },
        ))
        .id();
    app.update();
    assert_eq!(app.world().get::<Text>(entity).unwrap().0, "Hello Alice");
    for _ in 0..4 {
        app.update();
    }
    app.world_mut()
        .resource_mut::<UiLangVariables>()
        .vars
        .insert("name".into(), "Bob".into());
    app.update();
    assert_eq!(app.world().get::<Text>(entity).unwrap().0, "Hello Bob");
}

#[cfg(feature = "extended-framework")]
#[test]
fn unchanged_framework_store_does_not_dirty_shared_values_each_frame() {
    use crate::framework::{UiBindingStore, sync_ui_binding_store_values};
    let mut app = binding_app();
    app.init_resource::<UiBindingStore>();
    app.add_systems(PreUpdate, crate::lang::refresh_shared_values);
    app.world_mut()
        .resource_mut::<UiBindingStore>()
        .set("score", 7_u32);
    app.update();
    app.update();
    let tick = app
        .world()
        .get_resource_ref::<UiSharedValues>()
        .unwrap()
        .last_changed();
    app.update();
    sync_ui_binding_store_values(app.world_mut());
    assert_eq!(
        tick,
        app.world()
            .get_resource_ref::<UiSharedValues>()
            .unwrap()
            .last_changed()
    );
    assert!(
        !app.world()
            .resource::<HtmlBindingValueTracker>()
            .values_changed
    );
    assert_eq!(
        app.world().resource::<UiSharedValues>().values["score"],
        serde_json::json!(7)
    );
    app.world_mut()
        .resource_mut::<UiBindingStore>()
        .set("score", 8_u32);
    app.update();
    assert!(
        app.world()
            .resource::<HtmlBindingValueTracker>()
            .changed_roots
            .contains("score")
    );
    app.world_mut().remove_resource::<UiBindingStore>();
    app.update();
    assert!(
        !app.world()
            .resource::<UiSharedValues>()
            .values
            .contains_key("score")
    );
    assert!(
        app.world()
            .resource::<HtmlBindingValueTracker>()
            .changed_roots
            .contains("score")
    );
}
