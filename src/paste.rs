use scraper::{Html, Selector};

struct PasteContainer {

}

struct Paste {
    title: Option<String>,
}

pub fn get_csrftoken(dom: Html) -> String {
    dom.select(&Selector::parse("meta[name=csrf-token]").unwrap())
        .next()
        .unwrap()
        .value()
        .attr("content")
        .unwrap()
        .to_owned()
}

pub fn parse_paste_container(parent_selector: &str, dom: &Html) -> Vec<String> {
}

pub fn parse_paste() {

}