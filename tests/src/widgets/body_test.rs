#[cfg(test)]
mod tests {
    use crate::styles::{CssClass, TagName};
    use crate::widgets::body::{
        BodyBase, BodyContentRoot, BodyScrollContent, BodyScrollbar, HoveredBodyTracker,
        ensure_body_scroll_structure, handle_body_scroll_wheel,
    };
    use crate::widgets::controls::choice_box::ChoiceLayoutBoxBase;
    use crate::widgets::div::DivContentRoot;
    use crate::widgets::{Body, Div, Scrollbar, UIGenID, UIWidgetState};
    use bevy::ecs::message::Messages;
    use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
    use bevy::prelude::*;

    fn computed_node(viewport: Vec2, content: Vec2) -> ComputedNode {
        let mut computed = ComputedNode::default();
        computed.inverse_scale_factor = 1.0;
        computed.size = viewport;
        computed.content_size = content;
        computed
    }

    fn line_wheel(y: f32) -> MouseWheel {
        MouseWheel {
            unit: MouseScrollUnit::Line,
            x: 0.0,
            y,
            window: Entity::PLACEHOLDER,
        }
    }

    fn line_wheel_xy(x: f32, y: f32) -> MouseWheel {
        MouseWheel {
            unit: MouseScrollUnit::Line,
            x,
            y,
            window: Entity::PLACEHOLDER,
        }
    }

    fn spawn_body_scroll_content(world: &mut World, viewport_h: f32, content_h: f32) -> Entity {
        let mut node = Node::default();
        node.overflow = Overflow {
            x: OverflowAxis::Hidden,
            y: OverflowAxis::Scroll,
        };

        world
            .spawn((
                BodyScrollContent,
                node,
                computed_node(Vec2::new(200.0, viewport_h), Vec2::new(200.0, content_h)),
                ScrollPosition::default(),
            ))
            .id()
    }

    fn spawn_body_scroll_content_xy(world: &mut World, viewport: Vec2, content: Vec2) -> Entity {
        let mut node = Node::default();
        node.overflow = Overflow {
            x: OverflowAxis::Scroll,
            y: OverflowAxis::Hidden,
        };

        world
            .spawn((
                BodyScrollContent,
                node,
                computed_node(viewport, content),
                ScrollPosition::default(),
            ))
            .id()
    }

    fn spawn_div_scroll_content(world: &mut World, viewport_h: f32, content_h: f32) -> Entity {
        let mut node = Node::default();
        node.overflow = Overflow {
            x: OverflowAxis::Hidden,
            y: OverflowAxis::Scroll,
        };

        world
            .spawn((
                node,
                computed_node(Vec2::new(180.0, viewport_h), Vec2::new(180.0, content_h)),
                ScrollPosition::default(),
            ))
            .id()
    }

    fn spawn_choice_overlay(
        world: &mut World,
        viewport_h: f32,
        content_h: f32,
        hovered: bool,
    ) -> Entity {
        let mut node = Node::default();
        node.overflow = Overflow {
            x: OverflowAxis::Hidden,
            y: OverflowAxis::Scroll,
        };

        world
            .spawn((
                ChoiceLayoutBoxBase,
                UIWidgetState {
                    hovered,
                    ..default()
                },
                node,
                computed_node(Vec2::new(220.0, viewport_h), Vec2::new(220.0, content_h)),
                ScrollPosition::default(),
                Visibility::Inherited,
            ))
            .id()
    }

    fn spawn_dialog_overlay_marker(world: &mut World, visible: bool) -> Entity {
        world
            .spawn((
                TagName("dialog-overlay".to_string()),
                CssClass(vec!["dialog-overlay".to_string()]),
                if visible {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                },
            ))
            .id()
    }

    #[test]
    fn body_wheel_scroll_moves_body_content_when_no_div_is_hovered() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Messages<MouseWheel>>();
        app.insert_resource(HoveredBodyTracker::default());
        app.add_systems(Update, handle_body_scroll_wheel);

        let content_entity = spawn_body_scroll_content(app.world_mut(), 100.0, 300.0);
        let body_entity = app
            .world_mut()
            .spawn((Body::default(), BodyContentRoot(content_entity)))
            .id();
        app.world_mut()
            .resource_mut::<HoveredBodyTracker>()
            .last_body = Some(body_entity);

        app.world_mut()
            .resource_mut::<Messages<MouseWheel>>()
            .write(line_wheel(-1.0));
        app.update();

        let pos = app
            .world()
            .get::<ScrollPosition>(content_entity)
            .expect("body scroll content is missing ScrollPosition");
        assert_eq!(pos.y, 25.0);
    }

    #[test]
    fn body_wheel_scroll_moves_body_content_horizontally_when_x_wheel_is_used() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Messages<MouseWheel>>();
        app.insert_resource(HoveredBodyTracker::default());
        app.add_systems(Update, handle_body_scroll_wheel);

        let content_entity = spawn_body_scroll_content_xy(
            app.world_mut(),
            Vec2::new(100.0, 80.0),
            Vec2::new(260.0, 80.0),
        );
        let body_entity = app
            .world_mut()
            .spawn((Body::default(), BodyContentRoot(content_entity)))
            .id();
        app.world_mut()
            .resource_mut::<HoveredBodyTracker>()
            .last_body = Some(body_entity);

        app.world_mut()
            .resource_mut::<Messages<MouseWheel>>()
            .write(line_wheel_xy(-1.0, 0.0));
        app.update();

        let pos = app
            .world()
            .get::<ScrollPosition>(content_entity)
            .expect("body scroll content is missing ScrollPosition");
        assert_eq!(pos.x, 25.0);
        assert_eq!(pos.y, 0.0);
    }

    #[test]
    fn body_wheel_scroll_uses_shift_vertical_wheel_for_horizontal_scroll() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Messages<MouseWheel>>();
        app.insert_resource(ButtonInput::<KeyCode>::default());
        app.insert_resource(HoveredBodyTracker::default());
        app.add_systems(Update, handle_body_scroll_wheel);

        let content_entity = spawn_body_scroll_content_xy(
            app.world_mut(),
            Vec2::new(100.0, 80.0),
            Vec2::new(260.0, 80.0),
        );
        let body_entity = app
            .world_mut()
            .spawn((Body::default(), BodyContentRoot(content_entity)))
            .id();
        app.world_mut()
            .resource_mut::<HoveredBodyTracker>()
            .last_body = Some(body_entity);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::ShiftLeft);

        app.world_mut()
            .resource_mut::<Messages<MouseWheel>>()
            .write(line_wheel(-1.0));
        app.update();

        let pos = app
            .world()
            .get::<ScrollPosition>(content_entity)
            .expect("body scroll content is missing ScrollPosition");
        assert_eq!(pos.x, 25.0);
        assert_eq!(pos.y, 0.0);
    }

    #[test]
    fn body_wheel_scroll_is_blocked_by_hovered_scrollable_div() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Messages<MouseWheel>>();
        app.insert_resource(HoveredBodyTracker::default());
        app.add_systems(Update, handle_body_scroll_wheel);

        let body_content_entity = spawn_body_scroll_content(app.world_mut(), 100.0, 300.0);
        let body_entity = app
            .world_mut()
            .spawn((Body::default(), BodyContentRoot(body_content_entity)))
            .id();
        app.world_mut()
            .resource_mut::<HoveredBodyTracker>()
            .last_body = Some(body_entity);

        let div_content_entity = spawn_div_scroll_content(app.world_mut(), 120.0, 260.0);
        app.world_mut().spawn((
            Div::default(),
            UIWidgetState {
                hovered: true,
                ..default()
            },
            DivContentRoot(div_content_entity),
        ));

        app.world_mut()
            .resource_mut::<Messages<MouseWheel>>()
            .write(line_wheel(-2.0));
        app.update();

        let pos = app
            .world()
            .get::<ScrollPosition>(body_content_entity)
            .expect("body scroll content is missing ScrollPosition");
        assert_eq!(pos.y, 0.0);
    }

    #[test]
    fn body_wheel_scroll_ignores_hovered_div_without_scroll_range() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Messages<MouseWheel>>();
        app.insert_resource(HoveredBodyTracker::default());
        app.add_systems(Update, handle_body_scroll_wheel);

        let body_content_entity = spawn_body_scroll_content(app.world_mut(), 100.0, 300.0);
        let body_entity = app
            .world_mut()
            .spawn((Body::default(), BodyContentRoot(body_content_entity)))
            .id();
        app.world_mut()
            .resource_mut::<HoveredBodyTracker>()
            .last_body = Some(body_entity);

        // Content equals viewport -> max scroll is zero, so this div should not block body scrolling.
        let div_content_entity = spawn_div_scroll_content(app.world_mut(), 160.0, 160.0);
        app.world_mut().spawn((
            Div::default(),
            UIWidgetState {
                hovered: true,
                ..default()
            },
            DivContentRoot(div_content_entity),
        ));

        app.world_mut()
            .resource_mut::<Messages<MouseWheel>>()
            .write(line_wheel(-1.0));
        app.update();

        let pos = app
            .world()
            .get::<ScrollPosition>(body_content_entity)
            .expect("body scroll content is missing ScrollPosition");
        assert_eq!(pos.y, 25.0);
    }

    #[test]
    fn ensure_body_scroll_structure_creates_content_and_vertical_scrollbar() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, ensure_body_scroll_structure);

        let mut body_node = Node::default();
        body_node.width = Val::Px(640.0);
        body_node.height = Val::Px(360.0);
        body_node.overflow = Overflow {
            x: OverflowAxis::Hidden,
            y: OverflowAxis::Scroll,
        };

        let body_entity = app
            .world_mut()
            .spawn((Body::default(), BodyBase, UIGenID::default(), body_node))
            .id();

        app.update();

        let body_root = app
            .world()
            .get::<BodyContentRoot>(body_entity)
            .expect("body should have a content root after setup");
        let content_entity = **body_root;

        let body_node_after = app
            .world()
            .get::<Node>(body_entity)
            .expect("body should still have a node");
        assert_eq!(body_node_after.overflow.y, OverflowAxis::Clip);
        assert_eq!(body_node_after.overflow.x, OverflowAxis::Clip);

        assert!(
            app.world()
                .get::<BodyScrollContent>(content_entity)
                .is_some(),
            "content root should be marked as BodyScrollContent"
        );
        assert!(
            app.world().get::<ScrollPosition>(content_entity).is_some(),
            "content root should be scrollable"
        );

        let content_node = app
            .world()
            .get::<Node>(content_entity)
            .expect("content root should have a node");
        assert_eq!(content_node.overflow.y, OverflowAxis::Scroll);
        assert_eq!(content_node.overflow.x, OverflowAxis::Hidden);
        assert_eq!(content_node.width, Val::Percent(100.0));
        assert_eq!(content_node.height, Val::Percent(100.0));

        let scrollbar_ref = app
            .world()
            .get::<BodyScrollbar>(body_entity)
            .expect("vertical scrollbar should exist");
        let scrollbar = app
            .world()
            .get::<Scrollbar>(**scrollbar_ref)
            .expect("scrollbar entity should have Scrollbar component");
        assert!(scrollbar.vertical);
        assert_eq!(scrollbar.entity, Some(content_entity));
    }

    #[test]
    fn body_wheel_scroll_is_blocked_by_hovered_scrollable_choice_overlay() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Messages<MouseWheel>>();
        app.insert_resource(HoveredBodyTracker::default());
        app.add_systems(Update, handle_body_scroll_wheel);

        let body_content_entity = spawn_body_scroll_content(app.world_mut(), 100.0, 300.0);
        let body_entity = app
            .world_mut()
            .spawn((Body::default(), BodyContentRoot(body_content_entity)))
            .id();
        app.world_mut()
            .resource_mut::<HoveredBodyTracker>()
            .last_body = Some(body_entity);

        let _overlay = spawn_choice_overlay(app.world_mut(), 90.0, 240.0, true);

        app.world_mut()
            .resource_mut::<Messages<MouseWheel>>()
            .write(line_wheel(-2.0));
        app.update();

        let pos = app
            .world()
            .get::<ScrollPosition>(body_content_entity)
            .expect("body scroll content is missing ScrollPosition");
        assert_eq!(pos.y, 0.0);
    }

    #[test]
    fn body_wheel_scroll_is_blocked_by_visible_dialog_overlay() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Messages<MouseWheel>>();
        app.insert_resource(HoveredBodyTracker::default());
        app.add_systems(Update, handle_body_scroll_wheel);

        let body_content_entity = spawn_body_scroll_content(app.world_mut(), 100.0, 300.0);
        let body_entity = app
            .world_mut()
            .spawn((Body::default(), BodyContentRoot(body_content_entity)))
            .id();
        app.world_mut()
            .resource_mut::<HoveredBodyTracker>()
            .last_body = Some(body_entity);

        let _overlay = spawn_dialog_overlay_marker(app.world_mut(), true);

        app.world_mut()
            .resource_mut::<Messages<MouseWheel>>()
            .write(line_wheel(-2.0));
        app.update();

        let pos = app
            .world()
            .get::<ScrollPosition>(body_content_entity)
            .expect("body scroll content is missing ScrollPosition");
        assert_eq!(pos.y, 0.0);
    }
}
