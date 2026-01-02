use once_cell::sync::Lazy;
use scraper::{ElementRef, Html, Selector};
use serde::Serialize;

use crate::constants::URL;

use super::{FromElement, FromHtml, parse_date};

// Helper function to safely parse dates with fallback to 0
fn safe_parse_date(date_str: &str) -> i64 {
    parse_date(date_str).unwrap_or_else(|e| {
        eprintln!("Failed to parse date '{}': {}", date_str, e);
        0 // Unix epoch as fallback
    })
}

// Pre-compiled selectors to avoid unwrap() calls
static SELECTOR_META_OG_URL: Lazy<Selector> =
    Lazy::new(|| Selector::parse("meta[property='og:url']").expect("Valid CSS selector"));
static SELECTOR_USER_VIEW: Lazy<Selector> =
    Lazy::new(|| Selector::parse(".user-view").expect("Valid CSS selector"));
static SELECTOR_USER_ICON_IMG: Lazy<Selector> =
    Lazy::new(|| Selector::parse(".user-icon>img").expect("Valid CSS selector"));
static SELECTOR_WEB: Lazy<Selector> =
    Lazy::new(|| Selector::parse(".web").expect("Valid CSS selector"));
static SELECTOR_LOCATION: Lazy<Selector> =
    Lazy::new(|| Selector::parse(".location").expect("Valid CSS selector"));
static SELECTOR_VIEWS_NOT_ALL: Lazy<Selector> =
    Lazy::new(|| Selector::parse(".views:not(.-all)").expect("Valid CSS selector"));
static SELECTOR_VIEWS_ALL: Lazy<Selector> =
    Lazy::new(|| Selector::parse(".views.-all").expect("Valid CSS selector"));
static SELECTOR_RATING: Lazy<Selector> =
    Lazy::new(|| Selector::parse(".rating").expect("Valid CSS selector"));
static SELECTOR_DATE_TEXT: Lazy<Selector> =
    Lazy::new(|| Selector::parse(".date-text").expect("Valid CSS selector"));
static SELECTOR_PRO: Lazy<Selector> =
    Lazy::new(|| Selector::parse(".pro").expect("Valid CSS selector"));
static SELECTOR_MAINTABLE_TR: Lazy<Selector> =
    Lazy::new(|| Selector::parse(".maintable>tbody>tr").expect("Valid CSS selector"));
static SELECTOR_USERNAME: Lazy<Selector> =
    Lazy::new(|| Selector::parse(".username").expect("Valid CSS selector"));
static SELECTOR_USERNAME_A: Lazy<Selector> =
    Lazy::new(|| Selector::parse(".username>a").expect("Valid CSS selector"));
static SELECTOR_TD_CHILD_1_A: Lazy<Selector> =
    Lazy::new(|| Selector::parse("td:nth-child(1)>a").expect("Valid CSS selector"));
static SELECTOR_TD_CHILD_2: Lazy<Selector> =
    Lazy::new(|| Selector::parse("td:nth-child(2)").expect("Valid CSS selector"));
static SELECTOR_TD_CHILD_3: Lazy<Selector> =
    Lazy::new(|| Selector::parse("td:nth-child(3)").expect("Valid CSS selector"));
static SELECTOR_TD_CHILD_4: Lazy<Selector> =
    Lazy::new(|| Selector::parse("td:nth-child(4)").expect("Valid CSS selector"));
static SELECTOR_TD_CHILD_5: Lazy<Selector> =
    Lazy::new(|| Selector::parse("td:nth-child(5)").expect("Valid CSS selector"));
static SELECTOR_TD_CHILD_6_A: Lazy<Selector> =
    Lazy::new(|| Selector::parse("td:nth-child(6)>a").expect("Valid CSS selector"));

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
        let id = safe_attr_content(id_link, "href").replace("/", "");

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

        let format = safe_attr_content(parent.select(&SELECTOR_TD_CHILD_6_A).next(), "href")
            .replace("/archive/", "");

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
        let meta_element = dom.select(&SELECTOR_META_OG_URL).next();
        let username = safe_attr_content(meta_element, "content")
            .replace(&format!("{URL}/u/"), "");

        let parent = match dom.select(&SELECTOR_USER_VIEW).next() {
            Some(p) => p,
            None => {
                // Return default User if .user-view is not found
                return User {
                    username,
                    icon_url: String::new(),
                    website: None,
                    location: None,
                    profile_views: 0,
                    paste_views: 0,
                    rating: 0.0,
                    date_joined: 0,
                    pro: false,
                    pastes: Vec::new(),
                };
            }
        };

        let icon_img = parent.select(&SELECTOR_USER_ICON_IMG).next();
        let icon_url = safe_attr_content(icon_img, "src")
            .replace("/themes/pastebin/img/", "/imgs/")
            .replace("/cache/img/", "/imgs/");

        let website = parent
            .select(&SELECTOR_WEB)
            .next()
            .and_then(|e| e.value().attr("href").map(|s| s.to_owned()));

        let location = parent
            .select(&SELECTOR_LOCATION)
            .next()
            .map(|e| e.text().collect::<String>());

        let profile_views = parent
            .select(&SELECTOR_VIEWS_NOT_ALL)
            .next()
            .and_then(|el| {
                el.text()
                    .collect::<String>()
                    .replace(",", "")
                    .parse()
                    .ok()
            })
            .unwrap_or(0);

        let paste_views = parent
            .select(&SELECTOR_VIEWS_ALL)
            .next()
            .and_then(|el| {
                el.text()
                    .collect::<String>()
                    .replace(",", "")
                    .parse()
                    .ok()
            })
            .unwrap_or(0);

        let rating = parent
            .select(&SELECTOR_RATING)
            .next()
            .and_then(|el| {
                el.text()
                    .collect::<String>()
                    .parse()
                    .ok()
            })
            .unwrap_or(0.0);

        let date_joined = parent
            .select(&SELECTOR_DATE_TEXT)
            .next()
            .and_then(|el| el.value().attr("title"))
            .map(|date_str| safe_parse_date(date_str))
            .unwrap_or(0);

        let pro = parent
            .select(&SELECTOR_PRO)
            .next()
            .is_some();

        let pastes = dom
            .select(&SELECTOR_MAINTABLE_TR)
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
        let username_elem = parent.select(&SELECTOR_USERNAME).next();
        let username = safe_text_content(username_elem);

        let registered = parent
            .select(&SELECTOR_USERNAME_A)
            .next()
            .is_some();

        let pro = parent
            .select(&SELECTOR_PRO)
            .next()
            .is_some();

        let icon_img = parent.select(&SELECTOR_USER_ICON_IMG).next();
        let icon_url = safe_attr_content(icon_img, "src")
            .replace("/themes/pastebin/img/", "/imgs/")
            .replace("/cache/img/", "/imgs/");

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

    #[test]
    fn test_parse_simple_user_with_missing_username() {
        // Test SimpleUser::from_element with missing .username element
        let dom = Html::parse_document(
            r#"
            <div class="user">
                <div class="user-icon">
                    <img src="/themes/pastebin/img/user.png">
                </div>
            </div>
        "#,
        );

        let element = dom
            .select(&Selector::parse(".user").unwrap())
            .next()
            .unwrap();

        let user = SimpleUser::from_element(&element);

        // Should not panic and should return default values
        assert_eq!(user.username, "");
        assert_eq!(user.registered, false);
        assert_eq!(user.icon_url, "/imgs/user.png");
    }

    #[test]
    fn test_parse_simple_user_with_missing_icon() {
        // Test SimpleUser::from_element with missing icon
        let dom = Html::parse_document(
            r#"
            <div class="user">
                <div class="username">
                    <a href="/u/user">user</a>
                </div>
            </div>
        "#,
        );

        let element = dom
            .select(&Selector::parse(".user").unwrap())
            .next()
            .unwrap();

        let user = SimpleUser::from_element(&element);

        // Should not panic and should return default values
        assert_eq!(user.username, "user");
        assert_eq!(user.icon_url, "");
    }

    #[test]
    fn test_parse_user_with_missing_user_view() {
        // Test User::from_html with missing .user-view element
        let dom = Html::parse_document(
            r#"
            <html>
                <head>
                    <meta property="og:url" content="https://pastebin.com/u/testuser">
                </head>
                <body>
                    <div>No user view here</div>
                </body>
            </html>
        "#,
        );

        let user = User::from_html(&dom);

        // Should not panic and should return default values
        assert_eq!(user.username, "testuser");
        assert_eq!(user.icon_url, "");
        assert_eq!(user.profile_views, 0);
        assert_eq!(user.paste_views, 0);
        assert_eq!(user.rating, 0.0);
        assert_eq!(user.date_joined, 0);
        assert_eq!(user.pro, false);
        assert_eq!(user.pastes.len(), 0);
    }

    #[test]
    fn test_parse_user_with_missing_meta() {
        // Test User::from_html with missing meta tag
        let dom = Html::parse_document(
            r#"
            <html>
                <body>
                    <div class="user-view">
                        <div class="user-icon">
                            <img src="/themes/pastebin/img/user.png">
                        </div>
                    </div>
                </body>
            </html>
        "#,
        );

        let user = User::from_html(&dom);

        // Should not panic, username will be empty or default
        assert_eq!(user.username, "");
        assert_eq!(user.icon_url, "/imgs/user.png");
    }

    #[test]
    fn test_parse_user_with_missing_stats() {
        // Test User::from_html with missing profile stats
        let dom = Html::parse_document(
            r#"
            <html>
                <head>
                    <meta property="og:url" content="https://pastebin.com/u/testuser">
                </head>
                <body>
                    <div class="user-view">
                        <div class="user-icon">
                            <img src="/themes/pastebin/img/user.png">
                        </div>
                    </div>
                </body>
            </html>
        "#,
        );

        let user = User::from_html(&dom);

        // Should not panic and should use default values for missing stats
        assert_eq!(user.username, "testuser");
        assert_eq!(user.profile_views, 0);
        assert_eq!(user.paste_views, 0);
        assert_eq!(user.rating, 0.0);
        assert_eq!(user.date_joined, 0);
    }
}
