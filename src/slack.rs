pub mod slack {
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    #[serde(untagged)]
    pub enum ApiResponse<T> {
        Ok { ok: bool, user: T },
        Err { ok: bool, error: String },
    }

    #[derive(Debug, Deserialize)]
    pub struct SlackUser {
        pub id: String,
        pub profile: SlackProfile,
    }

    #[derive(Debug, Deserialize)]
    pub struct SlackProfile {
        pub display_name: String,
        pub real_name: String,
    }

    pub struct SlackClient {
        http_client: reqwest::blocking::Client,
        bot_token: String,
        display_name_cache: std::collections::HashMap<String, String>,

        api_endpoints: SlackApiEndpoints,
    }
    struct SlackApiEndpoints {
        user_info: String,
    }

    impl SlackClient {
        pub fn new(bot_token: String) -> Self {
            SlackClient {
                http_client: reqwest::blocking::Client::new(),
                bot_token,
                display_name_cache: std::collections::HashMap::new(),
                api_endpoints: SlackApiEndpoints {
                    user_info: "https://slack.com/api/users.info".to_string(),
                },
            }
        }

        /// Get the display name of the provided Slack user.
        /// Caches API responses in memory.
        pub fn get_display_name(&mut self, user_id: &str) -> Option<String> {
            if let Some(cached_name) = self.display_name_cache.get(user_id) {
                return Some(cached_name.clone());
            }

            let response = self
                .http_client
                .get(&self.api_endpoints.user_info)
                .query(&[("user", user_id)])
                .bearer_auth(&self.bot_token)
                .send()
                // TODO this is not the best error handling
                .ok()?;

            let parsed: ApiResponse<SlackUser> = response.json().ok()?;

            match parsed {
                ApiResponse::Ok { user, .. } => {
                    let display_name = user.profile.display_name;
                    self.display_name_cache
                        .insert(user_id.to_string(), display_name.clone());
                    Some(display_name)
                }
                ApiResponse::Err { error, .. } => {
                    eprintln!("Error parsing user info: {}", error);
                    None
                }
            }
        }
    }
}
