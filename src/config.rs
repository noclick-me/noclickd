use lazy_static::lazy_static;
lazy_static! {
    static ref CONF: Config = Config::default();
}

pub fn config() -> &'static Config {
    &CONF
}

#[derive(Debug)]
pub struct ApiConfig {
    pub host: String,
}

#[derive(Debug)]
pub struct LinkConfig {
    pub base_url: String,
    pub max_length: usize,
}

use actix_web::http::{HeaderName, Method};
#[derive(Debug)]
pub struct CorsConfig {
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<Method>,
    pub allowed_headers: Vec<HeaderName>,
}

#[derive(Debug)]
pub struct ViewerConfig {
    pub host: String,
}

#[derive(Debug)]
pub struct WebappConfig {
    pub redirect_from_path: String,
    pub redirect_from_host: String,
    pub redirect_to: String,
}

#[derive(Debug)]
pub struct Config {
    pub api: ApiConfig,
    pub link: LinkConfig,
    pub cors: CorsConfig,
    pub viewer: ViewerConfig,
    pub webapp: WebappConfig,
}

impl Default for Config {
    fn default() -> Self {
        use actix_web::http::header;
        Self {
            api: ApiConfig {
                host: "api.noclick.me".to_string(),
            },
            link: LinkConfig {
                base_url: "https://noclick.me".to_string(),
                max_length: 4096,
            },
            cors: CorsConfig {
                allowed_origins: vec![
                    "https://noclick.me",
                    "https://web.noclick.me",
                    "https://www.noclick.me",
                    "https://test.noclick.me",
                    "https://app.noclick.me",
                    "https://test.app.noclick.me",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
                allowed_methods: vec![Method::GET, Method::POST],
                allowed_headers: vec![header::ACCEPT, header::CONTENT_TYPE],
            },
            viewer: ViewerConfig {
                host: "noclick.me".to_string(),
            },
            webapp: WebappConfig {
                redirect_from_path: "/".to_string(),
                redirect_from_host: "noclick.me".to_string(),
                redirect_to: "https://app.noclick.me/".to_string(),
            },
        }
    }
}
