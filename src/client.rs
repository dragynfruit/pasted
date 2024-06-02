use scraper::Html;
use ureq::{Agent, AgentBuilder};

#[derive(Clone)]
pub struct Client {
    agent: Agent,
}

impl Client {
    pub fn new() -> Self {
        Self {
            agent: AgentBuilder::new().redirects(0).build(),
        }
    }

    // pub fn get_agent(&self) -> &Agent {
    //     &self.agent
    // }

    pub fn get_response(&self, url: &str) -> ureq::Response {
        self.agent.get(url).call().unwrap()
    }

    pub fn post_response(&self, url: &str, form: (String, Vec<u8>)) -> ureq::Response {
        self.agent
            .post(url)
            .set("Content-Type", &form.0)
            .send_bytes(&form.1)
            .unwrap()
    }

    pub fn get_string(&self, url: &str) -> String {
        self.get_response(url).into_string().unwrap()
    }

    pub fn post_string(&self, url: &str, form: (String, Vec<u8>)) -> String {
        self.post_response(url, form).into_string().unwrap()
    }

    pub fn get_bytes(&self, url: &str) -> Vec<u8> {
        let mut data = Vec::new();
        self.agent
            .get(url)
            .call()
            .unwrap()
            .into_reader()
            .read_to_end(&mut data)
            .unwrap();
        data
    }

    pub fn get_html(&self, url: &str) -> Html {
        Html::parse_document(&&self.get_string(url))
    }

    pub fn post_html(&self, url: &str, form: (String, Vec<u8>)) -> Html {
        Html::parse_document(&self.post_string(url, form))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client() {
        let client = Client::new();
        let response = client.get_response("https://pastebin.com").status();
        assert_eq!(response, 200);
    }

    #[test]
    fn test_get_string() {
        let client = Client::new();
        let response = client.get_string("https://pastebin.com");
        assert!(response.contains("Pastebin.com"));
    }

    #[test]
    fn test_get_bytes() {
        let client = Client::new();
        let response = client.get_bytes("https://pastebin.com");
        assert!(response.len() > 0);
    }
}