#[cfg(test)]
mod tests {
    use super::super::*;
    use bevy::asset::AssetPlugin;
    use bevy::prelude::*;
    use std::path::PathBuf;

    #[test]
    fn css_loader_extensions_are_correct() {
        let loader = CssLoader;
        assert_eq!(loader.extensions(), &["css"]);
    }

    #[test]
    fn html_loader_extensions_are_correct() {
        let loader = HtmlLoader;
        assert_eq!(loader.extensions(), &["html", "htm"]);
    }

    #[test]
    fn extract_attr_reads_double_quoted_values() {
        let tag = r#" rel="stylesheet" href="examples/base.css" "#;
        let value = extract_attr(tag, "href");
        assert_eq!(value.as_deref(), Some("examples/base.css"));
    }

    #[test]
    fn extract_attr_reads_single_quoted_values() {
        let tag = " rel='stylesheet' src='styles/main.css' ";
        let value = extract_attr(tag, "src");
        assert_eq!(value.as_deref(), Some("styles/main.css"));
    }

    #[test]
    fn extract_attr_returns_none_for_unquoted_values() {
        let tag = " rel=stylesheet href=examples/base.css ";
        assert_eq!(extract_attr(tag, "href"), None);
    }

    #[test]
    fn extract_attr_returns_none_for_missing_attr() {
        let tag = r#" rel="stylesheet" href="examples/base.css" "#;
        assert_eq!(extract_attr(tag, "src"), None);
    }

    #[test]
    fn extract_css_links_lenient_extracts_href_and_src_from_stylesheets() {
        let html = r#"
            <html>
              <head>
                <link rel="stylesheet" href="a.css">
                <link rel='stylesheet' src='b.css'>
                <link ref="text/css" href="c.css">
                <link rel="preload" href="ignore.css">
              </head>
            </html>
        "#;

        let out = extract_css_links_lenient(html);
        assert_eq!(out, vec!["a.css", "b.css", "c.css"]);
    }

    #[test]
    fn extract_css_links_lenient_returns_empty_when_no_stylesheet_found() {
        let html = r#"
            <html>
              <head>
                <link rel="preload" href="x.css">
                <script src="app.js"></script>
              </head>
            </html>
        "#;

        let out = extract_css_links_lenient(html);
        assert!(out.is_empty());
    }

    #[test]
    fn resolve_relative_joins_relative_paths_and_trims_whitespace() {
        let base = PathBuf::from("assets/examples");
        let out = resolve_relative(&base, "  ui/base.css  ");
        assert_eq!(out, PathBuf::from("assets/examples/ui/base.css"));
    }

    #[test]
    fn resolve_relative_keeps_absolute_paths() {
        let base = PathBuf::from("assets/examples");

        #[cfg(unix)]
        {
            let out = resolve_relative(&base, "/tmp/main.css");
            assert_eq!(out, PathBuf::from("/tmp/main.css"));
        }

        #[cfg(windows)]
        {
            let out = resolve_relative(&base, "C:\\temp\\main.css");
            assert_eq!(out, PathBuf::from("C:\\temp\\main.css"));
        }
    }

    #[test]
    fn extended_io_plugin_registers_default_css_asset() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default(), ExtendedIoPlugin));

        // Run startup schedule.
        app.update();

        let handle = app.world().resource::<DefaultCssHandle>().0.clone();
        let css_assets = app.world().resource::<Assets<CssAsset>>();
        let asset = css_assets
            .get(&handle)
            .expect("Default CssAsset should exist in asset storage");

        assert_eq!(asset.text, DEFAULT_UI_CSS_TEXT);
    }
}
