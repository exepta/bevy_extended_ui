#[cfg(test)]
mod tests {
    use super::super::converter::{
        extract_inner_bindings, parse_inner_content, preprocess_template_directives,
    };
    use crate::lang::UiLangVariables;
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

        let template = r#"
            @if(data.username.startsWidth("Net") && client.id == 42) {
              <div><p>Hello World</p></div>
            }
            @if(!state) {
              <p>Should Not Exist</p>
            }
        "#;

        let rendered = preprocess_template_directives(template, &vars);

        assert!(rendered.contains("<div><p>Hello World</p></div>"));
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
}
