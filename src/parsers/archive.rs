use scraper::{ElementRef, Html, Selector};
use serde::Serialize;

use crate::constants::URL;

use super::{FromElement, FromHtml};

#[derive(Serialize)]
pub struct Archive {
    id: String,
    title: String,
    age: String,
    format: String,
}

impl FromElement for Archive {
    fn from_element(parent: &ElementRef) -> Self {
        let id = parent
            .select(&Selector::parse(&"td:nth-child(1)>a").unwrap())
            .next()
            .and_then(|el| el.attr("href"))
            .map(|href| href.replace("/", ""))
            .unwrap_or_else(|| "unknown".to_string());

        let title = parent
            .select(&Selector::parse(&"td:nth-child(1)>a").unwrap())
            .next()
            .map(|el| el.text().collect::<String>().trim().to_owned())
            .unwrap_or_else(|| "Untitled".to_string());

        let age = parent
            .select(&Selector::parse(&"td:nth-child(2)").unwrap())
            .next()
            .map(|el| el.text().collect::<String>().trim().to_owned())
            .unwrap_or_else(|| "Unknown".to_string());

        let format = parent
            .select(&Selector::parse(&"td:nth-child(3)>a").unwrap())
            .next()
            .and_then(|el| el.attr("href"))
            .map(|href| href.replace("/archive/", ""))
            .unwrap_or_else(|| "text".to_string());

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
        let format = dom
            .select(&Selector::parse(&"meta[property='og:url']").unwrap())
            .next()
            .and_then(|el| el.attr("content"))
            .map(|content| {
                none_if_empty(
                    content
                        .replace(&format!("{URL}/archive"), "")
                        .replace("/", "")
                )
            })
            .flatten();

        let parent = dom
            .select(&Selector::parse(&".archive-table").unwrap())
            .next();

        let archives = if let Some(parent) = parent {
            parent
                .select(&Selector::parse(&".maintable>tbody>tr").unwrap())
                .enumerate()
                .filter(|&(i, _)| i != 0)
                .map(|(_, v)| Archive::from_element(&v))
                .collect::<Vec<Archive>>()
        } else {
            eprintln!("Warning: Archive .archive-table element not found");
            Vec::new()
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
