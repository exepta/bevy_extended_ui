use bevy::prelude::*;
use bevy_extended_ui_macros::ui_component;

#[ui_component]
pub struct TestComponent {
    pub template_name: &'static str,
    pub template_file: &'static str,
    pub styles: &'static [&'static str],
}

pub const TEST_COMPONENT: TestComponent = TestComponent {
    template_name: "app-test",
    template_file: "test.component.html",
    styles: &["test.component.css"],
};

pub fn framework_component_marker() -> Name {
    Name::new("TestComponent")
}
