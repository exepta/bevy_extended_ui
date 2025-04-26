use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;
use bevy::prelude::*;

static UI_ID_COUNTER: AtomicUsize = AtomicUsize::new(1);

#[derive(Component, Debug, Clone)]
pub struct UiGenID(pub usize);

impl Default for UiGenID {
    fn default() -> Self {
        Self(UI_ID_COUNTER.fetch_add(1, Relaxed))
    }
}

#[derive(Component, Debug, Clone)]
pub struct BindToID(pub usize);

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct UiElementState {
    pub disabled: bool,
    pub hovered: bool,
    pub selected: bool
}

impl Default for UiElementState {
    fn default() -> Self {
        Self {
            disabled: false,
            hovered: false,
            selected: false,
        }
    }
}