use once_cell::sync::Lazy;
use scraper::{ElementRef, Html, Selector};
use serde::Serialize;

use crate::constants::URL;

use super::{FromElement, FromHtml, parse_date};
use super::utils::{safe_text_content, safe_attr_content, safe_select, safe_parse_number};

// Helper function to safely parse dates with fallback to 0
fn safe_parse_date(date_str: &str) -> i64 {
    parse_date(date_str).unwrap_or_else(|e| {
        eprintln!("Failed to parse date '{}': {}", date_str, e).expect("Should not error");
        0 // Unix epoch as fallback
    })
}

// Pre-compiled selectors to avoid unwrap() calls
static SELECTOR_META_OG_URL: Lazy<Selector> =
    Lazy::new(|| Selector::parse("meta[property='og:url']").expect("Valid CSS selector")).expect("Should not error");
static SELECTOR_USER_VIEW: Lazy<Selector> =
    Lazy::new(|| Selector::parse(".user-view").expect("Valid CSS selector")).expect("Should not error");
static SELECTOR_USER_ICON_IMG: Lazy<Selector> =
    Lazy::new(|| Selector::parse(".user-icon>img").expect("Valid CSS selector")).expect("Should not error");
static SELECTOR_WEB: Lazy<Selector> =
    Lazy::new(|| Selector::parse(".web").expect("Valid CSS selector")).expect("Should not error");
static SELECTOR_LOCATION: Lazy<Selector> =
    Lazy::new(|| Selector::parse(".location").expect("Valid CSS selector")).expect("Should not error");
static SELECTOR_VIEWS_NOT_ALL: Lazy<Selector> =
    Lazy::new(|| Selector::parse(".views:not(.-all)").expect("Valid CSS selector")).expect("Should not error");
static SELECTOR_VIEWS_ALL: Lazy<Selector> =
    Lazy::new(|| Selector::parse(".views.-all").expect("Valid CSS selector")).expect("Should not error");
static SELECTOR_RATING: Lazy<Selector> =
    Lazy::new(|| Selector::parse(".rating").expect("Valid CSS selector")).expect("Should not error");
static SELECTOR_DATE_TEXT: Lazy<Selector> =
    Lazy::new(|| Selector::parse(".date-text").expect("Valid CSS selector")).expect("Should not error");
static SELECTOR_PRO: Lazy<Selector> =
    Lazy::new(|| Selector::parse(".pro").expect("Valid CSS selector")).expect("Should not error");
static SELECTOR_MAINTABLE_TR: Lazy<Selector> =
    Lazy::new(|| Selector::parse(".maintable>tbody>tr").expect("Valid CSS selector")).expect("Should not error");
static SELECTOR_USERNAME: Lazy<Selector> =
    Lazy::new(|| Selector::parse(".username").expect("Valid CSS selector")).expect("Should not error");
static SELECTOR_USERNAME_A: Lazy<Selector> =
    Lazy::new(|| Selector::parse(".username>a").expect("Valid CSS selector")).expect("Should not error");
static SELECTOR_TD_CHILD_1_A: Lazy<Selector> =
    Lazy::new(|| Selector::parse("td:nth-child(1)>a").expect("Valid CSS selector")).expect("Should not error");
static SELECTOR_TD_CHILD_2: Lazy<Selector> =
    Lazy::new(|| Selector::parse("td:nth-child(2)").expect("Valid CSS selector")).expect("Should not error");
static SELECTOR_TD_CHILD_3: Lazy<Selector> =
    Lazy::new(|| Selector::parse("td:nth-child(3)").expect("Valid CSS selector")).expect("Should not error");
static SELECTOR_TD_CHILD_4: Lazy<Selector> =
    Lazy::new(|| Selector::parse("td:nth-child(4)").expect("Valid CSS selector")).expect("Should not error");
static SELECTOR_TD_CHILD_5: Lazy<Selector> =
    Lazy::new(|| Selector::parse("td:nth-child(5)").expect("Valid CSS selector")).expect("Should not error");
static SELECTOR_TD_CHILD_6_A: Lazy<Selector> =
    Lazy::new(|| Selector::parse("td:nth-child(6)>a").expect("Valid CSS selector")).expect("Should not error");

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
    fn from_element(parent: &ElementRef) -> Result<Self, String> {
        let id_link = safe_select(parent, &SELECTOR_TD_CHILD_1_A).expect("Should not error");
        let id = safe_attr_content(id_link, "href").replace("/", "").expect("Should not error");

        let title = safe_text_content(id_link).expect("Should not error");

        let age = safe_text_content(safe_select(parent, &SELECTOR_TD_CHILD_2)).expect("Should not error");

        let expires = safe_text_content(safe_select(parent, &SELECTOR_TD_CHILD_3)).expect("Should not error");

        let views: u32 = safe_select(parent, &SELECTOR_TD_CHILD_4)
            .map(|el| safe_parse_number(&el.text().collect::<String>()))
            .unwrap_or(0).expect("Should not error");

        let num_comments: u32 = safe_select(parent, &SELECTOR_TD_CHILD_5)
            .map(|el| safe_parse_number(&el.text().collect::<String>()))
            .unwrap_or(0).expect("Should not error");

        let format = safe_attr_content(safe_select(parent, &SELECTOR_TD_CHILD_6_A), "href")
            .replace("/archive/", "").expect("Should not error");

        Ok(UserPaste {
            id,
            title,
            age,
            expires,
            views,
            num_comments,
            format,
        })
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
    fn from_html(dom: &Html) -> Result<Self, String> {
        let meta_element = dom.select(&SELECTOR_META_OG_URL).next().expect("Should not error");
        let username = safe_attr_content(meta_element, "content")
            .replace(&format!("{URL}/u/"), "").expect("Should not error");

        let parent = match dom.select(&SELECTOR_USER_VIEW).next() {
            Some(p) => p,
            None => {
                // Return default User if .user-view is not found
                return Ok(User {
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
                }).expect("Should not error");
            }
        };

        let icon_img = safe_select(&parent, &SELECTOR_USER_ICON_IMG).expect("Should not error");
        let icon_url = safe_attr_content(icon_img, "src")
            .replace("/themes/pastebin/img/", "/imgs/")
            .replace("/cache/img/", "/imgs/").expect("Should not error");

        let website = safe_select(&parent, &SELECTOR_WEB)
            .and_then(|e| e.value().attr("href").map(|s| s.to_owned())).expect("Should not error");

        let location = safe_select(&parent, &SELECTOR_LOCATION)
            .map(|e| e.text().collect::<String>()).expect("Should not error");

        let profile_views: u32 = safe_select(&parent, &SELECTOR_VIEWS_NOT_ALL)
            .map(|el| safe_parse_number(&el.text().collect::<String>()))
            .unwrap_or(0).expect("Should not error");

        let paste_views: u32 = safe_select(&parent, &SELECTOR_VIEWS_ALL)
            .map(|el| safe_parse_number(&el.text().collect::<String>()))
            .unwrap_or(0).expect("Should not error");

        let rating: f32 = safe_select(&parent, &SELECTOR_RATING)
            .and_then(|el| el.text().collect::<String>().parse().ok())
            .unwrap_or(0.0).expect("Should not error");

        let date_joined = safe_select(&parent, &SELECTOR_DATE_TEXT)
            .and_then(|el| el.value().attr("title"))
            .map(|date_str| safe_parse_date(date_str))
            .unwrap_or(0).expect("Should not error");

        let pro = safe_select(&parent, &SELECTOR_PRO).is_some().expect("Should not error");

        let pastes = dom
            .select(&SELECTOR_MAINTABLE_TR)
            .enumerate()
            .filter(|&(i, _)| i != 0)
            .map(|(_, v)| UserPaste::from_element(&v))
            .collect::<Result<Vec<UserPaste>, String>>()?;

        Ok(User {
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
        })
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
    fn from_element(parent: &ElementRef) -> Result<Self, String> {
        let username_elem = safe_select(parent, &SELECTOR_USERNAME).expect("Should not error");
        let username = safe_text_content(username_elem).expect("Should not error");

        let registered = safe_select(parent, &SELECTOR_USERNAME_A).is_some().expect("Should not error");

        let pro = safe_select(parent, &SELECTOR_PRO).is_some().expect("Should not error");

        let icon_img = safe_select(parent, &SELECTOR_USER_ICON_IMG).expect("Should not error");
        let icon_url = safe_attr_content(icon_img, "src")
            .replace("/themes/pastebin/img/", "/imgs/")
            .replace("/cache/img/", "/imgs/").expect("Should not error");

        Ok(SimpleUser {
            username,
            registered,
            pro,
            icon_url,
        })
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
        )
        .expect("Should not error");

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

        let user = SimpleUser::from_element(&element).expect("Should not error");

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

        let user = SimpleUser::from_element(&element).expect("Should not error");

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

        let user = User::from_html(&dom).expect("Should not error");

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

        let user = User::from_html(&dom).expect("Should not error");

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

        let user = User::from_html(&dom).expect("Should not error");

        // Should not panic and should use default values for missing stats
        assert_eq!(user.username, "testuser");
        assert_eq!(user.profile_views, 0);
        assert_eq!(user.paste_views, 0);
        assert_eq!(user.rating, 0.0);
        assert_eq!(user.date_joined, 0);
    }
}
