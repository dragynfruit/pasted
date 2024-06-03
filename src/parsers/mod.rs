use chrono::DateTime;
use scraper::{ElementRef, Html};

pub mod archive;
pub mod paste;
pub mod user;

pub trait FromHtml {
    fn from_html(dom: &Html) -> Self;
}

pub trait FromElement {
    fn from_element(parent: &ElementRef) -> Self;
}

pub fn parse_date(date: &str) -> i64 {
    let start_index = date.find(" of").unwrap() - 2;
    let end_index = start_index + 5;

    let date = format!("{}{}", &date[..start_index], &date[end_index..]).replace("CDT", "-0500");

    DateTime::parse_from_str(&date, "%A %e %B %Y %r %z")
        .unwrap()
        .timestamp()
}