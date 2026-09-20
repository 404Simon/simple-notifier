use std::time::Duration;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const USER_AGENT: &str = "simple-notifier/0.1";

#[derive(Clone)]
pub struct HttpClient {
    agent: ureq::Agent,
}

impl HttpClient {
    pub fn new() -> Self {
        let config = ureq::Agent::config_builder()
            .timeout_global(Some(REQUEST_TIMEOUT))
            .build();

        Self {
            agent: config.new_agent(),
        }
    }

    pub fn get(&self, url: &str, bearer_token: Option<&str>) -> Result<String, String> {
        let mut request = self.agent.get(url).header("User-Agent", USER_AGENT);

        if let Some(token) = bearer_token {
            request = request.header("Authorization", &format!("Bearer {token}"));
        }

        let response = request
            .call()
            .map_err(|e| format!("HTTP error fetching {url}: {e}"))?;

        response
            .into_body()
            .read_to_string()
            .map_err(|e| format!("read error fetching {url}: {e}"))
    }
}
