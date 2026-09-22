#![cfg(test)]

use bevy::{ecs::world::CommandQueue, prelude::*};
use bevy_extended_ui::{
    html::{HtmlID, builder::mount_html_fragment, converter::parse_html_fragment},
    styles::{CssClass, CssID},
    widgets::{Button, Div, Paragraph},
};

#[derive(Component)]
struct Controller;

#[test]
fn mounted_fragment_keeps_hierarchy_metadata_and_host_components() {
    let mut world = World::new();
    let parent = world.spawn_empty().id();
    let nodes = parse_html_fragment(
        "<div id='panel' class='game-panel'><p id='label'>Ready</p><button id='save'>Save</button></div>",
    );
    let mut queue = CommandQueue::default();
    let mut visited = Vec::new();
    let roots = mount_html_fragment(
        &mut Commands::new(&mut queue, &world),
        &nodes,
        Some(parent),
        &mut |commands, entity, node| {
            visited.push((entity, node.meta().id.clone().unwrap()));
            commands.entity(entity).insert(Controller);
        },
    );
    queue.apply(&mut world);
    assert_eq!(roots.len(), 1);
    assert_eq!(world.get::<Children>(parent).unwrap()[0], roots[0]);
    assert!(world.get::<Div>(roots[0]).is_some());
    assert_eq!(world.get::<CssClass>(roots[0]).unwrap().0, ["game-panel"]);
    assert_eq!(visited.len(), 3);
    for (entity, id) in &visited {
        let entity = *entity;
        assert!(world.get::<Controller>(entity).is_some());
        assert!(world.get::<HtmlID>(entity).is_none());
        assert_eq!(&world.get::<CssID>(entity).unwrap().0, id);
        if id == "label" {
            assert!(world.get::<Paragraph>(entity).is_some());
        } else if id == "save" {
            assert!(world.get::<Button>(entity).is_some());
        }
    }
    world.entity_mut(parent).despawn();
    assert!(
        visited
            .iter()
            .all(|(entity, _)| world.get_entity(*entity).is_err())
    );
}

#[test]
fn independently_parsed_instances_have_distinct_widget_ids() {
    let mut world = World::new();
    let mut queue = CommandQueue::default();
    let mut roots = Vec::new();
    for _ in 0..2 {
        roots.extend(mount_html_fragment(
            &mut Commands::new(&mut queue, &world),
            &parse_html_fragment("<button>Save</button>"),
            None,
            &mut |_, _, _| {},
        ));
    }
    queue.apply(&mut world);
    assert_ne!(roots[0], roots[1]);
    assert_ne!(
        world.get::<Button>(roots[0]).unwrap().entry,
        world.get::<Button>(roots[1]).unwrap().entry
    );
}
