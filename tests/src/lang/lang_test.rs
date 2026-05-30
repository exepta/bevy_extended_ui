#[cfg(all(test, any(feature = "fluent", feature = "properties-lang")))]
mod tests {
    #[cfg(feature = "properties-lang")]
    use super::super::localize_html;
    use super::super::{UiLangVariables, resolve_placeholder};

    #[test]
    fn unresolved_reactive_placeholder_stays_unchanged() {
        let vars = UiLangVariables::default();
        let resolved = resolve_placeholder("user.name", |_| None, &vars);
        assert_eq!(resolved, None);
    }

    #[test]
    fn dotted_language_keys_can_still_be_localized() {
        let vars = UiLangVariables::default();
        let resolved = resolve_placeholder("app.title", |key| Some(format!("tr:{key}")), &vars);
        assert_eq!(resolved, Some("tr:app.title".to_string()));
    }

    #[cfg(feature = "properties-lang")]
    #[test]
    fn properties_localization_replaces_placeholders_from_assets() {
        let html = "<h2>{{ LANGUAGE_TITLE }}</h2><p>{{ WELCOME_START_TEXT %player_name% WELCOME_END_TEXT }}</p>";
        let mut vars = UiLangVariables::default();
        vars.set("player_name", "Tester");

        let out = localize_html(html, Some("de"), "assets/lang", &vars);

        assert!(out.contains("Sprachbeispiel"), "output: {out}");
        assert!(out.contains("Willkommen Tester !"), "output: {out}");
    }
}

#[cfg(test)]
mod shared_state_tests {
    use super::super::{UiLangVariables, UiSharedValues, serde_json, shared_values_fingerprint};

    #[test]
    fn ui_lang_variables_bool_helpers_roundtrip_and_toggle() {
        let mut vars = UiLangVariables::default();
        assert_eq!(vars.get_bool("state"), None);

        vars.set_bool("state", true);
        assert_eq!(vars.get_bool("state"), Some(true));

        let next = vars.toggle_bool("state", false);
        assert!(!next);
        assert_eq!(vars.get_bool("state"), Some(false));
    }

    #[test]
    fn ui_lang_variables_bool_helpers_accept_common_text_forms() {
        let mut vars = UiLangVariables::default();
        vars.set("a", "1");
        vars.set("b", "yes");
        vars.set("c", "off");
        vars.set("d", "maybe");

        assert_eq!(vars.get_bool("a"), Some(true));
        assert_eq!(vars.get_bool("b"), Some(true));
        assert_eq!(vars.get_bool("c"), Some(false));
        assert_eq!(vars.get_bool("d"), None);
    }

    #[test]
    fn ui_lang_variables_json_helpers_roundtrip() {
        let mut vars = UiLangVariables::default();
        let model = serde_json::json!({"name":"runner","enabled":true});

        vars.set_json("model", &model)
            .expect("json encode should work");
        let decoded: Option<serde_json::Value> = vars.get_json("model");
        assert_eq!(decoded, Some(model));
    }

    #[test]
    fn shared_values_fingerprint_changes_on_values_aliases_and_known_types() {
        let mut a = UiSharedValues::default();
        a.values
            .insert("Player".to_string(), serde_json::json!({"state": true}));
        a.auto_use_aliases
            .insert("player".to_string(), "Player".to_string());
        a.known_types.insert("Player".to_string());

        let mut b = a.clone();
        assert_eq!(shared_values_fingerprint(&a), shared_values_fingerprint(&b));

        b.values
            .insert("Player".to_string(), serde_json::json!({"state": false}));
        assert_ne!(shared_values_fingerprint(&a), shared_values_fingerprint(&b));

        b = a.clone();
        b.auto_use_aliases
            .insert("player_alt".to_string(), "Player".to_string());
        assert_ne!(shared_values_fingerprint(&a), shared_values_fingerprint(&b));

        b = a.clone();
        b.known_types.insert("Info".to_string());
        assert_ne!(shared_values_fingerprint(&a), shared_values_fingerprint(&b));
    }
}
