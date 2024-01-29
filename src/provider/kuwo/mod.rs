mod playlist;

use reqwest::{Client, header};

const HOST: &'static str = "https://www.kuwo.cn";

pub fn create_client() -> Client {
    let mut headers = header::HeaderMap::new();
    headers.insert("Referer", header::HeaderValue::from_static(HOST));

    Client::builder()
        .default_headers(headers)
        .connection_verbose(true)
        .cookie_store(true)
        .build()
        .unwrap()
}


pub struct Kuwo {
    pub client: Client,
}

impl Kuwo {
    async fn get_playlists() {
    }
}