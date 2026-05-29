#[cfg(test)]
mod tests {
    use super::super::converter::{extract_inner_bindings, parse_inner_content};
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
}
