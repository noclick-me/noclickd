use crate::urlize::urlize_str;

use actix_web::{
    client::Client,
    web::{BufMut, BytesMut},
};
use kuchiki::traits::*;
use serde::{Deserialize, Serialize};
use std::error::Error;
use unicode_segmentation::UnicodeSegmentation;
use url::Url;

pub use url::ParseError;

// TODO: This will need to be changed to split information retrieved via HTTP (which will be
//       available for alll scrappers, at least when the resource is retrieved using HTTP), from
//       resource-specific data, like HTML title, meta tags, etc, or EXIF metadata for images (to
//       put a couple of examples).
#[derive(Serialize, Deserialize, Debug)]
pub struct ResourceInfo {
    pub url: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub content: String,
}

impl ResourceInfo {
    pub fn parse_doc(doc: kuchiki::NodeRef, url: Option<&str>) -> Result<Self, Box<dyn Error>> {
        let mut title = "".to_string();
        let mut selector = doc.select("title").map_err(|_| ParseError::EmptyHost)?; // FIXME
        if let Some(m) = selector.next() {
            title = m.as_node().text_contents().trim().to_string(); // 2 allocations, not ideal
        }

        let mut description = None;
        let mut selector = doc
            .select("meta[name='description']")
            .map_err(|_| ParseError::EmptyHost)?; // FIXME
        if let Some(m) = selector.next() {
            let elem = m.as_node().as_element().unwrap();
            description = elem.attributes.borrow().get("content").map(String::from);
        }

        let text_content_vec: Vec<_> = doc
            .select("p")
            .map_err(|_| ParseError::EmptyHost)? // FIXME
            .map(|n| n.text_contents())
            .collect();

        let content = text_content_vec
            .iter()
            .map(|s| s.trim())
            .collect::<Vec<_>>()
            .join(" ")
            .graphemes(true)
            .take(1024)
            .collect();

        Ok(Self {
            url: url.map(String::from),
            title,
            description,
            content,
        })
    }

    pub fn parse_str(text: &str, url: Option<&str>) -> Result<Self, Box<dyn Error>> {
        Self::parse_doc(kuchiki::parse_html().one(text), url)
    }

    pub fn parse_file(path: &str, url: Option<&str>) -> Result<Self, Box<dyn Error>> {
        Self::parse_str(&String::from_utf8_lossy(&std::fs::read(path)?), url)
    }

    pub async fn fetch(url: &str) -> Result<Self, Box<dyn Error>> {
        let client = Client::default();
        use actix_web::http::header::*;
        let req = client
            .get(url)
            .set(Accept(vec![
                qitem("text/html".parse().unwrap()),
                qitem("application/xhtml+xml".parse().unwrap()),
                QualityItem::new("text/xml".parse().unwrap(), q(900)),
                qitem("image/webp".parse().unwrap()),
                QualityItem::new("*/*".parse().unwrap(), q(800)),
            ]))
            .set_header(USER_AGENT, "curl/7.72.0")
            .set_header(REFERER, "noclick.me");
        let mut response = req.send().await?;
        dbg!(response.headers());
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
            buf.put(chunk);
        }

        Self::parse_str(&*String::from_utf8_lossy(&buf[..]), Some(url))
    }

    pub fn urlize(&self, max_length: usize) -> Result<String, ParseError> {
        let mut components = Vec::new();
        // We need to bind them to this scope because we borrow them in an internal scope we want them to outlive
        let (path, parsed_url);
        if let Some(ref url) = self.url {
            parsed_url = Url::parse(&url)?;
            components.push(parsed_url.scheme());
            path = vec![parsed_url.host_str().unwrap_or(""), parsed_url.path()].join("");
            components.push(&path);
        }
        components.push(&self.title);
        components.push(&self.content);
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
