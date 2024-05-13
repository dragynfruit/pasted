
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

    pub fn get_body(&self, url: &str) -> String {
        self.agent.get(url).call().unwrap().into_string().unwrap()
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
        Html::parse_document(&self.get_body(url))
    }
}
