use scraper::Html;
use ureq::{Agent, AgentBuilder};
use std::fmt;

#[derive(Clone)]
pub struct Client {
    agent: Agent,
}

#[derive(Debug)]
pub enum ClientError {
    UreqError(ureq::Error),
    IoError(std::io::Error)
}

impl From<ureq::Error> for ClientError {
    fn from(value: ureq::Error) -> Self {
        ClientError::UreqError(value)
    }
}

impl From<std::io::Error> for ClientError {
    fn from(value: std::io::Error) -> Self {
        ClientError::IoError(value)
    }
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClientError::UreqError(err) => write!(f, "HTTP request error: {}", err),
            ClientError::IoError(err) => write!(f, "IO error: {}", err),
        }
    }
}

impl Client {
    pub fn new() -> Self {
        Self {
            agent: AgentBuilder::new().redirects(0).build(),
        }
    }

    pub fn get_response(&self, url: &str) -> Result<ureq::Response, ClientError> {
        Ok(self.agent.get(url).call()?)
    }

    pub fn post_response(&self, url: &str, form: (String, Vec<u8>)) -> Result<ureq::Response, ClientError> {
        Ok(self.agent
            .post(url)
            .set("Content-Type", &form.0)
            .send_bytes(&form.1)?)
    }

    pub fn get_string(&self, url: &str) -> Result<String, ClientError> {
        Ok(self.get_response(url)?.into_string()?)
    }

    pub fn post_string(&self, url: &str, form: (String, Vec<u8>)) -> Result<String, ClientError> {
        Ok(self.post_response(url, form)?.into_string()?)
    }

    pub fn get_bytes(&self, url: &str) -> Result<Vec<u8>, ClientError> {
        let mut data = Vec::new();
        self.agent
            .get(url)
            .call()
            .unwrap()
            .into_reader()
            .read_to_end(&mut data)
            .unwrap();
        Ok(data)
    }

    pub fn get_html(&self, url: &str) -> Result<Html, ClientError> {
        Ok(Html::parse_document(&&self.get_string(url)?))
    }

    pub fn post_html(&self, url: &str, form: (String, Vec<u8>)) -> Result<Html, ClientError> {
        Ok(Html::parse_document(&self.post_string(url, form)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client() {
        let client = Client::new();
        let response = client.get_response("https://pastebin.com").unwrap().status();
        assert_eq!(response, 200);
    }

    #[test]
    fn test_get_string() {
        let client = Client::new();
        let response = client.get_string("https://pastebin.com").unwrap();
        assert!(response.contains("Pastebin.com"));
    }

    #[test]
    fn test_get_bytes() {
        let client = Client::new();
        let response = client.get_bytes("https://pastebin.com").unwrap();
        assert!(response.len() > 0);
    }
}