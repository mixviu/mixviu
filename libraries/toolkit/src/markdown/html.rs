// SPDX-License-Identifier: MIT OR Apache-2.0
//! Inert HTML export of the native Markdown dialect. Raw HTML stays text,
//! images become alt text and only the toolkit's openable links survive.
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
pub fn escape(text: &str) -> String {
    let mut out = String::new();
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            ch => out.push(ch),
        }
    }
    out
}
pub fn export(source: &str) -> String {
    let options =
        Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS;
    let mut links = Vec::new();
    let events = Parser::new_ext(source, options).filter_map(|event| match event {
        Event::Html(text) | Event::InlineHtml(text) => {
            Some(Event::InlineHtml(escape(&text).into()))
        }
        Event::Start(Tag::HtmlBlock) => Some(Event::Start(Tag::Paragraph)),
        Event::End(TagEnd::HtmlBlock) => Some(Event::End(TagEnd::Paragraph)),
        Event::Start(Tag::Image { .. }) | Event::End(TagEnd::Image) => None,
        Event::Start(Tag::Link { ref dest_url, .. }) => {
            let keep = super::openable(dest_url);
            links.push(keep);
            keep.then_some(event)
        }
        Event::End(TagEnd::Link) => links.pop().unwrap_or(false).then_some(event),
        other => Some(other),
    });
    let mut out = String::new();
    pulldown_cmark::html::push_html(&mut out, events);
    out
}
#[cfg(test)]
mod tests {
    #[test]
    fn export_keeps_markup_inert_images_local_and_links_openable() {
        let html = super::export(
            "# Heading\n\n<div onclick=\"x\">raw</div>\n\n![a cat](https://track.example.test/a.png) [ok](https://example.test/) [bad](javascript:alert(1)) [file](file:///etc/passwd)",
        );
        assert!(html.contains("<h1>Heading</h1>"));
        assert!(html.contains("&lt;div onclick=&quot;x&quot;&gt;raw&lt;/div&gt;"));
        assert!(!html.contains("<img") && !html.contains("track.example.test"));
        assert!(html.contains("a cat"));
        assert!(html.contains("href=\"https://example.test/\""));
        assert!(!html.contains("javascript:") && !html.contains("file://"));
    }
}
