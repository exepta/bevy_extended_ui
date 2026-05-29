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
