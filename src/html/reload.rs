use std::sync::{Arc, Mutex};
use std::sync::mpsc::channel;
use bevy::prelude::*;
use notify::{recommended_watcher, RecursiveMode, Watcher};
use crate::html::{HtmlChangeEvent, HtmlWatcher};

pub struct HtmlReloadSystem;

impl Plugin for HtmlReloadSystem {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, start_file_watcher);
        app.add_systems(Update, detect_changes.run_if(resource_exists::<HtmlWatcher>));
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