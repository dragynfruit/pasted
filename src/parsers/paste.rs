use byte_unit::Byte;
use scraper::{ElementRef, Html, Selector, selectable::Selectable};
use serde::Serialize;

use crate::constants::URL;

use super::{FromElement, FromHtml, parse_date, user::SimpleUser};

// Helper function to safely parse dates with fallback to 0
fn safe_parse_date(date_str: &str) -> i64 {
    parse_date(date_str).unwrap_or_else(|e| {
        eprintln!("Failed to parse date '{}': {}", date_str, e);
        0 // Unix epoch as fallback
    })
}

#[derive(Serialize)]
pub struct PasteContainer {
    category: Option<String>,
    size: u64,
    likes: Option<u32>,
    dislikes: Option<u32>,
    id: Option<String>,
    format: String,
    format_name: String,
    content: String,
}

impl FromElement for PasteContainer {
    fn from_element(parent: &ElementRef) -> Self {
        let category = parent
            .select(&Selector::parse("span[title=Category]").unwrap())
            .next()
            .and_then(|x| {
                let text = x.text().collect::<String>();
                text.trim().split_once(" ").map(|(_, cat)| cat.to_owned())
            });

        let size = parent
            .select(&Selector::parse(".left").unwrap())
            .next()
            .and_then(|el| {
                let text = el.text().collect::<String>();
                text.trim()
                    .split_once(" ")
                    .and_then(|(_, size)| size.split_once("\n"))
                    .map(|(size, _)| size.to_owned())
            })
            .and_then(|size| Byte::parse_str(&size, true).ok())
            .unwrap_or(Byte::default())
            .as_u64();

        let likes = parent
            .select(&Selector::parse(".-like").unwrap())
            .next()
            .and_then(|x| x.text().collect::<String>().trim().parse().ok());

        let dislikes = parent
            .select(&Selector::parse(".-dislike").unwrap())
            .next()
            .and_then(|x| x.text().collect::<String>().trim().parse().ok());

        let id = parent
            .select(&Selector::parse("a[href^='/report/']").unwrap())
            .next()
            .and_then(|x| x.value().attr("href"))
            .map(|href| href.replace("/report/", ""));

        let format_el = parent
            .select(&Selector::parse("a.h_800[href^='/archive/']").unwrap())
            .next();

        let format = format_el
            .and_then(|el| el.attr("href"))
            .map(|href| href.replace("/archive/", ""))
            .unwrap_or_else(|| "text".to_string());

        let format_name = format_el
            .map(|el| el.text().collect::<String>().trim().to_owned())
            .unwrap_or_else(|| "Plain Text".to_string());

        let content = parent
            .select(&Selector::parse(".source>ol").unwrap())
            .next()
            .map(|el| el.text().collect::<String>())
            .unwrap_or_default();

        PasteContainer {
            category,
            size,
            likes,
            dislikes,
            id,
            format,
            format_name,
            content,
        }
    }
}

#[derive(Serialize)]
pub struct Comment {
    author: SimpleUser,
    date: i64,
    edit_date: Option<i64>,
    container: PasteContainer,
    num_comments: u32,
}

impl FromElement for Comment {
    fn from_element(parent: &ElementRef) -> Self {
        let author = SimpleUser::from_element(&parent);

        let date = parent
            .select(&Selector::parse(&".date>span").unwrap())
            .next()
            .and_then(|el| el.attr("title"))
            .map(|date_str| safe_parse_date(date_str))
            .unwrap_or_else(|| {
                eprintln!("Warning: Comment date span element not found");
                0
            });

        let edit_date = parent
            .select(&Selector::parse(&".date>span:nth-child(2)").unwrap())
            .next()
            .map(|x| {
                if let Some(title) = x.attr("title") {
                    if let Some((_, date_part)) = title.split_once(":") {
                        Some(safe_parse_date(date_part.trim()))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .flatten();

        let container = parent
            .select(&Selector::parse(".highlighted-code").unwrap())
            .next()
            .map(|el| PasteContainer::from_element(&el))
            .unwrap_or_else(|| {
                eprintln!("Warning: Comment .highlighted-code element not found");
                PasteContainer {
                    category: None,
                    size: 0,
                    likes: None,
                    dislikes: None,
                    id: None,
                    format: "text".to_string(),
                    format_name: "Plain Text".to_string(),
                    content: String::new(),
                }
            });

        let num_comments = parent
            .select(&Selector::parse(&"a[href='#comments']").unwrap())
            .next()
            .map(|x| x.text().collect::<String>().trim().parse().unwrap_or(0))
            .unwrap_or(0);

        Comment {
            author,
            date,
            edit_date,
            container,
            num_comments,
        }
    }
}

#[derive(Serialize)]
pub struct Paste {
    id: String,
    title: Option<String>,
    tags: Vec<String>,
    container: PasteContainer,
    author: SimpleUser,
    date: i64,
    edit_date: Option<i64>,
    views: u32,
    rating: f32,
    expire: String,
    comment_for: Option<String>,
    unlisted: bool,
    num_comments: Option<u32>,
    comments: Vec<Comment>,
    locked: bool,
}

impl FromHtml for Paste {
    fn from_html(dom: &Html) -> Self {
        let id = dom
            .select(&Selector::parse(&"meta[property='og:url']").unwrap())
            .next()
            .and_then(|el| el.attr("content"))
            .map(|content| content.replace(&format!("{URL}/"), ""))
            .unwrap_or_else(|| {
                eprintln!("Warning: og:url meta tag not found");
                "unknown".to_string()
            });

        let parent = dom
            .select(&Selector::parse(".post-view").unwrap())
            .next();

        let Some(parent) = parent else {
            eprintln!("Warning: Paste .post-view element not found in HTML");
            // Return a minimal Paste struct with default values
            // Create a minimal HTML fragment to use for SimpleUser parsing
            let minimal_html = Html::parse_fragment("<div></div>");
            let minimal_element = minimal_html.root_element();
            
            return Paste {
                id,
                title: None,
                tags: Vec::new(),
                container: PasteContainer {
                    category: None,
                    size: 0,
                    likes: None,
                    dislikes: None,
                    id: None,
                    format: "text".to_string(),
                    format_name: "Plain Text".to_string(),
                    content: String::new(),
                },
                author: SimpleUser::from_element(&minimal_element),
                date: 0,
                edit_date: None,
                views: 0,
                rating: 0.0,
                expire: "Unknown".to_string(),
                comment_for: None,
                unlisted: false,
                num_comments: None,
                comments: Vec::new(),
                locked: false,
            };
        };

        let title = parent
            .select(&Selector::parse(&".info-top>h1").unwrap())
            .next()
            .map(|x| x.text().collect::<String>());

        let tags = parent
            .select(&Selector::parse(&".tags>a").unwrap())
            .map(|el| el.text().collect::<String>().to_owned())
            .collect::<Vec<String>>();

        let container = parent
            .select(&Selector::parse(".highlighted-code").unwrap())
            .next()
            .map(|el| PasteContainer::from_element(&el))
            .unwrap_or_else(|| {
                eprintln!("Warning: Paste .highlighted-code element not found");
                PasteContainer {
                    category: None,
                    size: 0,
                    likes: None,
                    dislikes: None,
                    id: None,
                    format: "text".to_string(),
                    format_name: "Plain Text".to_string(),
                    content: String::new(),
                }
            });

        let author = SimpleUser::from_element(&parent);

        let date = parent
            .select(&Selector::parse(&".date>span").unwrap())
            .next()
            .and_then(|el| el.attr("title"))
            .map(|date_str| safe_parse_date(date_str))
            .unwrap_or_else(|| {
                eprintln!("Warning: Paste date span element not found");
                0
            });

        let edit_date = parent
            .select(&Selector::parse(&".date>span:nth-child(2)").unwrap())
            .next()
            .map(|x| {
                if let Some(title) = x.attr("title") {
                    if let Some((_, date_part)) = title.split_once(":") {
                        Some(safe_parse_date(date_part.trim()))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .flatten();

        let views = parent
            .select(&Selector::parse(&".visits").unwrap())
            .next()
            .and_then(|el| {
                el.text()
                    .collect::<String>()
                    .trim()
                    .replace(",", "")
                    .parse()
                    .ok()
            })
            .unwrap_or(0);

        let rating = parent
            .select(&Selector::parse(&".rating").unwrap())
            .next()
            .and_then(|el| {
                el.text()
                    .collect::<String>()
                    .trim()
                    .parse()
                    .ok()
            })
            .unwrap_or(0.0);

        let expire = parent
            .select(&Selector::parse(&".expire").unwrap())
            .next()
            .map(|el| el.text().collect::<String>().trim().to_owned())
            .unwrap_or_else(|| "Unknown".to_string());

        let comment_for = parent
            .select(&Selector::parse(&".notice").unwrap())
            .find_map(|el| {
                if el
                    .select(&Selector::parse(&"*:first-child").unwrap())
                    .next()?
                    .text()
                    .collect::<String>()
                    .trim()
                    == "This is comment for paste"
                {
                    el.select(&Selector::parse("a").unwrap())
                        .next()
                        .and_then(|a| a.attr("href"))
                        .and_then(|href| {
                            href.replace("/", "")
                                .split_once("#")
                                .map(|(id, _)| id.to_owned())
                        })
                } else {
                    None
                }
            });

        let unlisted = parent
            .select(&Selector::parse(&".unlisted").unwrap())
            .next()
            .is_some();

        let num_comments = parent
            .select(&Selector::parse(&"div[title=Comments]>a").unwrap())
            .next()
            .map(|x| x.text().collect::<String>().trim().parse().unwrap_or(0));

        let comments = parent
            .select(&Selector::parse(&".comments__list>ul>li").unwrap())
            .map(|el| Comment::from_element(&el))
            .collect::<Vec<Comment>>();

        let locked = num_comments.is_none();

        Paste {
            id,
            title,
            tags,
            container,
            author,
            date,
            edit_date,
            views,
            rating,
            expire,
            comment_for,
            unlisted,
            num_comments,
            comments,
            locked,
        }
    }
}

pub fn get_csrftoken(dom: &Html) -> String {
    dom.select(&Selector::parse("meta[name=csrf-token]").unwrap())
        .next()
        .unwrap()
        .value()
        .attr("content")
        .unwrap()
        .to_owned()
}

pub fn is_locked(dom: &Html) -> bool {
    dom.select(&Selector::parse("#postpasswordverificationform-password").unwrap())
        .next()
        .is_some()
}

pub fn is_burn(dom: &Html) -> bool {
    dom.select(&Selector::parse(".burn, .-burn").unwrap())
        .next()
        .is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_csrftoken() {
        let dom = Html::parse_document(
            r#"
            <html>
                <head>
                    <meta name="csrf-token" content="token">
                </head>
            </html>
        "#,
        );

        assert_eq!(get_csrftoken(&dom), "token");
    }

    #[test]
    fn test_parse_date() {
        let result = parse_date("Thursday 2nd of May 2024 10:05:29 AM CDT");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1714662329);
    }

    #[test]
    fn test_is_locked() {
        let dom = Html::parse_document(
            r#"
            <form id="postpasswordverificationform">
                <input id="postpasswordverificationform-password">
            </form>
        "#,
        );

        assert_eq!(is_locked(&dom), true);
    }

    #[test]
    fn test_is_burn() {
        let dom = Html::parse_document(
            r#"
            <div class="burn"></div>
        "#,
        );

        assert_eq!(is_burn(&dom), true);
    }
}
