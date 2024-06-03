use scraper::{ElementRef, Html, Selector};
use serde::Serialize;

use crate::constants::URL;

use super::{FromElement, FromHtml};

#[derive(Serialize)]
pub struct Archive {
    id: String,
    title: String,
    age: String,
    syntax: String,
}

impl FromElement for Archive {
    fn from_element(parent: &ElementRef) -> Self {
        let id = parent
            .select(&Selector::parse(&"td:nth-child(1)>a").unwrap())
            .next()
            .unwrap()
            .attr("href")
            .unwrap()
            .replace("/", "")
            .to_owned();

        let title = parent
            .select(&Selector::parse(&"td:nth-child(1)>a").unwrap())
            .next()
            .unwrap()
            .text()
            .collect::<String>()
            .trim()
            .to_owned();

        let age = parent
            .select(&Selector::parse(&"td:nth-child(2)").unwrap())
            .next()
            .unwrap()
            .text()
            .collect::<String>()
            .trim()
            .to_owned();

        let syntax = parent
            .select(&Selector::parse(&"td:nth-child(3)>a").unwrap())
            .next()
            .unwrap()
            .attr("href")
            .unwrap()
            .replace("/archive/", "")
            .to_owned();

        Archive {
            id,
            title,
            age,
            syntax,
        }
    }
}

#[derive(Serialize)]
pub struct ArchivePage {
    syntax: Option<String>,
    archives: Vec<Archive>,
}

impl FromHtml for ArchivePage {
    fn from_html(dom: &Html) -> Self {
        let syntax = none_if_empty(
            dom.select(&Selector::parse(&"meta[property='og:url']").unwrap())
                .next()
                .unwrap()
                .attr("content")
                .unwrap()
                .replace(&format!("{URL}/archive"), "")
                .replace("/", "")
        );

        let parent = dom
            .select(&Selector::parse(&".archive-table").unwrap())
            .next()
            .unwrap();

        let archives = parent
            .select(&Selector::parse(&".maintable>tbody>tr").unwrap())
            .enumerate()
            .filter(|&(i, _)| i != 0)
            .map(|(_, v)| Archive::from_element(&v))
            .collect::<Vec<Archive>>();

        ArchivePage { syntax, archives }
    }
}

fn none_if_empty(string: String) -> Option<String> {
    if string.is_empty() {
        None
    } else {
        Some(string)
    }
}
