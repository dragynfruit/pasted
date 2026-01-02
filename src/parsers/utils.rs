use once_cell::sync::Lazy;
use scraper::{ElementRef, Selector};

// Pre-compiled common selectors to avoid runtime parsing
pub static SELECTOR_META_OG_URL: Lazy<Selector> =
    Lazy::new(|| Selector::parse("meta[property='og:url']").expect("Valid CSS selector"));
pub static SELECTOR_USER_ICON_IMG: Lazy<Selector> =
    Lazy::new(|| Selector::parse(".user-icon>img").expect("Valid CSS selector"));

/// Helper function to safely get text content from an element
pub fn safe_text_content(element: Option<ElementRef>) -> String {
    element
        .map(|e| e.text().collect::<String>().trim().to_owned())
        .unwrap_or_default()
}

/// Helper function to safely get an attribute from an element
pub fn safe_attr_content(element: Option<ElementRef>, attr: &str) -> String {
    element
        .and_then(|el| el.value().attr(attr))
        .unwrap_or_default()
        .to_owned()
}

/// Helper function to safely parse a CSS selector
pub fn safe_selector(selector_str: &str) -> Result<Selector, String> {
    Selector::parse(selector_str)
        .map_err(|e| format!("Invalid CSS selector '{}': {:?}", selector_str, e))
}

/// Helper function to safely select the first element matching a selector
pub fn safe_select<'a>(
    parent: &'a ElementRef,
    selector: &Selector,
) -> Option<ElementRef<'a>> {
    parent.select(selector).next()
}

/// Helper function to safely parse a number from text with a default fallback
pub fn safe_parse_number<T: std::str::FromStr>(text: &str) -> T
where
    T: Default,
{
    text.trim()
        .replace(",", "")
        .parse()
        .unwrap_or_default()
}
