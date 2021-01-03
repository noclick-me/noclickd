use crate::urlize::urlize_str;

use actix_web::{
    client::Client,
    http::header::CONTENT_LENGTH,
    web::{BufMut, BytesMut},
};
use serde::{Deserialize, Serialize};
use std::error::Error;
use url::Url;
use webpage::HTML;

pub use url::ParseError;

#[derive(Serialize, Deserialize, Debug)]
pub struct UrlInfo {
    pub url: String,
    pub site_name: String,
    pub title: String,
    pub description: Option<String>,
    pub text_content: String,
}

impl UrlInfo {
    pub async fn fetch(url: &str) -> Result<Self, Box<dyn Error>> {
        let client = Client::default();
        let mut response = client.get(url).send().await?;
        use futures::stream::TryStreamExt;

        let mut len = None;
        if let Some(l) = response.headers().get(&CONTENT_LENGTH) {
            if let Ok(s) = l.to_str() {
                if let Ok(l) = s.parse::<usize>() {
                    len = Some(l)
                } else {
                    println!("WARN: can't parse Content-Length: {:?}", s);
                }
            } else {
                println!("WARN: can't decode Content-Length: {:?}", len);
            }
        }
        // TODO: accumulate body, check for maxiumum length
        let mut buf = BytesMut::with_capacity(len.unwrap_or(256 * 1024));
        while let Some(chunk) = response.try_next().await? {
            println!(
                "received chunk ({} bytes): {}\n\n\n",
                chunk.len(),
                String::from_utf8_lossy(&chunk)
            );
            buf.put(chunk);
        }

        let info = HTML::from_string(
            String::from_utf8_lossy(&buf[..]).to_string(),
            Some(url.to_string()),
        )?;
        dbg!(&info);

        let default = info.title.unwrap_or("".to_string());

        Ok(Self {
            url: url.to_string(),
            site_name: default.clone(),
            title: default.clone(),
            description: info.description,
            text_content: info.text_content.clone(),
        })
    }

    pub fn urlize(&self, max_length: usize) -> Result<String, ParseError> {
        let url = Url::parse(&self.url)?;
        let host_path = vec![url.host_str().unwrap_or(""), url.path()].join("");
        let components = vec![url.scheme(), &host_path, &self.title, &self.text_content];
        let mut r = components
            .iter()
            .filter(|c| !c.is_empty())
            .map(|c| urlize_str(&c))
            .collect::<Vec<String>>()
            .join("/");
        r.truncate(max_length); // only safe because we know urlize_str() will only produce ASCII

        Ok(r)
    }
}
