use scraper::{ElementRef, Html, Selector};
use serde::Serialize;
use once_cell::sync::Lazy;

use crate::constants::URL;

use super::{parse_date, FromElement, FromHtml};

// Helper function to safely parse dates with fallback to 0
fn safe_parse_date(date_str: &str) -> i64 {
    parse_date(date_str).unwrap_or_else(|e| {
        eprintln!("Failed to parse date '{}': {}", date_str, e);
        0 // Unix epoch as fallback
    })
}

// Pre-compiled selectors to avoid unwrap() calls
static SELECTOR_TD_CHILD_1_A: Lazy<Selector> = Lazy::new(|| {
    Selector::parse("td:nth-child(1)>a").expect("Valid CSS selector")
});
static SELECTOR_TD_CHILD_2: Lazy<Selector> = Lazy::new(|| {
    Selector::parse("td:nth-child(2)").expect("Valid CSS selector")
});
static SELECTOR_TD_CHILD_3: Lazy<Selector> = Lazy::new(|| {
    Selector::parse("td:nth-child(3)").expect("Valid CSS selector")
});
static SELECTOR_TD_CHILD_4: Lazy<Selector> = Lazy::new(|| {
    Selector::parse("td:nth-child(4)").expect("Valid CSS selector")
});
static SELECTOR_TD_CHILD_5: Lazy<Selector> = Lazy::new(|| {
    Selector::parse("td:nth-child(5)").expect("Valid CSS selector")
});
static SELECTOR_TD_CHILD_6_A: Lazy<Selector> = Lazy::new(|| {
    Selector::parse("td:nth-child(6)>a").expect("Valid CSS selector")
});

// Helper function to safely get text content from an element
fn safe_text_content(element: Option<ElementRef>) -> String {
    element
        .map(|e| e.text().collect::<String>().trim().to_owned())
        .unwrap_or_default()
}

// Helper function to safely get an attribute from an element
fn safe_attr_content(element: Option<ElementRef>, attr: &str) -> String {
    element
        .and_then(|el| el.value().attr(attr))
        .unwrap_or_default()
        .to_owned()
}

#[derive(Serialize)]
pub struct UserPaste {
    id: String,
    title: String,
    age: String,
    expires: String,
    views: u32,
    num_comments: u32,
    format: String,
}

impl FromElement for UserPaste {
    fn from_element(parent: &ElementRef) -> Self {
        let id_link = parent.select(&SELECTOR_TD_CHILD_1_A).next();
        let id = safe_attr_content(id_link, "href")
            .replace("/", "");

        let title = safe_text_content(id_link);

        let age = safe_text_content(parent.select(&SELECTOR_TD_CHILD_2).next());

        let expires = safe_text_content(parent.select(&SELECTOR_TD_CHILD_3).next());

        let views = parent
            .select(&SELECTOR_TD_CHILD_4)
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

        let num_comments = parent
            .select(&SELECTOR_TD_CHILD_5)
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

        let format = safe_attr_content(
            parent.select(&SELECTOR_TD_CHILD_6_A).next(),
            "href"
        ).replace("/archive/", "");

        UserPaste {
            id,
            title,
            age,
            expires,
            views,
            num_comments,
            format,
        }
    }
}

#[derive(Serialize)]
pub struct User {
    username: String,
    icon_url: String,
    website: Option<String>,
    location: Option<String>,
    profile_views: u32,
    paste_views: u32,
    rating: f32,
    date_joined: i64,
    pro: bool,
    pastes: Vec<UserPaste>,
}

impl FromHtml for User {
    fn from_html(dom: &Html) -> Self {
        let username = dom
            .select(&Selector::parse(&"meta[property='og:url']").unwrap())
            .next()
            .unwrap()
            .attr("content")
            .unwrap()
            .replace(&format!("{URL}/u/"), "");

        let parent = dom
            .select(&Selector::parse(&".user-view").unwrap())
            .next()
            .unwrap();

        let icon_url = parent
            .select(&Selector::parse(&".user-icon>img").unwrap())
            .next()
            .unwrap()
            .value()
            .attr("src")
            .unwrap()
            .replace("/themes/pastebin/img/", "/imgs/")
            .replace("/cache/img/", "/imgs/");

        let website = parent
            .select(&Selector::parse(&".web").unwrap())
            .next()
            .map(|e| e.value().attr("href").unwrap().to_owned());

        let location = parent
            .select(&Selector::parse(&".location").unwrap())
            .next()
            .map(|e| e.text().collect::<String>());

        let profile_views = parent
            .select(&Selector::parse(&".views:not(.-all)").unwrap())
            .next()
            .unwrap()
            .text()
            .collect::<String>()
            .replace(",", "")
            .parse()
            .unwrap();

        let paste_views = parent
            .select(&Selector::parse(&".views.-all").unwrap())
            .next()
            .unwrap()
            .text()
            .collect::<String>()
            .replace(",", "")
            .parse()
            .unwrap();

        let rating = parent
            .select(&Selector::parse(&".rating").unwrap())
            .next()
            .unwrap()
            .text()
            .collect::<String>()
            .parse()
            .unwrap();

        let date_joined = safe_parse_date(
            parent
                .select(&Selector::parse(&".date-text").unwrap())
                .next()
                .unwrap()
                .attr("title")
                .unwrap(),
        );

        let pro = parent
            .select(&Selector::parse(&".pro").unwrap())
            .next()
            .is_some();

        let pastes = dom
            .select(&Selector::parse(&".maintable>tbody>tr").unwrap())
            .enumerate()
            .filter(|&(i, _)| i != 0)
            .map(|(_, v)| UserPaste::from_element(&v))
            .collect::<Vec<UserPaste>>();

        User {
            username,
            icon_url,
            website,
            location,
            profile_views,
            paste_views,
            rating,
            date_joined,
            pro,
            pastes,
        }
    }
}

#[derive(Serialize)]
pub struct SimpleUser {
    username: String,
    registered: bool,
    pro: bool,
    icon_url: String,
}

impl FromElement for SimpleUser {
    fn from_element(parent: &ElementRef) -> Self {
        let username = parent
            .select(&Selector::parse(&".username").unwrap())
            .next()
            .unwrap()
            .text()
            .collect::<String>()
            .trim()
            .to_owned();

        let registered = parent
            .select(&Selector::parse(&".username>a").unwrap())
            .next()
            .is_some();

        let pro = parent
            .select(&Selector::parse(&".pro").unwrap())
            .next()
            .is_some();

        let icon_url = parent
            .select(&Selector::parse(&".user-icon>img").unwrap())
            .next()
            .unwrap()
            .value()
            .attr("src")
            .unwrap()
            .replace("/themes/pastebin/img/", "/imgs/")
            .replace("/cache/img/", "/imgs/")
            .to_owned();

        SimpleUser {
            username,
            registered,
            pro,
            icon_url,
        }
    }
}

#[cfg(test)]
mod tests {
    use scraper::{Html, Selector};

    use super::*;

    #[test]
    fn test_parse_simple_user() {
        let dom = Html::parse_document(
            r#"
            <div class="user">
                <div class="user-icon">
                    <img src="/themes/pastebin/img/user.png">
                </div>
                <div class="username">
                    <a href="/u/user">user</a>
                </div>
                <div class="pro"></div>
            </div>
        "#,
        );

        let user = SimpleUser::from_element(
            &dom.select(&Selector::parse(".user").unwrap())
                .next()
                .unwrap(),
        );

        assert_eq!(user.username, "user");
        assert_eq!(user.registered, true);
        assert_eq!(user.pro, true);
        assert_eq!(user.icon_url, "/imgs/user.png");
    }
}
