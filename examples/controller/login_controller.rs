use bevy::prelude::*;
use bevy_extended_ui::html::{HtmlEventBindings, HtmlFunctionRegistry};

#[derive(Resource, Clone, Debug, PartialEq, Eq, Default)]
struct TestResource(pub usize);

pub struct LoginController;

impl Plugin for LoginController {
    fn build(&self, app: &mut App) {
        app.insert_resource(TestResource(20));
        app.add_systems(Startup, register_functions);
    }
}

fn register_functions(mut functions: ResMut<HtmlFunctionRegistry>) {
    functions.click.insert("login".to_string(), login);
    functions.click.insert("username".to_string(), username);
    functions.over.insert("hover".to_string(), hover);
}

fn login(event: Trigger<Pointer<Click>>, mut _commands: Commands) {
    info!("Clicked {:?}", event.target);
    _commands.queue(|command: &mut World| {
        let info = command.resource::<HtmlFunctionRegistry>().click.len();
        info!("Info: {}", info);

        let mut query = command.query_filtered::<(Entity, &HtmlEventBindings), With<HtmlEventBindings>>();
        for (entity, bind) in query.iter(command) {
            info!("Entity: {:?} Bindings: {:?}", entity, bind);
        }
    });
}

fn hover(event: Trigger<Pointer<Over>>, mut _commands: Commands) {
    info!("Hovered {:?}", event.target);
}

fn username(event: Trigger<Pointer<Click>>, mut _commands: Commands) {
    info!("Input Clicked {:?}", event.target);
}