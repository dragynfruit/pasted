use byte_unit::Byte;
use chrono::DateTime;
use scraper::{selectable::Selectable, ElementRef, Html, Selector};
use serde::Serialize;

// pub struct User {
//     simple: SimpleUser,
//     website: Option<String>,
// }

#[derive(Serialize)]
pub struct SimpleUser {
    username: String,
    registered: bool,
    pro: bool,
    icon_url: String,
}

#[derive(Serialize)]
pub struct PasteContainer {
    category: Option<String>,
    size: u64,
    likes: Option<u32>,
    dislikes: Option<u32>,
    id: Option<String>,
    format: String,
    content: String,
}

#[derive(Serialize)]
pub struct Comment {
    author: SimpleUser,
    date: i64,
    edit_date: Option<i64>,
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

pub fn get_csrftoken(dom: &Html) -> String {
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

    let date = format!("{}{}", &date[..start_index], &date[end_index..]).replace("CDT", "-0500");

    DateTime::parse_from_str(&date, "%A %e %B %Y %r %z")
        .unwrap()
        .timestamp()
}

pub fn parse_simple_user(parent: &ElementRef) -> SimpleUser {
    SimpleUser {
        username: parent
            .select(&Selector::parse(&".username").unwrap())
            .next()
            .unwrap()
            .text()
            .collect::<String>()
            .trim()
            .to_owned(),
        registered: parent
            .select(&Selector::parse(&".username>a").unwrap())
            .next()
            .is_some(),
        pro: parent
            .select(&Selector::parse(&".pro").unwrap())
            .next()
            .is_some(),
        icon_url: parent
            .select(&Selector::parse(&".user-icon>img").unwrap())
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

pub fn parse_paste_container(parent: &ElementRef) -> PasteContainer {
    let category = parent
        .select(&Selector::parse(&"span[title=Category]").unwrap())
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
        parent
            .select(&Selector::parse(".left").unwrap())
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

    let likes = parent
        .select(&Selector::parse(&".-like").unwrap())
        .next()
        .map(|x| x.text().collect::<String>().trim().parse().unwrap());

    let dislikes = parent
        .select(&Selector::parse(&".-dislike").unwrap())
        .next()
        .map(|x| x.text().collect::<String>().trim().parse().unwrap());

    let id = parent
        .select(&Selector::parse(&"a[href^='/report/']").unwrap())
        .next()
        .map(|x| {
            x.value()
                .attr("href")
                .unwrap()
                .replace("/report/", "")
                .to_owned()
        });

    let format = parent
        .select(&Selector::parse(&"a.h_800[href^='/archive/']").unwrap())
        .next()
        .unwrap()
        .text()
        .collect::<String>()
        .trim()
        .to_owned();

    let content = parent
        .select(&Selector::parse(&".source>ol").unwrap())
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

pub fn parse_comment(parent: &ElementRef) -> Comment {
    let author = parse_simple_user(parent);

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

    let container = parse_paste_container(
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

pub fn parse_paste(dom: &Html) -> Paste {
    let parent = dom
        .select(&Selector::parse(".post-view").unwrap())
        .next()
        .unwrap();

    let title = parent
        .select(&Selector::parse(&".info-top>h1").unwrap())
        .next()
        .map(|x| x.text().collect::<String>());

    let tags = dom
        .select(&Selector::parse(&".tags>a").unwrap())
        .map(|el| el.text().collect::<String>().to_owned())
        .collect::<Vec<String>>();

    let container = parse_paste_container(
        &parent
            .select(&Selector::parse(".highlighted-code").unwrap())
            .next()
            .unwrap(),
    );

    let author = parse_simple_user(&parent);

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
        .map(|el| parse_comment(&el))
        .collect::<Vec<Comment>>();

    let locked = num_comments.is_none();

    Paste {
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
