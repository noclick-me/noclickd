use crate::urlize::urlize_str;
use serde::{Deserialize, Serialize};
use std::io;
use url::Url;
use webpage::{Webpage, WebpageOptions};

pub use url::ParseError;

#[derive(Serialize, Deserialize, Debug)]
pub struct UrlInfo {
    pub url: String,
    pub site_name: String,
    pub title: String,
    pub description: Option<String>,
    pub text_content: String,
}

const MAX_LEN: usize = 1024;

impl UrlInfo {
    pub async fn fetch(url: &str) -> Result<Self, io::Error> {
        // TODO: async
        let info = Webpage::from_url(url, WebpageOptions::default())?;
        dbg!(&info.html.opengraph);

        let default = info.html.title.unwrap_or("".to_string());

        Ok(Self {
            url: url.to_string(),
            site_name: default.clone(),
            title: default.clone(),
            description: info.html.description,
            text_content: info.html.text_content[..MAX_LEN].to_string(),
        })
    }

    pub fn urlize(&self) -> Result<String, ParseError> {
        let url = Url::parse(&self.url)?;
        let host_path = vec![url.host_str().unwrap_or(""), url.path()].join("");
        let components = vec![url.scheme(), &host_path, &self.title, &self.text_content];
        let r = components
            .iter()
            .filter(|c| !c.is_empty())
            .map(|c| urlize_str(&c))
            .collect::<Vec<String>>()
            .join("/")[..MAX_LEN]
            .to_string();
        Ok(r)
    }
}
