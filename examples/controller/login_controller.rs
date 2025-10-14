use bevy::prelude::*;
use bevy_extended_ui::html::{HtmlEventBindings, HtmlFunctionRegistry};
use bevy_extended_ui::observer::time_tick_trigger::TimeTick;
use bevy_extended_ui::observer::widget_init_trigger::WidgetInit;
use bevy_extended_ui::widgets::ProgressBar;

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
    functions.update.insert("progress".to_string(), progress);
    functions.load.insert("test".to_string(), test);
    functions.load.insert("test_2".to_string(), test_2);
    functions.load.insert("test_3".to_string(), test_3);
    functions.load.insert("test_4".to_string(), test_4);
}

fn login(event: On<Pointer<Click>>, mut commands: Commands) {
    info!("Clicked {:?}", event.entity);
    commands.queue(|command: &mut World| {
        let info = command.resource::<HtmlFunctionRegistry>().click.len();
        info!("Info: {}", info);

        let mut query = command.query_filtered::<(Entity, &HtmlEventBindings), With<HtmlEventBindings>>();
        for (entity, bind) in query.iter(command) {
            info!("Entity: {:?} Bindings: {:?}", entity, bind);
        }
    });
}

fn hover(event: On<Pointer<Over>>, _commands: Commands) {
    info!("Hovered {:?}", event.entity);
}

fn username(event: On<Pointer<Click>>, _commands: Commands) {
    info!("Input Clicked {:?}", event.entity);
}

fn progress(event: On<TimeTick>, mut commands: Commands) {
    let target = event.entity;

    commands.queue(move |command: &mut World| {
        let mut query = command.query_filtered::<(Entity, &mut ProgressBar), With<ProgressBar>>();
        for (entity, mut progress) in query.iter_mut(command) {
            if entity.eq(&target) {
                if progress.value < progress.max {
                    progress.value += 0.05;
                } else {
                    progress.value = 0.0;
                }
            }
        }
    });
}

fn test(event: On<WidgetInit>, _commands: Commands) {
    info!("Load {:?} - 1", event.entity);
}


fn test_2(event: On<WidgetInit>, _commands: Commands) {
    info!("Load {:?} - 2", event.entity);
}

fn test_3(event: On<WidgetInit>, _commands: Commands) {
    info!("Load {:?} - 3", event.entity);
}

fn test_4(event: On<WidgetInit>, _commands: Commands) {
    info!("Load {:?} - 4", event.entity);
}