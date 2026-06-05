#[cfg(feature = "extended-framework")]
use bevy_extended_ui_macros::beu_registry;

#[cfg(feature = "extended-framework")]
#[beu_registry]
mod beu_registry_marker {}

#[cfg(feature = "extended-framework")]
#[allow(dead_code)]
#[path = "../assets/components/beu.routes.rs"]
mod beu_routes;

#[cfg(feature = "extended-framework")]
#[allow(dead_code)]
#[path = "../assets/components/secondary.routes.rs"]
mod secondary_routes;

#[cfg(feature = "extended-framework")]
#[allow(dead_code)]
#[path = "../assets/components/main.component.rs"]
mod main_component_mod;

#[cfg(feature = "extended-framework")]
#[allow(dead_code)]
#[path = "../assets/components/test/test.component.rs"]
mod test_component_mod;

#[cfg(feature = "extended-framework")]
#[allow(dead_code)]
#[path = "../assets/components/help/help.component.rs"]
mod help_component_mod;

#[cfg(feature = "extended-framework")]
#[allow(dead_code)]
#[path = "../assets/components/settings/settings.component.rs"]
mod settings_component_mod;

#[cfg(feature = "extended-framework")]
#[allow(dead_code)]
#[path = "../assets/components/infopage/infopage.component.rs"]
mod infopage_component_mod;
