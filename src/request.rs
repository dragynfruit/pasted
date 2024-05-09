use scraper::Html;
use ureq::Agent;

pub fn get_body(agent: Agent, url: &str) -> String {
    agent.get(url).call().unwrap().into_string().unwrap()
}

pub fn get_bytes(agent: Agent, url: &str) -> Vec<u8> {
    let mut data = Vec::new();
    agent
        .get(url)
        .call()
        .unwrap()
        .into_reader()
        .read_to_end(&mut data)
        .unwrap();
    data
}

pub fn get_html(agent: Agent, url: &str) -> Html {
    Html::parse_document(&get_body(agent, url))
}
