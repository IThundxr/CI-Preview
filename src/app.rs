use reqwest::Client;

#[derive(Clone)]
pub struct App {
    pub https: Client,
}

impl App {
    pub fn new() -> Self {
        Self {
            https: Client::new(),
        }
    }
}
