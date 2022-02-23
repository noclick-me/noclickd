use config::ConfigError;
use config::{File, FileFormat};
use lazy_static::lazy_static;
use serde::Deserialize;

lazy_static! {
    static ref CONF: Config = Config::new("config.toml".to_string()).unwrap();
}

pub fn config() -> &'static Config {
    &CONF
}
#[derive(Debug, Deserialize)]
pub struct ApiConfig {
    pub host: String,
}

#[derive(Debug, Deserialize)]
pub struct CorsConfig {
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub allowed_headers: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct DbConfig {
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct LimitsConfig {
    pub global_requests_per_day: usize,
}

#[derive(Debug, Deserialize)]
pub struct LinkConfig {
    pub base_url: String,
    pub max_length: usize,
}

#[derive(Debug, Deserialize)]
pub struct ViewerConfig {
    pub host: String,
}

#[derive(Debug, Deserialize)]
pub struct WebappConfig {
    pub redirect_from_path: String,
    pub redirect_from_host: String,
    pub redirect_to: String,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub api: ApiConfig,
    pub cors: CorsConfig,
    pub db: DbConfig,
    pub limits: LimitsConfig,
    pub link: LinkConfig,
    pub viewer: ViewerConfig,
    pub webapp: WebappConfig,
}

impl Config {
    fn new(config_path: String) -> Result<Self, ConfigError> {
        config::Config::builder()
            .set_default("db.url", "sqlite://db.sqlite?mode=rwc".to_string())?
            .set_default("cors.allowed_methods", vec!["GET", "POST"])?
            .set_default("cors.allowed_headers", vec!["ACCEPT", "CONTENT_TYPE"])?
            .set_default("limits.global_requests_per_day", 10000)?
            .set_default("link.max_length", 4096)?
            .set_default("webapp.redirect_from_path", "/".to_string())?
            .add_source(File::new(&config_path, FileFormat::Toml))
            .build()?
            .try_deserialize()
    }
}
