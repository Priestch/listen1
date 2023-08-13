use super::media::{L1PlaylistInfo, Provider};
use super::utils::create_url;
use async_trait::async_trait;
use kuchiki::traits::TendrilSink;
use kuchiki::{parse_html, NodeRef};
use rand;
use rand::Rng;
use reqwest::{cookie, header, Client};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::string::String;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use url::Url;

const HOST: &'static str = "https://music.163.com";
const PLAYLIST_URL: &'static str = "https://music.163.com/discover/playlist";
const PLAYLIST_DETAIL_URL: &'static str = "https://music.163.com/weapi/v3/playlist/detail";
const SONG_DETAIL_URL: &'static str = "https://music.163.com/weapi/v3/song/detail";
const SONG_LYRICS_URL: &'static str = "https://music.163.com/weapi/song/lyric?csrf_token=";

const SECRET_CHARS: &'static str = "012345679abcdef";

pub struct Netease<'a> {
  pub client: &'a Client,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NeteaseFormData {
  pub params: String,
  pub enc_sec_key: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct TrackData {
  id: u64,
}

#[derive(Deserialize, Serialize, Debug)]
struct PlaylistData {
  id: u64,
  coverImgUrl: String,
  name: String,
  description: String,
  trackIds: Vec<TrackData>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PlaylistResponse {
  playlist: PlaylistData,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Artist {
  id: u64,
  name: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Album {
  id: u64,
  name: String,
  picUrl: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Song {
  id: u64,
  name: String,
  ar: Vec<Artist>,
  al: Album,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SongResponse {
  songs: Vec<Song>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Lyrics {
  version: u32,
  lyric: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct LyricResponse {
  lrc: Lyrics,
  tlyric: Option<Lyrics>,
}

#[derive(Default)]
pub struct PlaylistParams {
  order: String,
  limit: u32,
  offset: u32,
  cat: String,
}

impl PlaylistParams {
  fn to_params(&self) -> Vec<(String, String)> {
    let mut items: Vec<(String, String)> = vec![];
    items.push(("order".to_string(), self.order.to_string()));
    items.push(("limit".to_string(), self.limit.to_string()));
    items.push(("offset".to_string(), self.offset.to_string()));
    if self.cat != "" {
      items.push(("cat".to_string(), self.cat.to_string()));
    }

    items
  }
}

#[derive(Debug, Serialize)]
pub struct NeteasePlaylist {
  pub id: String,
  pub cover_img_url: String,
  pub source_url: String,
  pub title: String,
}

fn build_playlist_url(param: HashMap<String, String>) -> String {
  let mut items: Vec<(String, String)> = vec![];
  let order = param.get("order").unwrap().to_string();
  let offset = param.get("offset").unwrap().parse().unwrap();

  items.push(("order".to_string(), order));
  items.push(("offset".to_string(), offset));

  if let Some(val) = param.get("category_id") {
    items.push(("cat".to_string(), val.to_string()));
  }
  let url = Url::parse_with_params(PLAYLIST_URL, &items).unwrap();

  url.to_string()
}

#[async_trait]
impl Provider for Netease<'_> {
  async fn get_playlists(&self, params: HashMap<String, String>) -> Vec<L1PlaylistInfo> {
    let url = build_playlist_url(params);
    let resp = self
      .client
      .get(url)
      // .header("Referer", "http://music.163.com/")
      // .header("Origin", "http://music.163.com/")
      .send()
      .await
      .unwrap()
      .text()
      .await
      .unwrap();

    let document = parse_html().one(resp);
    let list_element = document.select_first(".m-cvrlst").unwrap();
    let mut playlists: Vec<L1PlaylistInfo> = Vec::new();
    for data in list_element.as_node().select("li").unwrap() {
      let playlist = Netease::create_playlist(&data.as_node()).into();
      playlists.push(playlist);
    }

    return playlists;
  }
}

fn get_time() -> u64 {
  let start = SystemTime::now();
  let since_the_epoch = start
    .duration_since(UNIX_EPOCH)
    .expect("Time went backwards");

  let in_ms = since_the_epoch.as_secs() * 1000 + since_the_epoch.subsec_nanos() as u64 / 1_000_000;

  in_ms
}

impl Netease<'_> {
  fn create_secret_key(size: u8) -> String {
    let mut rng = rand::thread_rng();
    let mut result: Vec<char> = vec![];
    let chars = SECRET_CHARS.chars();
    let range = 0..chars.count();

    for _i in 0..size {
      let index = rng.gen_range(range.clone());
      result.push(SECRET_CHARS.chars().nth(index).unwrap())
    }

    result.iter().collect()
  }

  pub fn create_cookie_jar() -> cookie::Jar {
    let uid = Netease::create_secret_key(32);
    let time = get_time();
    let nid = format!("{uid},{timestamp}", uid = uid, timestamp = time);

    let expire_at = (time + 1000 * 60 * 60 * 24 * 365 * 100) / 1000;
    println!("nid is {:?}", nid);

    let url = HOST.parse::<Url>().unwrap();
    let uid_cookie = format!(
      "_ntes_nuid={}; expires={}; Domain={}",
      uid,
      expire_at,
      url.domain().unwrap()
    );
    let nid_cookie = format!(
      "_ntes_nnid={}; expires={}; Domain={}",
      uid,
      expire_at,
      url.domain().unwrap()
    );

    let jar = cookie::Jar::default();
    jar.add_cookie_str(&uid_cookie, &url);
    jar.add_cookie_str(&nid_cookie, &url);

    jar
  }

  pub fn create_client() -> Client {
    let mut headers = header::HeaderMap::new();
    headers.insert("Referer", header::HeaderValue::from_static(HOST));

    let jar = Netease::create_cookie_jar();
    Client::builder()
      .default_headers(headers)
      .cookie_provider(Arc::new(jar))
      .build()
      .unwrap()
  }

  pub async fn get_playlist_detail(&self, payload: NeteaseFormData) -> PlaylistResponse {
    let response = self
      .client
      .post(PLAYLIST_DETAIL_URL)
      .form(&payload)
      .send()
      .await
      .unwrap()
      .json::<PlaylistResponse>()
      .await
      .unwrap();

    return response;
  }

  pub async fn get_song(&self, payload: NeteaseFormData) -> SongResponse {
    let response = self
      .client
      .post(SONG_DETAIL_URL)
      .form(&payload)
      .send()
      .await
      .unwrap()
      .json::<SongResponse>()
      .await
      .unwrap();

    response
  }

  pub async fn get_song_lyrics(&self, payload: NeteaseFormData) -> LyricResponse {
    let response = self
      .client
      .post(SONG_LYRICS_URL)
      .form(&payload)
      .send()
      .await
      .unwrap()
      .json::<LyricResponse>()
      .await
      .unwrap();

    response
  }

  fn create_playlist(node_ref: &NodeRef) -> NeteasePlaylist {
    let cover_node = node_ref.select_first("img").unwrap();
    let cover_url = cover_node
      .attributes
      .borrow()
      .get("src")
      .unwrap()
      .replace("140y140", "512y512");

    let title_container = node_ref.select_first("div").unwrap();
    let anchor_el = title_container.as_node().select_first("a").unwrap();
    let anchor_attrs = anchor_el.attributes.borrow();
    let title = anchor_attrs.get("title").unwrap().to_string();
    let href = anchor_attrs.get("href").unwrap();
    let url = create_url(href).unwrap();
    let pair = url
      .query_pairs()
      .find(|(name, _value)| name == "id")
      .unwrap();

    let mut id = "neplaylist_".to_string();
    let playlist_id = &pair.1.into_owned();
    id.push_str(playlist_id);
    let mut source_url = "https://music.163.com/#/playlist?id=".to_string();
    source_url.push_str(playlist_id);
    let playlist = NeteasePlaylist {
      id,
      cover_img_url: cover_url,
      source_url,
      title,
    };

    playlist
  }

  pub async fn get_self_playlists(&self, params: PlaylistParams) -> Vec<NeteasePlaylist> {
    let url = Url::parse_with_params(PLAYLIST_URL, params.to_params()).unwrap();
    let resp = self.client.get(url.clone()).send().await.unwrap().text().await.unwrap();

    let document = parse_html().one(resp);
    let list_element = document.select_first(".m-cvrlst").unwrap();
    let mut playlists: Vec<NeteasePlaylist> = Vec::new();
    for data in list_element.as_node().select("li").unwrap() {
      let playlist = Netease::create_playlist(&data.as_node());
      playlists.push(playlist);
    }

    playlists
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
  async fn it_get_playlists() {
    let client = Netease::create_client();
    let netease = Netease { client: &client };
    let params = PlaylistParams {
      cat: "".to_string(),
      order: "hot".to_string(),
      limit: 35,
      offset: 0,
    };
    let playlists = netease.get_self_playlists(params).await;
    assert_eq!(playlists.len(), 35);
  }
}