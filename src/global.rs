use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;
use bevy::prelude::*;

static UI_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(Component, Debug, Clone)]
pub struct UiGenID(pub usize);

impl Default for UiGenID {
    fn default() -> Self {
        Self(UI_ID_COUNTER.fetch_add(1, Relaxed))
    }
}

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct UiElementState {
    pub hovered: bool,
    pub selected: bool,
    pub visible: bool,
    pub enabled: bool
}

impl Default for UiElementState {
    fn default() -> Self {
        Self {
            enabled: true,
            visible: true,
            hovered: false,
            selected: false,
        }
    }
}