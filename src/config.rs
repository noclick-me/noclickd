
#[derive(Debug)]
pub struct LinkConfig {
    pub base_url: String,
    pub max_length: usize,
}

#[derive(Debug)]
pub struct Config {
    pub link: LinkConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            link: LinkConfig {
                base_url: "https://noclick.me".to_string(),
                max_length: 4096,
            },
        }
    }
}
