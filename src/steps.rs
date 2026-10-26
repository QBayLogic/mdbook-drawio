//! Step-by-step diagrams (`steps="layers"`): each layer of a page is a step.
//!
//! Step N shows layers 0 to N. The later layers stay in the diagram with an
//! opacity of 0, because draw.io crops an export to the shapes it contains:
//! this way all steps have the same size and line up.

use base64::Engine;
use std::collections::HashMap;

/// The style appended to the cells of the layers that a step hides.
const HIDDEN_STYLE: &str = "opacity=0;textOpacity=0;";

/// The script and style for step-by-step diagrams, added once to each chapter
/// that has one.
pub const ASSETS: &str = concat!(
    "\n\n<style>\n",
    include_str!("steps.css"),
    "</style>\n<script>\n",
    include_str!("steps.js"),
    "</script>\n"
);

/// A page of a diagram, with its layers.
pub struct Layers {
    /// The `mxGraphModel` of the page, uncompressed.
    model: String,
    /// The names of the layers, bottom first.
    pub names: Vec<String>,
}

impl Layers {
    /// Reads the layers of `page` (zero-based) of a draw.io diagram.
    pub fn read(bytes: &[u8], page: u32) -> Result<Layers, String> {
        let text = std::str::from_utf8(bytes).map_err(|e| format!("not valid UTF-8: {e}"))?;
        let doc =
            roxmltree::Document::parse(text).map_err(|e| format!("not a valid diagram: {e}"))?;
        let root = doc.root_element();
        let model = match root.tag_name().name() {
            "mxfile" => {
                let diagrams: Vec<_> =
                    root.children().filter(|n| n.has_tag_name("diagram")).collect();
                let diagram = diagrams.get(page as usize).ok_or_else(|| {
                    format!("has no page {page} (it has {}, numbered from 0)", diagrams.len())
                })?;
                match diagram.children().find(|n| n.has_tag_name("mxGraphModel")) {
                    Some(model) => text[model.range()].to_string(),
                    None => decompress(diagram.text().unwrap_or("").trim())?,
                }
            }
            "mxGraphModel" if page == 0 => text.to_string(),
            "mxGraphModel" => return Err(format!("has no page {page} (it has 1)")),
            _ => return Err("`steps` needs a draw.io diagram, such as a .drawio file".into()),
        };
        let names = {
            let doc = roxmltree::Document::parse(&model)
                .map_err(|e| format!("not a valid diagram: {e}"))?;
            let cells = cells(&doc);
            let root_id = cells.iter().find(|c| c.parent.is_none()).map(|c| c.id);
            cells
                .iter()
                .filter(|c| c.parent.is_some() && c.parent == root_id)
                .enumerate()
                .map(|(i, c)| match c.label {
                    Some(label) if !label.trim().is_empty() => label.trim().to_string(),
                    _ if i == 0 => "Background".to_string(),
                    _ => format!("Layer {}", i + 1),
                })
                .collect()
        };
        Ok(Layers { model, names })
    }

    /// A one-page diagram that shows layers 0 to `step`.
    pub fn step_diagram(&self, step: usize) -> String {
        // `read` parsed the model already.
        let doc = roxmltree::Document::parse(&self.model).unwrap();
        let cells = cells(&doc);
        let parents: HashMap<&str, Option<&str>> = cells.iter().map(|c| (c.id, c.parent)).collect();
        let root_id = cells.iter().find(|c| c.parent.is_none()).map(|c| c.id);
        let layers: HashMap<&str, usize> = cells
            .iter()
            .filter(|c| c.parent.is_some() && c.parent == root_id)
            .enumerate()
            .map(|(i, c)| (c.id, i))
            .collect();
        // The layer of a cell: follow its parents up to a layer.
        let layer_of = |cell: &Cell| {
            let mut id = cell.id;
            for _ in 0..parents.len() {
                if let Some(&layer) = layers.get(id) {
                    return Some(layer);
                }
                id = parents.get(id).copied().flatten()?;
            }
            None
        };

        let mut edits = Vec::new();
        for cell in &cells {
            if layers.contains_key(cell.id) || layer_of(cell).is_none_or(|l| l <= step) {
                continue;
            }
            match cell.node.attribute_node("style") {
                Some(style) => edits.push((style.range_value().end, format!(";{HIDDEN_STYLE}"))),
                None => {
                    let at = cell.node.range().start + "<mxCell".len();
                    edits.push((at, format!(" style=\"{HIDDEN_STYLE}\"")));
                }
            }
        }
        let mut model = self.model.clone();
        edits.sort_by_key(|(at, _)| std::cmp::Reverse(*at));
        for (at, text) in edits {
            model.insert_str(at, &text);
        }
        format!("<mxfile><diagram id=\"step\" name=\"Step {}\">{model}</diagram></mxfile>", step + 1)
    }
}

/// A cell of a diagram.
struct Cell<'a, 'input> {
    id: &'a str,
    parent: Option<&'a str>,
    label: Option<&'a str>,
    /// The `mxCell` element, which holds the style.
    node: roxmltree::Node<'a, 'input>,
}

/// The cells of an `mxGraphModel`. A cell with metadata is an `mxCell` inside
/// an `object` or `UserObject` element, which holds the id and the label.
fn cells<'a, 'input>(doc: &'a roxmltree::Document<'input>) -> Vec<Cell<'a, 'input>> {
    doc.descendants()
        .filter(|n| n.has_tag_name("mxCell"))
        .filter_map(|node| {
            let wrapper = node
                .parent_element()
                .filter(|p| p.has_tag_name("object") || p.has_tag_name("UserObject"));
            let (id, label) = match wrapper {
                Some(w) => (w.attribute("id")?, w.attribute("label")),
                None => (node.attribute("id")?, node.attribute("value")),
            };
            Some(Cell { id, parent: node.attribute("parent"), label, node })
        })
        .collect()
}

/// Decompresses a page that draw.io stored compressed: Base64 of raw deflate
/// of the URL-encoded XML.
fn decompress(text: &str) -> Result<String, String> {
    let invalid = |e: &dyn std::fmt::Display| format!("cannot decompress page: {e}");
    let bytes = base64::engine::general_purpose::STANDARD.decode(text).map_err(|e| invalid(&e))?;
    let inflated =
        miniz_oxide::inflate::decompress_to_vec(&bytes).map_err(|e| invalid(&format!("{e:?}")))?;
    let mut decoded = Vec::with_capacity(inflated.len());
    let mut i = 0;
    while i < inflated.len() {
        let hex = inflated.get(i + 1..i + 3).and_then(|h| std::str::from_utf8(h).ok());
        match (inflated[i], hex.and_then(|h| u8::from_str_radix(h, 16).ok())) {
            (b'%', Some(byte)) => {
                decoded.push(byte);
                i += 3;
            }
            (byte, _) => {
                decoded.push(byte);
                i += 1;
            }
        }
    }
    String::from_utf8(decoded).map_err(|e| invalid(&e))
}

/// The HTML for a step-by-step diagram: all steps stacked, and controls that
/// the script in [`ASSETS`] enables. Without the script, the last step (the
/// whole diagram) is shown.
pub fn figure_html(alt: &str, links: &[String], names: &[String]) -> String {
    let mut html = String::from(
        "<figure class=\"mdbook-drawio-steps\" tabindex=\"0\">\n<div class=\"mdbook-drawio-frames\">\n",
    );
    for (i, (link, name)) in links.iter().zip(names).enumerate() {
        let alt = format!("{alt}, step {} of {}: {name}", i + 1, links.len());
        html.push_str(&format!(
            "<img src=\"{}\" alt=\"{}\" data-label=\"{}\">\n",
            escape_html(link),
            escape_html(&alt),
            escape_html(name)
        ));
    }
    html.push_str(concat!(
        "</div>\n<figcaption class=\"mdbook-drawio-controls\" hidden>\n",
        "<button type=\"button\" data-action=\"prev\" aria-label=\"Previous step\">&#x2039;</button>\n",
        "<button type=\"button\" data-action=\"play\" aria-label=\"Play\">&#x25B6;</button>\n",
        "<button type=\"button\" data-action=\"next\" aria-label=\"Next step\">&#x203A;</button>\n",
        "<span class=\"mdbook-drawio-label\" aria-live=\"polite\"></span>\n",
        "</figcaption>\n</figure>"
    ));
    html
}

fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;

    const DIAGRAM: &str = r#"<mxfile><diagram id="p"><mxGraphModel><root>
<mxCell id="0"/>
<mxCell id="1" parent="0"/>
<mxCell id="a" style="rounded=1;" vertex="1" parent="1"/>
<mxCell id="L2" value="Second" parent="0"/>
<mxCell id="g" vertex="1" parent="L2"/>
<mxCell id="b" style="rounded=1" vertex="1" parent="g"/>
<UserObject id="L3" label="Third"><mxCell parent="0"/></UserObject>
<UserObject id="c" label="C"><mxCell vertex="1" parent="L3"/></UserObject>
</root></mxGraphModel></diagram></mxfile>"#;

    #[test]
    fn reads_layer_names() {
        let layers = Layers::read(DIAGRAM.as_bytes(), 0).unwrap();
        assert_eq!(layers.names, vec!["Background", "Second", "Third"]);
        assert!(Layers::read(DIAGRAM.as_bytes(), 1).is_err());
    }

    #[test]
    fn hides_later_layers() {
        let layers = Layers::read(DIAGRAM.as_bytes(), 0).unwrap();
        let step = layers.step_diagram(0);
        assert!(step.contains(r#"<mxCell id="a" style="rounded=1;" vertex"#));
        assert!(step.contains(r#"<mxCell style="opacity=0;textOpacity=0;" id="g" vertex"#));
        assert!(step.contains(r#"style="rounded=1;opacity=0;textOpacity=0;""#));
        assert!(step.contains(r#"<mxCell style="opacity=0;textOpacity=0;" vertex="1" parent="L3"/>"#));
        // Layers themselves are not changed.
        assert!(step.contains(r#"<mxCell id="L2" value="Second" parent="0"/>"#));
        roxmltree::Document::parse(&step).unwrap();

        let last = layers.step_diagram(2);
        assert!(!last.contains("opacity=0"));
    }

    #[test]
    fn reads_compressed_pages() {
        let model = r#"<mxGraphModel><root><mxCell id="0"/><mxCell id="1" value="Base layer" parent="0"/></root></mxGraphModel>"#;
        let encoded: String = model
            .bytes()
            .map(|b| match b {
                b'<' | b'>' | b'"' | b' ' | b'=' | b'/' => format!("%{b:02X}"),
                _ => (b as char).to_string(),
            })
            .collect();
        let compressed = miniz_oxide::deflate::compress_to_vec(encoded.as_bytes(), 6);
        let text = base64::engine::general_purpose::STANDARD.encode(compressed);
        let file = format!(r#"<mxfile><diagram id="p">{text}</diagram></mxfile>"#);
        let layers = Layers::read(file.as_bytes(), 0).unwrap();
        assert_eq!(layers.names, vec!["Base layer"]);
        assert!(layers.step_diagram(0).contains(model));
    }

    #[test]
    fn figure_escapes_text() {
        let html = figure_html("A \"b\"", &["x.svg".into()], &["<one>".into()]);
        assert!(html.contains(r#"alt="A &quot;b&quot;, step 1 of 1: &lt;one&gt;""#));
        assert!(!html.contains("\n\n"));
    }
}
