use actix_web::http::{header, HeaderName, Method};
use lazy_static::lazy_static;

lazy_static! {
    static ref CONF: Config = Config::default();
}

pub fn config() -> &'static Config {
    &CONF
}


#[derive(Debug)]
pub struct LinkConfig {
    pub base_url: String,
    pub max_length: usize,
}

#[derive(Debug)]
pub struct CorsConfig {
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<Method>,
    pub allowed_headers: Vec<HeaderName>,
}

#[derive(Debug)]
pub struct Config {
    pub link: LinkConfig,
    pub cors: CorsConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            link: LinkConfig {
                base_url: "https://api.noclick.me".to_string(),
                max_length: 4096,
            },
            cors: CorsConfig {
                allowed_origins: vec![
                    "https://noclick.me",
                    "https://test.noclick.me",
                    "https://app.noclick.me",
                    "https://test.app.noclick.me",
                ].iter().map(|s| s.to_string()).collect(),
                allowed_methods: vec![Method::GET, Method::POST],
                allowed_headers: vec![
                    header::ACCEPT,
                    header::CONTENT_TYPE,
                ],
            },
        }
    }
}
