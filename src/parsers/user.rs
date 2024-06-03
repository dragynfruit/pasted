use scraper::{ElementRef, Selector};
use serde::Serialize;

use super::FromElement;

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
