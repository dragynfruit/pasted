use byte_unit::Byte;
use once_cell::sync::Lazy;
use scraper::{ElementRef, Html, Selector, selectable::Selectable};
use serde::Serialize;

use crate::constants::URL;

use super::{FromElement, FromHtml, parse_date, user::SimpleUser};
use super::utils::{safe_text_content, safe_attr_content, safe_select, safe_parse_number};

// Helper function to safely parse dates with fallback to 0
fn safe_parse_date(date_str: &str) -> i64 {
    parse_date(date_str).unwrap_or_else(|e| {
        eprintln!("Failed to parse date '{}': {}", date_str, e);
        0 // Unix epoch as fallback
    })
}

// Pre-compiled selectors
static SELECTOR_CATEGORY: Lazy<Selector> =
    Lazy::new(|| Selector::parse("span[title=Category]").expect("Valid CSS selector"));
static SELECTOR_LEFT: Lazy<Selector> =
    Lazy::new(|| Selector::parse(".left").expect("Valid CSS selector"));
static SELECTOR_LIKE: Lazy<Selector> =
    Lazy::new(|| Selector::parse(".-like").expect("Valid CSS selector"));
static SELECTOR_DISLIKE: Lazy<Selector> =
    Lazy::new(|| Selector::parse(".-dislike").expect("Valid CSS selector"));
static SELECTOR_REPORT: Lazy<Selector> =
    Lazy::new(|| Selector::parse("a[href^='/report/']").expect("Valid CSS selector"));
static SELECTOR_ARCHIVE_LINK: Lazy<Selector> =
    Lazy::new(|| Selector::parse("a.h_800[href^='/archive/']").expect("Valid CSS selector"));
static SELECTOR_SOURCE_OL: Lazy<Selector> =
    Lazy::new(|| Selector::parse(".source>ol").expect("Valid CSS selector"));
static SELECTOR_USERNAME_USER: Lazy<Selector> =
    Lazy::new(|| Selector::parse(".username>.user").expect("Valid CSS selector"));
static SELECTOR_DATE_SPAN: Lazy<Selector> =
    Lazy::new(|| Selector::parse(".date>span").expect("Valid CSS selector"));
static SELECTOR_DATE_SPAN_2: Lazy<Selector> =
    Lazy::new(|| Selector::parse(".date>span:nth-child(2)").expect("Valid CSS selector"));
static SELECTOR_HIGHLIGHTED_CODE: Lazy<Selector> =
    Lazy::new(|| Selector::parse(".highlighted-code").expect("Valid CSS selector"));
static SELECTOR_COMMENTS_LINK: Lazy<Selector> =
    Lazy::new(|| Selector::parse("a[href='#comments']").expect("Valid CSS selector"));
static SELECTOR_META_OG_URL: Lazy<Selector> =
    Lazy::new(|| Selector::parse("meta[property='og:url']").expect("Valid CSS selector"));
static SELECTOR_POST_VIEW: Lazy<Selector> =
    Lazy::new(|| Selector::parse(".post-view").expect("Valid CSS selector"));
static SELECTOR_INFO_TOP_H1: Lazy<Selector> =
    Lazy::new(|| Selector::parse(".info-top>h1").expect("Valid CSS selector"));
static SELECTOR_TAGS_A: Lazy<Selector> =
    Lazy::new(|| Selector::parse(".tags>a").expect("Valid CSS selector"));
static SELECTOR_VISITS: Lazy<Selector> =
    Lazy::new(|| Selector::parse(".visits").expect("Valid CSS selector"));
static SELECTOR_RATING: Lazy<Selector> =
    Lazy::new(|| Selector::parse(".rating").expect("Valid CSS selector"));
static SELECTOR_EXPIRE: Lazy<Selector> =
    Lazy::new(|| Selector::parse(".expire").expect("Valid CSS selector"));
static SELECTOR_NOTICE: Lazy<Selector> =
    Lazy::new(|| Selector::parse(".notice").expect("Valid CSS selector"));
static SELECTOR_FIRST_CHILD: Lazy<Selector> =
    Lazy::new(|| Selector::parse("*:first-child").expect("Valid CSS selector"));
static SELECTOR_A: Lazy<Selector> =
    Lazy::new(|| Selector::parse("a").expect("Valid CSS selector"));
static SELECTOR_UNLISTED: Lazy<Selector> =
    Lazy::new(|| Selector::parse(".unlisted").expect("Valid CSS selector"));
static SELECTOR_COMMENTS_TITLE: Lazy<Selector> =
    Lazy::new(|| Selector::parse("div[title=Comments]>a").expect("Valid CSS selector"));
static SELECTOR_COMMENTS_LIST: Lazy<Selector> =
    Lazy::new(|| Selector::parse(".comments__list>ul>li").expect("Valid CSS selector"));
static SELECTOR_CSRF_TOKEN: Lazy<Selector> =
    Lazy::new(|| Selector::parse("meta[name=csrf-token]").expect("Valid CSS selector"));
static SELECTOR_PASSWORD_FORM: Lazy<Selector> =
    Lazy::new(|| Selector::parse("#postpasswordverificationform-password").expect("Valid CSS selector"));
static SELECTOR_BURN: Lazy<Selector> =
    Lazy::new(|| Selector::parse(".burn, .-burn").expect("Valid CSS selector"));

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
    fn from_element(parent: &ElementRef) -> Result<Self, String> {
        let category = safe_select(parent, &SELECTOR_CATEGORY)
            .and_then(|x| {
                let text = x.text().collect::<String>();
                text.trim().split_once(" ").map(|(_, cat)| cat.to_owned())
            });

        let size = safe_select(parent, &SELECTOR_LEFT)
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

        let likes = safe_select(parent, &SELECTOR_LIKE)
            .and_then(|x| x.text().collect::<String>().trim().parse().ok());

        let dislikes = safe_select(parent, &SELECTOR_DISLIKE)
            .and_then(|x| x.text().collect::<String>().trim().parse().ok());

        let id = safe_select(parent, &SELECTOR_REPORT)
            .and_then(|x| x.value().attr("href"))
            .map(|href| href.replace("/report/", ""));

        let format_el = safe_select(parent, &SELECTOR_ARCHIVE_LINK);

        let format = format_el
            .and_then(|el| el.attr("href"))
            .map(|href| href.replace("/archive/", ""))
            .unwrap_or_else(|| "text".to_string());

        let format_name = format_el
            .map(|el| el.text().collect::<String>().trim().to_owned())
            .unwrap_or_else(|| "Plain Text".to_string());

        let content = safe_select(parent, &SELECTOR_SOURCE_OL)
            .map(|el| el.text().collect::<String>())
            .unwrap_or_default();

        Ok(PasteContainer {
            category,
            size,
            likes,
            dislikes,
            id,
            format,
            format_name,
            content,
        })
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
    fn from_element(parent: &ElementRef) -> Result<Self, String> {
        let author = SimpleUser::from_element(parent)?;

        let date = safe_select(parent, &SELECTOR_DATE_SPAN)
            .and_then(|el| el.attr("title"))
            .map(|title| safe_parse_date(title))
            .unwrap_or(0);

        let edit_date = safe_select(parent, &SELECTOR_DATE_SPAN_2)
            .and_then(|x| {
                x.attr("title").and_then(|title| {
                    title.split_once(":").map(|(_, date_part)| {
                        safe_parse_date(date_part.trim())
                    })
                })
            });

        let container = safe_select(parent, &SELECTOR_HIGHLIGHTED_CODE)
            .ok_or("Missing highlighted code section")?;
        let container = PasteContainer::from_element(&container)?;

        let num_comments = safe_select(parent, &SELECTOR_COMMENTS_LINK)
            .map(|x| safe_parse_number::<u32>(&x.text().collect::<String>()))
            .unwrap_or(0);

        Ok(Comment {
            author,
            date,
            edit_date,
            container,
            num_comments,
        })
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
    fn from_html(dom: &Html) -> Result<Self, String> {
        let meta_element = dom.select(&SELECTOR_META_OG_URL).next();
        let id = safe_attr_content(meta_element, "content")
            .replace(&format!("{URL}/"), "");

        let parent = dom.select(&SELECTOR_POST_VIEW).next()
            .ok_or("Missing .post-view element")?;

        let title = safe_select(&parent, &SELECTOR_INFO_TOP_H1)
            .map(|x| x.text().collect::<String>());

        let tags = parent
            .select(&SELECTOR_TAGS_A)
            .map(|el| el.text().collect::<String>())
            .collect::<Vec<String>>();

        let container_elem = safe_select(&parent, &SELECTOR_HIGHLIGHTED_CODE)
            .ok_or("Missing highlighted code section")?;
        let container = PasteContainer::from_element(&container_elem)?;

        let author = SimpleUser::from_element(&parent)?;

        let date = safe_select(&parent, &SELECTOR_DATE_SPAN)
            .and_then(|el| el.attr("title"))
            .map(|title| safe_parse_date(title))
            .unwrap_or(0);

        let edit_date = safe_select(&parent, &SELECTOR_DATE_SPAN_2)
            .and_then(|x| {
                x.attr("title").and_then(|title| {
                    title.split_once(":").map(|(_, date_part)| {
                        safe_parse_date(date_part.trim())
                    })
                })
            });

        let views: u32 = safe_select(&parent, &SELECTOR_VISITS)
            .map(|el| safe_parse_number(&el.text().collect::<String>()))
            .unwrap_or(0);

        let rating: f32 = safe_select(&parent, &SELECTOR_RATING)
            .and_then(|el| el.text().collect::<String>().trim().parse().ok())
            .unwrap_or(0.0);

        let expire = safe_select(&parent, &SELECTOR_EXPIRE)
            .map(|el| el.text().collect::<String>().trim().to_owned())
            .unwrap_or_default();

        let comment_for = parent
            .select(&SELECTOR_NOTICE)
            .find_map(|el| {
                let first_child = safe_select(&el, &SELECTOR_FIRST_CHILD)?;
                if first_child.text().collect::<String>().trim() == "This is comment for paste" {
                    let link = safe_select(&el, &SELECTOR_A)?;
                    let href = link.attr("href")?;
                    href.replace("/", "")
                        .split_once("#")
                        .map(|(id, _)| id.to_owned())
                } else {
                    None
                }
            });

        let unlisted = safe_select(&parent, &SELECTOR_UNLISTED).is_some();

        let num_comments = safe_select(&parent, &SELECTOR_COMMENTS_TITLE)
            .map(|x| safe_parse_number::<u32>(&x.text().collect::<String>()));

        let comments = parent
            .select(&SELECTOR_COMMENTS_LIST)
            .map(|el| Comment::from_element(&el))
            .collect::<Result<Vec<Comment>, String>>()?;

        let locked = num_comments.is_none();

        Ok(Paste {
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
        })
    }
}

pub fn get_csrftoken(dom: &Html) -> Option<String> {
    dom.select(&SELECTOR_CSRF_TOKEN)
        .next()
        .and_then(|el| el.value().attr("content"))
        .map(|s| s.to_owned())
}

pub fn is_locked(dom: &Html) -> bool {
    dom.select(&SELECTOR_PASSWORD_FORM).next().is_some()
}

pub fn is_burn(dom: &Html) -> bool {
    dom.select(&SELECTOR_BURN).next().is_some()
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

        assert_eq!(get_csrftoken(&dom), Some("token".to_string()));
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
