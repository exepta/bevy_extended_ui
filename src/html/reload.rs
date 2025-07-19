use std::sync::{Arc, Mutex};
use std::sync::mpsc::channel;
use bevy::prelude::*;
use notify::{recommended_watcher, RecursiveMode, Watcher};
use crate::html::{HtmlChangeEvent, HtmlSource, HtmlWatcher};
use crate::registry::UiRegistry;

pub struct HtmlReloadSystem;

impl Plugin for HtmlReloadSystem {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, start_file_watcher);
        app.add_systems(Update, (detect_changes, reload_html.after(detect_changes)).run_if(resource_exists::<HtmlWatcher>));
    }
}

fn start_file_watcher(mut commands: Commands) {
    let (tx, rx) = channel();

    let mut watcher = recommended_watcher(tx).expect("Failed to create watcher");
    watcher
        .watch("examples/html/login-ui.html".as_ref(), RecursiveMode::NonRecursive)
        .expect("Failed to watch");

    commands.insert_resource(HtmlWatcher {
        watcher,
        rx: Arc::new(Mutex::new(rx)),
    });
}

fn detect_changes(
    watcher: Res<HtmlWatcher>,
    mut reload: EventWriter<HtmlChangeEvent>
) {
    let guard = watcher.rx.lock().unwrap();
    while let Ok(Ok(event)) = guard.try_recv() {
        if matches!(event.kind, notify::EventKind::Modify(_)) {
            debug!("Detected change in html file");
            reload.write(HtmlChangeEvent);
        }
    }
}


fn reload_html(
    mut ev: EventReader<HtmlChangeEvent>,
    mut registry: ResMut<UiRegistry>,
    mut query: Query<&mut HtmlSource>
) {
    if ev.read().next().is_none() {
        return;
    }

    let ui_name = match registry.current.as_ref() {
        Some(name) => name.clone(),
        None => {
            warn!("HtmlChangeEvent, but nothing to reload.");
            return;
        }
    };

    let current_source = match registry.collection.get_mut(&ui_name) {
        Some(src) => src,
        None => {
            warn!("HtmlChangeEvent, but '{}' not in registry.", ui_name);
            return;
        }
    };

    if current_source.was_updated {
        return;
    }

    current_source.was_updated = true;
    for mut src in query.iter_mut() {
        if src.source_id.eq(&ui_name) {
            src.was_updated = true;
        }
    }

    info!("Reloaded html file");
}