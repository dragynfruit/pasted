use byte_unit::Byte;
use scraper::{selectable::Selectable, ElementRef, Html, Selector};
use serde::Serialize;

use crate::constants::URL;

use super::{parse_date, user::SimpleUser, FromElement, FromHtml};

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

        let date = parse_date(
            &parent
                .select(&Selector::parse(&".date>span").unwrap())
                .next()
                .unwrap()
                .attr("title")
                .unwrap(),
        );

        let edit_date = parent
            .select(&Selector::parse(&".date>span:nth-child(2)").unwrap())
            .next()
            .map(|x| parse_date(x.attr("title").unwrap().split_once(":").unwrap().1.trim()));

        let container = PasteContainer::from_element(
            &parent
                .select(&Selector::parse(".highlighted-code").unwrap())
                .next()
                .unwrap(),
        );

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
            .unwrap()
            .attr("content")
            .unwrap()
            .replace(&format!("{URL}/"), "")
            .to_owned();

        let parent = dom
            .select(&Selector::parse(".post-view").unwrap())
            .next()
            .unwrap();

        let title = parent
            .select(&Selector::parse(&".info-top>h1").unwrap())
            .next()
            .map(|x| x.text().collect::<String>());

        let tags = parent
            .select(&Selector::parse(&".tags>a").unwrap())
            .map(|el| el.text().collect::<String>().to_owned())
            .collect::<Vec<String>>();

        let container = PasteContainer::from_element(
            &parent
                .select(&Selector::parse(".highlighted-code").unwrap())
                .next()
                .unwrap(),
        );

        let author = SimpleUser::from_element(&parent);

        let date = parse_date(
            &parent
                .select(&Selector::parse(&".date>span").unwrap())
                .next()
                .unwrap()
                .attr("title")
                .unwrap(),
        );

        let edit_date = parent
            .select(&Selector::parse(&".date>span:nth-child(2)").unwrap())
            .next()
            .map(|x| parse_date(x.attr("title").unwrap().split_once(":").unwrap().1.trim()));

        let views = parent
            .select(&Selector::parse(&".visits").unwrap())
            .next()
            .unwrap()
            .text()
            .collect::<String>()
            .trim()
            .replace(",", "")
            .parse()
            .unwrap();

        let rating = parent
            .select(&Selector::parse(&".rating").unwrap())
            .next()
            .unwrap()
            .text()
            .collect::<String>()
            .trim()
            .parse()
            .unwrap();

        let expire = parent
            .select(&Selector::parse(&".expire").unwrap())
            .next()
            .unwrap()
            .text()
            .collect::<String>()
            .trim()
            .to_owned();

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
                    Some(
                        el.select(&Selector::parse("a").unwrap())
                            .next()
                            .unwrap()
                            .attr("href")
                            .unwrap()
                            .replace("/", "")
                            .split_once("#")
                            .unwrap()
                            .0
                            .to_owned(),
                    )
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
        assert_eq!(
            parse_date("Thursday 2nd of May 2024 10:05:29 AM CDT"),
            1714662329
        );
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
