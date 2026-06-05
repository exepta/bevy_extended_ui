#[cfg(test)]
mod tests {
    use super::super::converter::{
        extract_inner_bindings, parse_inner_content, preprocess_template_directives,
        preprocess_template_directives_with_shared,
        preprocess_template_directives_with_shared_and_local_types,
    };
    use crate::lang::{UiLangVariables, UiSharedValues};
    use kuchiki::traits::TendrilSink;

    #[test]
    fn extract_inner_bindings_returns_unique_placeholders() {
        let src = "<p>{{user.name}} {{ user.name }} {{ user.name }} {{user.id}}</p>";
        let bindings = extract_inner_bindings(src);

        assert_eq!(
            bindings,
            vec![
                "{{user.name}}".to_string(),
                "{{ user.name }}".to_string(),
                "{{user.id}}".to_string()
            ]
        );
    }

    #[test]
    fn parse_inner_content_collects_text_html_and_bindings() {
        let doc = kuchiki::parse_html().one("<p>Hello <b>{{ user.name }}</b>!</p>");
        let node = doc.select_first("p").unwrap();
        let content = parse_inner_content(node.as_node());

        assert!(content.inner_text().contains("Hello"));
        assert!(content.inner_html().contains("<b>{{ user.name }}</b>"));
        assert_eq!(content.inner_bindings(), &["{{ user.name }}".to_string()]);
    }

    #[test]
    fn preprocess_template_directives_handles_if_logic_and_methods() {
        let mut vars = UiLangVariables::default();
        vars.set("data", r#"{"username":"NetRunner","state":true}"#);
        vars.set("client", r#"{"id":42}"#);
        vars.set("state", "true");
        vars.set("file", r#""avatar.png""#);

        let template = r#"
            @if(data.username.startsWidth("Net") && client.id == 42) {
              <div><p>Hello World</p></div>
            }
            @if(file != "") {
              <img src="{{ file }}">
            } @else {
              <p>No File</p>
            }
            @if(!state) {
              <p>Should Not Exist</p>
            }
        "#;

        let rendered = preprocess_template_directives(template, &vars);

        assert!(rendered.contains("<div><p>Hello World</p></div>"));
        assert!(rendered.contains(r#"<img src="avatar.png">"#));
        assert!(!rendered.contains("No File"));
        assert!(!rendered.contains("Should Not Exist"));
    }

    #[test]
    fn preprocess_template_directives_handles_for_loops_and_placeholder_expansion() {
        let mut vars = UiLangVariables::default();
        vars.set(
            "data",
            r#"{"users":[{"name":"Alice"},{"name":"Bob"}],"show":true}"#,
        );

        let template = r#"
            @for(user, idx in data.users) {
              @if(data.show && user.name.contains("o") || idx == 0) {
                <p>{{ idx }}:{{ user.name }}</p>
              }
            }
        "#;

        let rendered = preprocess_template_directives(template, &vars);

        assert!(rendered.contains("<p>0:Alice</p>"));
        assert!(rendered.contains("<p>1:Bob</p>"));
    }

    #[test]
    fn preprocess_template_directives_resolves_use_alias_and_wildcard() {
        let vars = UiLangVariables::default();
        let mut shared = UiSharedValues::default();
        shared.values.insert(
            "Player".to_string(),
            crate::lang::serde_json::from_str(r#"{"state":true,"name":"NetRunner"}"#).unwrap(),
        );

        let template = r#"
            @use "Player" as player;
            @if(player.state && player.name.startsWidth("Net")) {
              <p>Alias Works</p>
            }
            @use "Player" as *;
            @if(state && name.endsWidth("Runner")) {
              <p>Wildcard Works</p>
            }
        "#;

        let rendered = preprocess_template_directives_with_shared(template, &vars, &shared);

        assert!(rendered.contains("<p>Alias Works</p>"));
        assert!(rendered.contains("<p>Wildcard Works</p>"));
        assert!(!rendered.contains("@use"));
    }

    #[test]
    fn preprocess_template_directives_resolves_use_with_default_alias() {
        let vars = UiLangVariables::default();
        let mut shared = UiSharedValues::default();
        shared.values.insert(
            "DataPack".to_string(),
            crate::lang::serde_json::from_str(r#"{"version":"1.0.0","used":false}"#).unwrap(),
        );

        let template = r#"
            @use "DataPack";
            <p>Version: {{ data_pack.version }}</p>
            @if(!data_pack.used) {
              <p>Unused</p>
            }
        "#;

        let rendered = preprocess_template_directives_with_shared(template, &vars, &shared);

        assert!(rendered.contains("<p>Version: 1.0.0</p>"));
        assert!(rendered.contains("<p>Unused</p>"));
        assert!(!rendered.contains("@use"));
    }

    #[test]
    fn preprocess_template_directives_compares_shared_enum_variant_literals() {
        let vars = UiLangVariables::default();
        let mut shared = UiSharedValues::default();
        shared.values.insert(
            "DataState".to_string(),
            crate::lang::serde_json::Value::String("Inactive".to_string()),
        );

        let template = r#"
            @use "DataState";
            @if(data_state == DataState::Inactive) {
              <p>Inactive</p>
            } @else {
              <p>Active</p>
            }
        "#;

        let rendered = preprocess_template_directives_with_shared(template, &vars, &shared);

        assert!(rendered.contains("<p>Inactive</p>"));
        assert!(!rendered.contains("<p>Active</p>"));
    }

    #[test]
    fn preprocess_template_directives_resolves_use_path_with_default_alias() {
        let vars = UiLangVariables::default();
        let mut shared = UiSharedValues::default();
        shared.values.insert(
            "bevy_extended_ui_tests::data_structs::DataPack".to_string(),
            crate::lang::serde_json::from_str(r#"{"version":"1.0.0","data":[0,2,1]}"#).unwrap(),
        );
        shared
            .known_types
            .insert("bevy_extended_ui_tests::data_structs::DataPack".to_string());

        let template = r#"
            @use "crate::data_structs::DataPack";
            <p>Version: {{ data_pack.version }}</p>
            @for (entry in data_pack.get_data()) {
              <span>{{ entry }}</span>
            }
        "#;

        let rendered = preprocess_template_directives_with_shared(template, &vars, &shared);

        assert!(rendered.contains("<p>Version: 1.0.0</p>"));
        assert!(rendered.contains("<span>0</span>"));
        assert!(rendered.contains("<span>2</span>"));
        assert!(rendered.contains("<span>1</span>"));
        assert!(!rendered.contains("@use"));
    }

    #[test]
    fn preprocess_template_directives_resolves_grouped_use_paths() {
        let vars = UiLangVariables::default();
        let mut shared = UiSharedValues::default();
        shared.values.insert(
            "bevy_extended_ui_tests::data_structs::DataPack".to_string(),
            crate::lang::serde_json::from_str(r#"{"version":"1.0.0","used":false}"#).unwrap(),
        );
        shared.values.insert(
            "bevy_extended_ui_tests::data_structs::DataState".to_string(),
            crate::lang::serde_json::Value::String("Inactive".to_string()),
        );
        shared
            .known_types
            .insert("bevy_extended_ui_tests::data_structs::DataPack".to_string());
        shared
            .known_types
            .insert("bevy_extended_ui_tests::data_structs::DataState".to_string());

        let template = r#"
            @use "crate::data_structs::{DataState as hey, DataPack}";
            <p>Version: {{ data_pack.version }}</p>
            @if(!data_pack.used && hey == DataState::Inactive) {
              <p>Grouped Use Works</p>
            }
        "#;

        let rendered = preprocess_template_directives_with_shared(template, &vars, &shared);

        assert!(rendered.contains("<p>Version: 1.0.0</p>"));
        assert!(rendered.contains("<p>Grouped Use Works</p>"));
        assert!(!rendered.contains("@use"));
    }

    #[test]
    fn preprocess_template_directives_resolves_path_wildcard_use() {
        let vars = UiLangVariables::default();
        let mut shared = UiSharedValues::default();
        shared.values.insert(
            "bevy_extended_ui_tests::data_structs::DataPack".to_string(),
            crate::lang::serde_json::from_str(r#"{"version":"1.0.0","used":false}"#).unwrap(),
        );
        shared.values.insert(
            "bevy_extended_ui_tests::data_structs::DataState".to_string(),
            crate::lang::serde_json::Value::String("Inactive".to_string()),
        );

        let template = r#"
            @use "crate::data_structs::*";
            <p>Version: {{ data_pack.version }}</p>
            @if(!data_pack.used && data_state == DataState::Inactive) {
              <p>Wildcard Use Works</p>
            }
        "#;

        let rendered = preprocess_template_directives_with_shared(template, &vars, &shared);

        assert!(rendered.contains("<p>Version: 1.0.0</p>"));
        assert!(rendered.contains("<p>Wildcard Use Works</p>"));
        assert!(!rendered.contains("@use"));
    }

    #[test]
    fn preprocess_template_directives_interpolates_moustache_from_shared_alias() {
        let vars = UiLangVariables::default();
        let mut shared = UiSharedValues::default();
        shared.values.insert(
            "Player".to_string(),
            crate::lang::serde_json::from_str(r#"{"name":"NetRunner"}"#).unwrap(),
        );

        let template = r#"
            @use "Player" as player;
            <p>Player Name: {{ player.name }}</p>
        "#;

        let rendered = preprocess_template_directives_with_shared(template, &vars, &shared);
        assert!(rendered.contains("<p>Player Name: NetRunner</p>"));
    }

    #[test]
    fn preprocess_template_directives_auto_aliases_local_component_types() {
        let vars = UiLangVariables::default();
        let mut shared = UiSharedValues::default();
        shared.values.insert(
            "Player".to_string(),
            crate::lang::serde_json::from_str(r#"{"name":"NetRunner","state":true}"#).unwrap(),
        );
        shared.values.insert(
            "Info".to_string(),
            crate::lang::serde_json::from_str(r#"{"display":"Ready"}"#).unwrap(),
        );

        let template = r#"
            <p>{{ player.name }}</p>
            <span>{{ info.display }}</span>
            @if(player.state) { <b>Active</b> }
        "#;

        let rendered = preprocess_template_directives_with_shared_and_local_types(
            template,
            &vars,
            &shared,
            &["Player", "Info"],
        );

        assert!(rendered.contains("<p>NetRunner</p>"));
        assert!(rendered.contains("<span>Ready</span>"));
        assert!(rendered.contains("<b>Active</b>"));
    }

    #[test]
    fn preprocess_template_directives_interpolates_moustache_from_html_use_fields() {
        let vars = UiLangVariables::default();
        let mut shared = UiSharedValues::default();
        shared.values.insert(
            "Player".to_string(),
            crate::lang::serde_json::from_str(r#"{"name":"NetRunner","state":true}"#).unwrap(),
        );
        shared
            .auto_use_aliases
            .insert("player".to_string(), "Player".to_string());

        let template = r#"
            <p>Player Name: {{ name }}</p>
            @if(state) { <p>Enabled</p> }
        "#;

        let rendered = preprocess_template_directives_with_shared(template, &vars, &shared);
        assert!(rendered.contains("<p>Player Name: NetRunner</p>"));
        assert!(rendered.contains("<p>Enabled</p>"));
    }
}
