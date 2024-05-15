use byte_unit::Byte;
use chrono::DateTime;
use scraper::{Html, Selector};
use serde::Serialize;

pub struct User {
    simple: SimpleUser,
    website: Option<String>,
}

#[derive(Serialize)]
pub struct SimpleUser {
    username: String,
    registered: bool,
    icon_url: String,
}

#[derive(Serialize)]
pub struct PasteContainer {
    category: Option<String>,
    size: u64,
    likes: u32,
    dislikes: u32,
    id: String,
    format: String,
    content: String,
}

#[derive(Serialize)]
pub struct Comment {
    author: SimpleUser,
    date: i64,
    container: PasteContainer,
    num_comments: u32,
}

#[derive(Serialize)]
pub struct Paste {
    title: Option<String>,
    tags: Vec<String>,
    container: PasteContainer,
    author: SimpleUser,
    date: i64,
    views: u32,
    rating: f32,
    expire: String,
    is_comment: bool,
    is_unlisted: bool,
    num_comments: u32,
    comments: Vec<Comment>,
}

pub fn get_csrftoken(dom: Html) -> String {
    dom.select(&Selector::parse("meta[name=csrf-token]").unwrap())
        .next()
        .unwrap()
        .value()
        .attr("content")
        .unwrap()
        .to_owned()
}

pub fn parse_date(date: &str) -> i64 {
    let start_index = date.find(" of").unwrap() - 2;
    let end_index = start_index + 5;

    let date =
        format!("{}{}", &date[..start_index], &date[end_index..]).replace("CDT", "-0500");

    DateTime::parse_from_str(&date, "%A %e %B %Y %r %z")
        .unwrap()
        .timestamp()
}

pub fn parse_simple_user(parent_selector: &str, dom: &Html) -> SimpleUser {
    SimpleUser {
        username: dom
            .select(&Selector::parse(&format!("{parent_selector} .username")).unwrap())
            .next()
            .unwrap()
            .text()
            .collect::<String>()
            .trim()
            .to_owned(),
        registered: dom
            .select(&Selector::parse(&format!("{parent_selector} .username > a")).unwrap())
            .next()
            .is_some(),
        icon_url: dom
            .select(&Selector::parse(&format!("{parent_selector} .user-icon > img")).unwrap())
            .next()
            .unwrap()
            .value()
            .attr("src")
            .unwrap()
            .replace("/themes/pastebin/img/", "/imgs/")
            .replace("/cache/img/", "/imgs/")
            .to_owned(),
    }
}

pub fn parse_paste_container(parent_selector: &str, dom: &Html) -> PasteContainer {
    let category = dom
        .select(&Selector::parse(&format!("{parent_selector} span[title=Category]")).unwrap())
        .next()
        .map(|x| {
            x.text()
                .collect::<String>()
                .trim()
                .split_once(" ")
                .unwrap()
                .1
                .to_owned()
        });

    let size = Byte::parse_str(
        dom.select(&Selector::parse(&format!("{parent_selector} .left")).unwrap())
            .next()
            .unwrap()
            .text()
            .collect::<String>()
            .trim()
            .split_once(" ")
            .unwrap()
            .1
            .split_once("\n")
            .unwrap()
            .0
            .to_owned(),
        true,
    )
    .unwrap_or(Byte::default())
    .as_u64();

    let likes = dom
        .select(&Selector::parse(&format!("{parent_selector} a[title=Like]")).unwrap())
        .next()
        .unwrap()
        .text()
        .collect::<String>()
        .trim()
        .parse()
        .unwrap();

    let dislikes = dom
        .select(&Selector::parse(&format!("{parent_selector} a[title=Dislike]")).unwrap())
        .next()
        .unwrap()
        .text()
        .collect::<String>()
        .trim()
        .parse()
        .unwrap();

    let id = dom
        .select(&Selector::parse(&format!("{parent_selector} a[href^='/report/']")).unwrap())
        .next()
        .unwrap()
        .value()
        .attr("href")
        .unwrap()
        .replace("/report/", "")
        .to_owned();

    let format = dom
        .select(&Selector::parse(&format!("{parent_selector} a.h_800[href^='/archive/']")).unwrap())
        .next()
        .unwrap()
        .text()
        .collect::<String>()
        .trim()
        .to_owned();

    let content = dom
        .select(&Selector::parse(&format!("{parent_selector} .source > ol")).unwrap())
        .next()
        .unwrap()
        .text()
        .collect::<String>()
        .to_owned();

    PasteContainer {
        category,
        size,
        likes,
        dislikes,
        id,
        format,
        content,
    }
}

pub fn parse_comment(parent_selector: &str, dom: &Html) -> Comment {
    let author = parse_simple_user(&format!("{parent_selector}"), dom);

    let date = parse_date(
        &dom.select(&Selector::parse(&format!("{parent_selector} .date > span")).unwrap())
            .next()
            .unwrap()
            .attr("title")
            .unwrap(),
    );

    let container = parse_paste_container(&format!("{parent_selector} .highlighted-code"), dom);

    let num_comments = dom
        .select(&Selector::parse(&format!("{parent_selector} a[href='#comments']")).unwrap())
        .next()
        .map(|x| x.text().collect::<String>().trim().parse().unwrap_or(0))
        .unwrap_or(0);

    Comment {
        author,
        date,
        container,
        num_comments,
    }
}

pub fn parse_paste(dom: &Html) -> Paste {
    let parent_selector = ".post-view";

    let title = dom
        .select(&Selector::parse(&format!("{parent_selector} .info-top > h1")).unwrap())
        .next()
        .map(|x| x.text().collect::<String>());

    let tags = dom
        .select(&Selector::parse(&format!("{parent_selector} .tags > a")).unwrap())
        .map(|el| el.text().collect::<String>().to_owned())
        .collect::<Vec<String>>();

    let container = parse_paste_container(&format!("{parent_selector} .highlighted-code"), dom);

    let author = parse_simple_user(&format!("{parent_selector}"), dom);

    let date = parse_date(
        &dom.select(&Selector::parse(&format!("{parent_selector} .date > span")).unwrap())
            .next()
            .unwrap()
            .attr("title")
            .unwrap(),
    );

    let views = dom
        .select(&Selector::parse(&format!("{parent_selector} .visits")).unwrap())
        .next()
        .unwrap()
        .text()
        .collect::<String>()
        .trim()
        .parse()
        .unwrap();

    let rating = dom
        .select(&Selector::parse(&format!("{parent_selector} .rating")).unwrap())
        .next()
        .unwrap()
        .text()
        .collect::<String>()
        .trim()
        .parse()
        .unwrap();

    let expire = dom
        .select(&Selector::parse(&format!("{parent_selector} .expire")).unwrap())
        .next()
        .unwrap()
        .text()
        .collect::<String>()
        .trim()
        .to_owned();

    let is_comment = title.is_none();

    let is_unlisted = dom
        .select(&Selector::parse(&format!("{parent_selector} .unlisted")).unwrap())
        .next()
        .is_some();

    let num_comments = dom
        .select(&Selector::parse(&format!("{parent_selector} div[title=Comments] > a")).unwrap())
        .next()
        .unwrap()
        .text()
        .collect::<String>()
        .trim()
        .parse()
        .unwrap_or(0);

    let comments = dom
        .select(&Selector::parse(&format!("{parent_selector} .comments__list > ul > li")).unwrap())
        .enumerate()
        .map(|(i, _)| {
            parse_comment(
                &format!("{parent_selector} .comments__list > ul > li:nth-child({})", i + 1),
                dom,
            )
        })
        .collect::<Vec<Comment>>();

    Paste {
        title,
        tags,
        container,
        author,
        date,
        views,
        rating,
        expire,
        is_comment,
        is_unlisted,
        num_comments,
        comments,
    }
}
