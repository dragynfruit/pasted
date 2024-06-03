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