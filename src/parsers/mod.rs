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

pub fn parse_date(date: &str) -> Result<i64, Box<dyn std::error::Error>> {
    let start_index = date.find(" of")
        .ok_or("Date string missing ' of' marker")?
        .checked_sub(2)
        .ok_or("Invalid date format")?;
    let end_index = start_index + 5;

    if end_index > date.len() {
        return Err("Date string too short".into());
    }

    let parsed_date = format!("{}{}", &date[..start_index], &date[end_index..])
        .replace("CDT", "-0500");

    let timestamp = DateTime::parse_from_str(&parsed_date, "%A %e %B %Y %r %z")?
        .timestamp();
    
    Ok(timestamp)
}