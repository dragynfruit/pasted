use once_cell::sync::Lazy;
use scraper::{ElementRef, Html, Selector};
use serde::Serialize;

use crate::constants::URL;

use super::{FromElement, FromHtml};

// Pre-compiled selectors to avoid unwrap() calls
static SELECTOR_META_OG_URL: Lazy<Selector> =
    Lazy::new(|| Selector::parse("meta[property='og:url']").expect("Valid CSS selector"));
static SELECTOR_ARCHIVE_TABLE: Lazy<Selector> =
    Lazy::new(|| Selector::parse(".archive-table").expect("Valid CSS selector"));
static SELECTOR_MAINTABLE_TR: Lazy<Selector> =
    Lazy::new(|| Selector::parse(".maintable>tbody>tr").expect("Valid CSS selector"));
static SELECTOR_TD_CHILD_1_A: Lazy<Selector> =
    Lazy::new(|| Selector::parse("td:nth-child(1)>a").expect("Valid CSS selector"));
static SELECTOR_TD_CHILD_2: Lazy<Selector> =
    Lazy::new(|| Selector::parse("td:nth-child(2)").expect("Valid CSS selector"));
static SELECTOR_TD_CHILD_3_A: Lazy<Selector> =
    Lazy::new(|| Selector::parse("td:nth-child(3)>a").expect("Valid CSS selector"));

// Helper function to safely get text content from an element
fn safe_text_content(element: Option<ElementRef>) -> String {
    element
        .map(|e| e.text().collect::<String>().trim().to_owned())
        .unwrap_or_default()
}

// Helper function to safely get an attribute from an element
fn safe_attr_content(element: Option<ElementRef>, attr: &str) -> String {
    element
        .and_then(|el| el.value().attr(attr))
        .unwrap_or_default()
        .to_owned()
}

#[derive(Serialize)]
pub struct Archive {
    id: String,
    title: String,
    age: String,
    format: String,
}

impl FromElement for Archive {
    fn from_element(parent: &ElementRef) -> Self {
        let id_link = parent.select(&SELECTOR_TD_CHILD_1_A).next();
        let id = safe_attr_content(id_link, "href").replace("/", "");

        let title = safe_text_content(id_link);

        let age = safe_text_content(parent.select(&SELECTOR_TD_CHILD_2).next());

        let format_link = parent.select(&SELECTOR_TD_CHILD_3_A).next();
        let format = safe_attr_content(format_link, "href").replace("/archive/", "");

        Archive {
            id,
            title,
            age,
            format,
        }
    }
}

#[derive(Serialize)]
pub struct ArchivePage {
    format: Option<String>,
    archives: Vec<Archive>,
}

impl FromHtml for ArchivePage {
    fn from_html(dom: &Html) -> Self {
        let meta_element = dom.select(&SELECTOR_META_OG_URL).next();
        let format = none_if_empty(
            safe_attr_content(meta_element, "content")
                .replace(&format!("{URL}/archive"), "")
                .replace("/", ""),
        );

        let parent = dom.select(&SELECTOR_ARCHIVE_TABLE).next();

        let archives = match parent {
            Some(parent_elem) => parent_elem
                .select(&SELECTOR_MAINTABLE_TR)
                .enumerate()
                .filter(|&(i, _)| i != 0)
                .map(|(_, v)| Archive::from_element(&v))
                .collect::<Vec<Archive>>(),
            None => Vec::new(),
        };

        ArchivePage { format, archives }
    }
}

fn none_if_empty(string: String) -> Option<String> {
    if string.is_empty() {
        None
    } else {
        Some(string)
    }
}

#[cfg(test)]
mod tests {
    use scraper::{Html, Selector};

    use super::*;

    #[test]
    fn test_parse_archive_with_missing_elements() {
        // Test Archive::from_element with missing elements
        let dom = Html::parse_document(
            r#"
            <html>
            <body>
            <table>
            <tbody>
            <tr>
                <td></td>
                <td></td>
                <td></td>
            </tr>
            </tbody>
            </table>
            </body>
            </html>
        "#,
        );

        let element = dom
            .select(&Selector::parse("tr").unwrap())
            .next()
            .unwrap();

        let archive = Archive::from_element(&element);

        // Should not panic and should return default values
        assert_eq!(archive.id, "");
        assert_eq!(archive.title, "");
        assert_eq!(archive.age, "");
        assert_eq!(archive.format, "");
    }

    #[test]
    fn test_parse_archive_page_with_missing_table() {
        // Test ArchivePage::from_html with missing .archive-table
        let dom = Html::parse_document(
            r#"
            <html>
                <head>
                    <meta property="og:url" content="https://pastebin.com/archive">
                </head>
                <body>
                    <div>No archive table here</div>
                </body>
            </html>
        "#,
        );

        let archive_page = ArchivePage::from_html(&dom);

        // Should not panic and should return empty archives
        assert_eq!(archive_page.format, None);
        assert_eq!(archive_page.archives.len(), 0);
    }

    #[test]
    fn test_parse_archive_page_with_missing_meta() {
        // Test ArchivePage::from_html with missing meta tag
        let dom = Html::parse_document(
            r#"
            <html>
                <body>
                    <div class="archive-table">
                        <table class="maintable">
                            <tbody>
                                <tr><th>Header</th></tr>
                            </tbody>
                        </table>
                    </div>
                </body>
            </html>
        "#,
        );

        let archive_page = ArchivePage::from_html(&dom);

        // Should not panic
        assert_eq!(archive_page.format, None);
    }
}
