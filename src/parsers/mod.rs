use chrono::DateTime;
use scraper::{ElementRef, Html};

pub mod archive;
pub mod paste;
pub mod user;
pub mod utils;

pub trait FromHtml {
    fn from_html(dom: &Html) -> Result<Self, String>
    where
        Self: Sized;
}

pub trait FromElement {
    fn from_element(parent: &ElementRef) -> Result<Self, String>
    where
        Self: Sized;
}

pub fn parse_date(date: &str) -> Result<i64, String> {
    let start_index = date
        .find(" of")
        .ok_or_else(|| format!("Date string missing ' of' marker: {}", date))?
        .checked_sub(2)
        .ok_or_else(|| format!("Invalid date format: {}", date))?;
    let end_index = start_index + 5;

    if end_index > date.len() {
        return Err(format!("Date string too short: {}", date));
    }

    let parsed_date =
        format!("{}{}", &date[..start_index], &date[end_index..]).replace("CDT", "-0500");

    let timestamp = DateTime::parse_from_str(&parsed_date, "%A %e %B %Y %r %z")
        .map_err(|e| format!("Failed to parse date '{}': {}", parsed_date, e))?
        .timestamp();

    Ok(timestamp)
}
