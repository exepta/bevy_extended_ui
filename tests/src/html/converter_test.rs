#[cfg(test)]
mod tests {
    use super::super::HtmlWidgetNode;
    use super::super::converter::{
        extract_inner_bindings, parse_html_fragment, parse_inner_content,
        preprocess_template_directives, preprocess_template_directives_with_shared,
        preprocess_template_directives_with_shared_and_local_types,
    };
    use crate::lang::{UiLangVariables, UiSharedValues};
    use kuchiki::traits::TendrilSink;

    /// Returns the cells of the first parsed `Table` node, asserting the fragment
    /// produced exactly one table and no stray top-level `<tr>` nodes.
    fn table_cells(html: &str) -> (usize, Vec<(bool, usize, usize, String)>) {
        let nodes = parse_html_fragment(html);
        let tables: Vec<&HtmlWidgetNode> = nodes
            .iter()
            .filter(|n| matches!(n, HtmlWidgetNode::Table(..)))
            .collect();
        assert_eq!(tables.len(), 1, "expected exactly one Table node");

        let HtmlWidgetNode::Table(table, _, _, cells, _, _, _) = tables[0] else {
            unreachable!()
        };

        // <tr> is flattened: every direct table child must be a TableCell, never a
        // row node. `parse_html_node` returns `None` for `<tr>`, so a row node could
        // not appear here — this assertion locks that contract.
        let parsed = cells
            .iter()
            .map(|cell| {
                let HtmlWidgetNode::TableCell(c, meta, _, _, _, _, _) = cell else {
                    panic!("table child was not a TableCell: {cell:?}");
                };
                (
                    c.header,
                    c.row,
                    c.col,
                    meta.inner_content.inner_text().trim().to_string(),
                )
            })
            .collect();

        (table.columns, parsed)
    }

    /// Returns the raw `TableCell` nodes of the single table in `html` (cells kept
    /// as `HtmlWidgetNode` so callers can inspect their children).
    fn first_table_cells(html: &str) -> Vec<HtmlWidgetNode> {
        let nodes = parse_html_fragment(html);
        let table = nodes
            .into_iter()
            .find(|n| matches!(n, HtmlWidgetNode::Table(..)))
            .expect("expected a Table node");
        let HtmlWidgetNode::Table(_, _, _, cells, _, _, _) = table else {
            unreachable!()
        };
        cells
    }

    #[test]
    fn parse_cell_marks_header_flag() {
        // A bare `<td>`/`<th>` is foster-parented away by the HTML5 parser, so the
        // `td|th` arm is reached only inside a table — exercise it there.
        let cells = first_table_cells("<table><tr><td>Hi</td><th>Yo</th></tr></table>");
        assert_eq!(cells.len(), 2);
        match (&cells[0], &cells[1]) {
            (
                HtmlWidgetNode::TableCell(data, _, _, _, _, _, _),
                HtmlWidgetNode::TableCell(header, _, _, _, _, _, _),
            ) => {
                assert!(!data.header);
                assert_eq!((data.row, data.col), (0, 0));
                assert!(header.header);
                assert_eq!((header.row, header.col), (0, 1));
            }
            other => panic!("expected two TableCells, got {other:?}"),
        }
    }

    #[test]
    fn parse_cell_with_nested_button_keeps_child_node() {
        let cells = first_table_cells(
            r#"<table><tr><td><button onclick="go">X</button></td></tr></table>"#,
        );
        match &cells[0] {
            HtmlWidgetNode::TableCell(_, _, _, children, _, _, _) => {
                assert_eq!(children.len(), 1);
                assert!(matches!(children[0], HtmlWidgetNode::Button(..)));
            }
            other => panic!("expected TableCell, got {other:?}"),
        }
    }

    #[test]
    fn parse_table_2x3_yields_six_cells_and_three_columns() {
        let html = r#"
            <table>
                <tr><th>A</th><th>B</th><th>C</th></tr>
                <tr><td>1</td><td>2</td><td>3</td></tr>
            </table>
        "#;
        let (columns, cells) = table_cells(html);
        assert_eq!(columns, 3, "column count is the widest row");
        assert_eq!(cells.len(), 6);

        // Row 0 is the header row, row 1 data; indices stamped per cell.
        assert_eq!(
            cells,
            vec![
                (true, 0, 0, "A".to_string()),
                (true, 0, 1, "B".to_string()),
                (true, 0, 2, "C".to_string()),
                (false, 1, 0, "1".to_string()),
                (false, 1, 1, "2".to_string()),
                (false, 1, 2, "3".to_string()),
            ]
        );
    }

    #[test]
    fn parse_table_text_only_cell_exposes_inner_content() {
        let (_, cells) = table_cells("<table><tr><td>Hello</td></tr></table>");
        assert_eq!(cells.len(), 1);
        assert_eq!(cells[0].3, "Hello");
    }

    #[test]
    fn parse_table_skips_whitespace_between_rows_and_cells() {
        // Newlines/spaces between <tr> and between <td> must not create cells.
        let html = "<table>\n  <tr>\n    <td>1</td>\n    <td>2</td>\n  </tr>\n</table>";
        let (columns, cells) = table_cells(html);
        assert_eq!(columns, 2);
        assert_eq!(cells.len(), 2, "whitespace must not become cells");
    }

    #[test]
    fn parse_table_ignores_stray_non_structural_tags() {
        // A stray non-<tr> child of <table>, and a stray non-cell child of <tr>.
        let html = r#"
            <table>
                <div>ignored</div>
                <tr><td>1</td><span>nope</span><td>2</td></tr>
            </table>
        "#;
        let (columns, cells) = table_cells(html);
        assert_eq!(columns, 2, "stray tags do not count as cells");
        assert_eq!(cells.len(), 2);
        assert_eq!(cells[0].3, "1");
        assert_eq!(cells[1].3, "2");
        assert_eq!(cells[1].2, 1, "second cell keeps col index 1");
    }

    #[test]
    fn parse_table_ragged_rows_keep_max_columns_and_own_indices() {
        let html = r#"
            <table>
                <tr><td>a</td><td>b</td><td>c</td></tr>
                <tr><td>d</td><td>e</td></tr>
            </table>
        "#;
        let (columns, cells) = table_cells(html);
        assert_eq!(columns, 3, "widest row sets the column count");
        assert_eq!(cells.len(), 5);
        // Row 1's two cells sit at col 0 and 1; col 2 is simply unoccupied.
        assert_eq!(cells[3], (false, 1, 0, "d".to_string()));
        assert_eq!(cells[4], (false, 1, 1, "e".to_string()));
    }

    #[test]
    fn parse_table_stamps_section_and_class_from_wrappers() {
        use crate::widgets::TableSection;

        let html = r#"
            <table>
                <thead><tr><th>H</th></tr></thead>
                <tbody><tr><td>B</td></tr></tbody>
                <tfoot><tr><td>F</td></tr></tfoot>
            </table>
        "#;
        let cells = first_table_cells(html);
        assert_eq!(cells.len(), 3);

        let section_and_class: Vec<(TableSection, Vec<String>)> = cells
            .iter()
            .map(|cell| {
                let HtmlWidgetNode::TableCell(c, meta, _, _, _, _, _) = cell else {
                    panic!("expected TableCell, got {cell:?}");
                };
                (c.section, meta.class.clone().unwrap_or_default())
            })
            .collect();

        assert_eq!(section_and_class[0].0, TableSection::Head);
        assert!(section_and_class[0].1.contains(&"thead".to_string()));
        assert_eq!(section_and_class[1].0, TableSection::Body);
        assert!(section_and_class[1].1.contains(&"tbody".to_string()));
        assert_eq!(section_and_class[2].0, TableSection::Foot);
        assert!(section_and_class[2].1.contains(&"tfoot".to_string()));
    }

    #[test]
    fn parse_table_bare_rows_default_to_body_section() {
        use crate::widgets::TableSection;

        // A `<tr>` directly under `<table>` (implicit tbody) is the Body section,
        // and its class carries `tbody`.
        let cells = first_table_cells("<table><tr><td>x</td></tr></table>");
        let HtmlWidgetNode::TableCell(c, meta, _, _, _, _, _) = &cells[0] else {
            panic!("expected TableCell");
        };
        assert_eq!(c.section, TableSection::Body);
        assert!(
            meta.class
                .clone()
                .unwrap_or_default()
                .contains(&"tbody".to_string())
        );
    }

    #[test]
    fn parse_table_section_class_preserves_author_classes() {
        use crate::widgets::TableSection;

        let cells =
            first_table_cells(r#"<table><thead><tr><th class="hd">H</th></tr></thead></table>"#);
        let HtmlWidgetNode::TableCell(c, meta, _, _, _, _, _) = &cells[0] else {
            panic!("expected TableCell");
        };
        assert_eq!(c.section, TableSection::Head);
        let classes = meta.class.clone().unwrap_or_default();
        assert!(classes.contains(&"hd".to_string()), "author class kept");
        assert!(
            classes.contains(&"thead".to_string()),
            "section class added"
        );
    }

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
