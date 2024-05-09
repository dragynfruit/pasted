use scraper::{Html, Selector};

pub fn get_csrftoken(dom: Html) -> String {
    dom
        .select(&Selector::parse("meta[name=csrf-token]").unwrap())
        .next()
        .unwrap()
        .value()
        .attr("content")
        .unwrap()
        .to_owned()
}
