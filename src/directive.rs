//! Finding and parsing `{{#drawio ...}}` directives in chapter content.

use log::warn;
use pulldown_cmark::{Event, Options, Parser, Tag};
use regex::Regex;
use std::ops::Range;
use std::path::Path;
use std::sync::LazyLock;

/// Matches a directive, including a leading backslash if it is escaped.
/// Quoted attribute values may contain `}`.
static DIRECTIVE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"\\?\{\{#drawio((?:\s(?:"[^"]*"|[^"}])*)?)\}\}"#).unwrap()
});

/// Matches one `key=value` or `key="value"` attribute at the start of the input.
static ATTRIBUTE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"^\s+([A-Za-z][\w-]*)=(?:"([^"]*)"|([^\s"]+))"#).unwrap()
});

/// A `{{#drawio path="..." page=N alt="..."}}` directive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Directive {
    /// Byte range of the directive in the chapter content.
    pub range: Range<usize>,
    /// Path to the diagram, relative to the book root.
    pub path: String,
    /// Zero-based page index. Defaults to 0.
    pub page: u32,
    /// Alternative text for the image.
    pub alt: Option<String>,
}

/// Finds all directives in `content`.
///
/// Directives inside code blocks or inline code are skipped, so that a book
/// can document the syntax. Escaped directives (`\{{#drawio ...}}`) are
/// skipped too; mdBook's `links` preprocessor removes the backslash later, as
/// it does for its own `{{#include}}`.
pub fn find_directives(content: &str) -> Result<Vec<Directive>, String> {
    let code = code_ranges(content);
    let mut directives = Vec::new();
    for caps in DIRECTIVE_RE.captures_iter(content) {
        let m = caps.get(0).unwrap();
        if m.as_str().starts_with('\\') || code.iter().any(|r| r.contains(&m.start())) {
            continue;
        }
        let directive = parse_attributes(m.range(), &caps[1])
            .map_err(|e| format!("invalid directive `{}`: {e}", m.as_str()))?;
        directives.push(directive);
    }
    Ok(directives)
}

fn parse_attributes(range: Range<usize>, mut input: &str) -> Result<Directive, String> {
    let mut path = None;
    let mut page = 0;
    let mut alt = None;
    while !input.trim().is_empty() {
        let caps = ATTRIBUTE_RE
            .captures(input)
            .ok_or_else(|| format!("cannot parse `{}`", input.trim()))?;
        let value = caps.get(2).or(caps.get(3)).unwrap().as_str();
        match &caps[1] {
            "path" => path = Some(value.to_string()),
            "page" => {
                page = value
                    .parse()
                    .map_err(|_| format!("page must be a number, not `{value}`"))?
            }
            "alt" => alt = Some(value.to_string()),
            key => warn!("Ignoring unknown attribute `{key}` in drawio directive"),
        }
        input = &input[caps.get(0).unwrap().end()..];
    }
    let path = path.ok_or("missing `path` attribute")?;
    if Path::new(&path).file_stem().is_none() {
        return Err(format!("`{path}` is not a file path"));
    }
    Ok(Directive { range, path, page, alt })
}

/// Byte ranges of code blocks and inline code.
fn code_ranges(content: &str) -> Vec<Range<usize>> {
    // The extensions mdBook enables, as they change what is parsed as code.
    let options = Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_HEADING_ATTRIBUTES;
    Parser::new_ext(content, options)
        .into_offset_iter()
        .filter_map(|(event, range)| match event {
            Event::Start(Tag::CodeBlock(_)) | Event::Code(_) => Some(range),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paths(content: &str) -> Vec<String> {
        find_directives(content).unwrap().into_iter().map(|d| d.path).collect()
    }

    #[test]
    fn parses_attributes() {
        let content = r#"Text {{#drawio path="d/a.drawio" page=2 alt="A {b}, c"}} more"#;
        let d = find_directives(content).unwrap();
        assert_eq!(
            d,
            vec![Directive {
                range: 5..56,
                path: "d/a.drawio".into(),
                page: 2,
                alt: Some("A {b}, c".into()),
            }]
        );
        assert!(content[d[0].range.clone()].ends_with("}}"));
    }

    #[test]
    fn page_defaults_to_zero() {
        let d = find_directives(r#"{{#drawio path="a.drawio"}}"#).unwrap();
        assert_eq!(d[0].page, 0);
        assert_eq!(d[0].alt, None);
    }

    #[test]
    fn skips_code_and_escapes() {
        let content = r#"
{{#drawio path="real.drawio" page=0}}

`{{#drawio path="inline.drawio" page=0}}`

```md
{{#drawio path="fenced.drawio" page=0}}
```

    {{#drawio path="indented.drawio" page=0}}

\{{#drawio path="escaped.drawio" page=0}}
"#;
        assert_eq!(paths(content), vec!["real.drawio"]);
    }

    #[test]
    fn does_not_match_other_directives() {
        assert!(paths(r#"{{#drawiox path="a"}} {{#include a.md}}"#).is_empty());
    }

    #[test]
    fn rejects_invalid_directives() {
        assert!(find_directives(r#"{{#drawio page=0}}"#).is_err());
        assert!(find_directives(r#"{{#drawio path="a.drawio" page=x}}"#).is_err());
        assert!(find_directives(r#"{{#drawio path="a.drawio" "stray"}}"#).is_err());
    }
}
