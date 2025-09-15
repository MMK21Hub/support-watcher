pub mod slack {
    pub struct SlackClient {
        http_client: reqwest::Client,
        bot_token: String,
        display_name_cache: std::collections::HashMap<String, String>,
    }

    impl SlackClient {
        pub fn new(bot_token: String) -> Self {
            SlackClient {
                http_client: reqwest::Client::new(),
                bot_token,
                display_name_cache: std::collections::HashMap::new(),
            }
        }
    }
}
