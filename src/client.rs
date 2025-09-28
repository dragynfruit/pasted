use axum::http::Response;
use scraper::Html;
use ureq::{Agent, Body};
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
            agent: Agent::config_builder().max_redirects(0).build().new_agent(),
        }
    }

    pub fn get_response(&self, url: &str) -> Result<Response<Body>, ClientError> {
        Ok(self.agent.get(url).call()?)
    }

    pub fn post_response(&self, url: &str, form: Vec<(String, String)>) -> Result<Response<Body>, ClientError> {
        Ok(self.agent
            .post(url)
            .send_form(form)?)
    }

    pub fn get_string(&self, url: &str) -> Result<String, ClientError> {
        let mut response = self.get_response(url)?;
        Ok(response.body_mut().read_to_string()?)
    }

    pub fn post_string(&self, url: &str, form: Vec<(String, String)>) -> Result<String, ClientError> {
        let mut response = self.post_response(url, form)?;
        Ok(response.body_mut().read_to_string()?)
    }

    pub fn get_bytes(&self, url: &str) -> Result<Vec<u8>, ClientError> {
        let data = self.agent
            .get(url)
            .call()?
            .body_mut()
            .read_to_vec()?;
        
        Ok(data)
    }

    pub fn get_html(&self, url: &str) -> Result<Html, ClientError> {
        self.get_string(url)
            .map(|s| Html::parse_document(&s))
    }

    pub fn post_html(&self, url: &str, form: Vec<(String, String)>) -> Result<Html, ClientError> {
        self.post_string(url, form)
            .map(|s| Html::parse_document(&s))
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