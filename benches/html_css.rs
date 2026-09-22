//! Headless CPU benchmark. Timings exclude setup and are not rendered frame rates.
use bevy::prelude::*;
use bevy_extended_ui::html::converter::{HtmlConverterSystem, parse_html_fragment};
use bevy_extended_ui::html::reload::CssDirty;
use bevy_extended_ui::html::{HtmlDirty, HtmlStructureMap};
use bevy_extended_ui::io::{CssAsset, DefaultCssHandle, HtmlAsset};
use bevy_extended_ui::lang::{UILang, UiLangState, UiLangVariables, UiSharedValues};
use bevy_extended_ui::services::css_service::CssService;
use bevy_extended_ui::services::style_service::{
    apply_calc_styles_system, update_widget_styles_system,
};
use bevy_extended_ui::styles::{CssClass, CssSource, TagName};
use bevy_extended_ui::{ExtendedUiConfiguration, ImageCache};
use std::hint::black_box;
use std::time::Instant;

#[derive(Resource, Default)]
struct LayoutChanges(usize);

fn count_layout_changes(query: Query<Entity, Changed<Node>>, mut count: ResMut<LayoutChanges>) {
    count.0 = query.iter().count();
}

fn measure(label: &str, iterations: usize, mut run: impl FnMut()) {
    let mut samples = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        let start = Instant::now();
        run();
        samples.push(start.elapsed().as_secs_f64() * 1_000.0);
    }
    samples.sort_by(f64::total_cmp);
    println!(
        "{label}: median={:.3} ms p95={:.3} ms ({iterations} samples)",
        samples[iterations / 2],
        samples[(iterations * 95 / 100).min(iterations - 1)]
    );
}

fn main() {
    let smoke = std::env::args().any(|arg| arg == "--smoke");
    let count = if smoke { 100 } else { 1_000 };
    let samples = if smoke { 3 } else { 30 };
    println!("HTML/CSS CPU benchmark: {count} nodes, 250 unrelated CSS rules");
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default(), CssService));
    app.init_asset::<CssAsset>().init_asset::<Image>();
    app.init_resource::<ImageCache>()
        .init_resource::<LayoutChanges>();
    app.add_systems(
        Update,
        (
            update_widget_styles_system,
            apply_calc_styles_system,
            count_layout_changes,
        )
            .chain(),
    );
    let mut css =
        String::from(".item { width: calc(50% - 10px); height: 20px; gap: calc(2px + 2px); }");
    for i in 0..250 {
        css.push_str(&format!(
            ".unused-{i} {{ color: red; padding: 5px; width: 100px; }}"
        ));
    }
    let sheet = app
        .world_mut()
        .resource_mut::<Assets<CssAsset>>()
        .add(CssAsset { text: css });
    let parent = app
        .world_mut()
        .spawn(ComputedNode {
            size: Vec2::new(800.0, 600.0),
            inverse_scale_factor: 1.0,
            ..default()
        })
        .id();
    let entities: Vec<_> = (0..count)
        .map(|_| {
            app.world_mut()
                .spawn((
                    Node::default(),
                    CssSource(vec![sheet.clone()]),
                    CssClass(vec!["item".into()]),
                    TagName("div".into()),
                    ChildOf(parent),
                ))
                .id()
        })
        .collect();
    for _ in 0..5 {
        app.update();
    }
    measure("css/reapply", samples, || {
        for &entity in &entities {
            app.world_mut().entity_mut(entity).insert(CssDirty);
        }
        app.update();
        black_box(app.world());
    });
    for _ in 0..3 {
        app.update();
    }
    measure("css/idle-calc", samples, || {
        app.update();
        black_box(app.world());
    });
    println!(
        "idle Changed<Node>: {}",
        app.world().resource::<LayoutChanges>().0
    );
    if smoke {
        assert_eq!(
            app.world().resource::<LayoutChanges>().0,
            0,
            "idle calc must not invalidate layout"
        );
    }

    let html = format!(
        "<body>{}</body>",
        "<div class=\"item\"><p>Hello</p></div>".repeat(count)
    );
    measure("html/parse", samples, || {
        black_box(parse_html_fragment(&html));
    });

    let mut html_app = App::new();
    html_app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    html_app.init_asset::<HtmlAsset>();
    html_app.insert_resource(DefaultCssHandle(Handle::default()));
    html_app
        .init_resource::<ExtendedUiConfiguration>()
        .init_resource::<HtmlStructureMap>()
        .init_resource::<HtmlDirty>()
        .init_resource::<UILang>()
        .init_resource::<UiLangState>()
        .init_resource::<UiLangVariables>()
        .init_resource::<UiSharedValues>();
    html_app.add_plugins(HtmlConverterSystem);
    for i in 0..count {
        html_app
            .world_mut()
            .resource_mut::<UiSharedValues>()
            .values
            .insert(
                format!("item{i}"),
                serde_json::json!({"title": "Example", "values": [1, 2, 3, 4, 5]}),
            );
    }
    for _ in 0..5 {
        html_app.update();
    }
    measure("html/idle-bindings", samples, || {
        html_app.update();
        black_box(html_app.world());
    });

    #[cfg(feature = "extended-framework")]
    {
        use bevy_extended_ui::framework::UiBindingStore;
        html_app.init_resource::<UiBindingStore>();
        html_app.add_systems(PreUpdate, bevy_extended_ui::lang::refresh_shared_values);
        for i in 0..count {
            html_app
                .world_mut()
                .resource_mut::<UiBindingStore>()
                .set(format!("item{i}"), i);
        }
        for _ in 0..5 {
            html_app.update();
        }
        measure("html/idle-store-sync", samples, || {
            html_app.update();
            black_box(html_app.world());
        });
    }
}
