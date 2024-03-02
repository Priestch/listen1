use crate::provider::utils;
use crate::provider::utils::to_query_string;
use base64::engine::general_purpose;
use base64::Engine;
use kuchiki::traits::TendrilSink;
use kuchiki::{parse_html, NodeRef};
use libaes::Cipher;
use num_bigint::BigInt;
use rand::Rng;
use regex::Regex;
use reqwest::{cookie, header, Client};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;
use std::convert::TryInto;
use std::fmt::Write;
use std::io::Read;
use std::ops::Deref;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use url::Url;

fn get_time() -> u64 {
    let start = SystemTime::now();
    let duration = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    let in_ms = duration.as_secs() * 1000 + duration.subsec_nanos() as u64 / 1_000_000;

    in_ms
}

pub fn to_hex(text: &str) -> String {
    text.as_bytes()
        .iter()
        .map(|x| format!("{:x?}", x))
        .collect::<String>()
}

pub fn reverse_acsii_str(text: &str) -> String {
    let reversed_chars: Vec<_> = text.as_bytes().iter().rev().copied().collect();
    String::from_utf8(reversed_chars).expect("Invalid UTF-8")
}

#[derive(Deserialize, Debug)]
struct TrackData {
    id: u64,
}

#[derive(Deserialize, Debug)]
struct PlaylistData {
    id: u64,
    #[serde(rename(deserialize = "name"))]
    title: String,
    #[serde(rename(deserialize = "coverImgUrl"))]
    cover_img_url: String,
    description: String,
    #[serde(rename(deserialize = "trackIds"))]
    track_ids: Vec<TrackData>,
}

#[derive(Deserialize, Debug)]
pub struct PlaylistResponse {
    playlist: PlaylistData,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct NeteaseFormData {
    pub params: String,
    #[serde(rename(serialize = "encSecKey"))]
    pub enc_sec_key: String,
}

#[derive(Debug, Serialize)]
pub struct NeteasePlaylist {
    pub id: u64,
    pub cover_img_url: String,
    pub source_url: String,
    pub title: String,
}

#[derive(Default, Serialize)]
pub struct NeteasePlaylistsParams {
    pub order: String,
    pub limit: u32,
    pub offset: Option<u64>,
    #[serde(rename(serialize = "cat"))]
    #[serde(skip_serializing_if = "String::is_empty")]
    pub category_id: String,
}

#[derive(Serialize)]
pub struct NeteasePlaylistParams {
    pub id: u64,
    pub offset: u64,
    pub total: bool,
    pub limit: u32,
    pub n: u32,
    pub csrf_token: String,
}

impl Default for NeteasePlaylistParams {
    fn default() -> Self {
        Self {
            id: 0,
            offset: 0,
            total: true,
            limit: 1000,
            n: 1000,
            csrf_token: "".to_string(),
        }
    }
}

#[derive(Serialize)]
pub struct SongParams {
    pub c: String,
    pub ids: String,
}

#[derive(Serialize)]
pub struct LyricParams {
    pub id: u64,
    pub lv: i32,
    pub tv: i32,
    pub csrf_token: String,
}

impl Default for LyricParams {
    fn default() -> Self {
        Self {
            id: 0,
            lv: -1,
            tv: -1,
            csrf_token: "".to_string(),
        }
    }
}

impl SongParams {
    fn new(ids: Vec<u64>) -> Self {
        let c = ids
            .iter()
            .map(|id| json!({"id": id}).to_string())
            .collect::<Vec<String>>()
            .join(",");

        Self {
            c: "[".to_string() + &c + "]",
            ids: ids
                .iter()
                .map(|i| i.to_string())
                .collect::<Vec<String>>()
                .join(","),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct TopPlaylistTrack {
    pub first: String,
    pub second: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct TopPlaylist {
    pub id: u64,
    pub name: String,
    pub description: Option<String>,
    pub cover_img_id: u64,
    pub cover_img_url: String,
    pub subscribed_count: u64,
    pub play_count: u64,
    pub track_count: u64,
    pub update_time: u64,
    pub create_time: u64,
    pub ordered: bool,
    // pub update_frequency: Option<String>,
    // pub tracks: Option<Vec<TopPlaylistTrack>>,
    comment_thread_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct NeteaseTopPlaylistsResponse {
    code: u32,
    list: Vec<TopPlaylist>,
}

#[derive(Debug, Deserialize)]
pub struct ArtistSummary {
    pub id: u64,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct Artist {
    pub id: u64,
    pub name: String,
    #[serde(rename(deserialize = "picUrl"))]
    img_url: String,
    #[serde(rename(deserialize = "picId"))]
    img_id: u64,
    #[serde(rename(deserialize = "musicSize"))]
    music_size: u32,
    #[serde(rename(deserialize = "albumSize"))]
    album_size: u32,
    #[serde(rename(deserialize = "accountId"))]
    account_id: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct AlbumSong {
    pub id: u64,
    #[serde(rename(deserialize = "name"))]
    pub title: String,
    pub artists: Vec<ArtistSummary>,
    pub album: Album,
}

#[derive(Debug, Deserialize)]
pub struct Album {
    pub id: u64,
    pub name: String,
    #[serde(rename(deserialize = "picUrl"))]
    pub img_url: String,
    pub songs: Option<Vec<AlbumSong>>,
}

#[derive(Debug, Deserialize)]
pub struct Song {
    pub id: u64,
    #[serde(rename(deserialize = "name"))]
    pub title: String,
    #[serde(rename(deserialize = "ar"))]
    pub artists: Vec<ArtistSummary>,
    #[serde(rename(deserialize = "al"))]
    pub album: Album,
}

#[derive(Debug, Deserialize)]
pub struct HotSong {
    pub id: u64,
    #[serde(rename(deserialize = "name"))]
    pub title: String,
    pub artists: Vec<Artist>,
    pub album: Album,
}

#[derive(Debug, Deserialize)]
pub struct NeteaseSongsResponse {
    pub songs: Vec<Song>,
}

#[derive(Debug, Deserialize)]
pub struct NeteaseAlbumResponse {
    pub code: i32,
    pub album: Option<Album>,
}

#[derive(Deserialize, Debug)]
pub struct Lyrics {
    version: u32,
    lyric: String,
}

#[derive(Deserialize, Debug)]
pub struct LyricResponse {
    code: i32,
    lrc: Lyrics,
    tlyric: Option<Lyrics>,
    sgc: bool,
    sfy: bool,
    qfy: bool,
    #[serde(rename(deserialize = "briefDesc"))]
    brief_description: Option<String>,
    #[serde(rename(deserialize = "pureMusic"))]
    pure_music: bool,
    #[serde(rename(deserialize = "needDesc"))]
    need_desc: bool,
}

#[derive(Deserialize, Debug)]
pub struct ArtistResponse {
    code: i32,
    artist: Artist,
    #[serde(rename(deserialize = "hotSongs"))]
    hot_songs: Vec<HotSong>,
}

pub struct Netease {
    pub client: Client,
}

const SECRET_CHARS: &'static [u8; 15] = b"012345679abcdef";
const MODULUS: &'static [u8; 258] = b"00e0b509f6259df8642dbc35662901477df22677ec152b5ff68ace615bb7b725152b3ab17a876aea8a5aa76d2e417629ec4ee341f56135fccf695280104e0312ecbda92557c93870114af6c9d05c4f7f0c3685b7a46bee255932575cce10b424d813cfe4875d3e82047b97ddef52741d546b8e289dc6935b3ece0462db0a22b8e7";

impl Netease {
    const HOST: &'static str = "https://music.163.com";
    const IV: &'static [u8; 16] = b"0102030405060708";
    const PUBLIC_KEY: &'static [u8; 6] = b"010001";

    const PLAYLIST_DETAIL_URL: &'static str = "https://music.163.com/weapi/v3/playlist/detail";

    fn playlist_url(params: &NeteasePlaylistParams) -> String {
        let text = json!(params).to_string();
        let data = Netease::encrypt_we_api_data(text.as_bytes());
        format!(
            "{}/weapi/v3/playlist/detail?{}",
            Netease::HOST,
            to_query_string(&data)
        )
    }

    fn playlist_song_url(params: &SongParams) -> String {
        let params = json!(params).to_string();
        let data = Netease::encrypt_we_api_data(params.as_bytes());

        format!(
            "{}/weapi/v3/song/detail?{}",
            Netease::HOST,
            to_query_string(&data)
        )
    }

    fn lyric_url(&self, song_id: u64) -> String {
        let mut params = LyricParams::default();
        params.id = song_id;
        let params = json!(params).to_string();
        let data = Netease::encrypt_we_api_data(params.as_bytes());

        format!(
            "{}/weapi/song/lyric?{}",
            Netease::HOST,
            to_query_string(&data)
        )
    }

    fn artist_url(artist_id: u64) -> String {
        format!("{}/api/artist/{artist_id}", Netease::HOST)
    }

    fn album_url(album_id: u64) -> String {
        format!("{}/api/album/{album_id}", Netease::HOST)
    }

    fn playlists_url(params: NeteasePlaylistsParams) -> String {
        if params.offset.is_none() {
            let category_id = params.category_id;
            return format!(
                "{}/discover/playlist?order={}&cat={}",
                Netease::HOST,
                params.order,
                category_id
            );
        }

        format!(
            "{}/discover/playlist?{}",
            Netease::HOST,
            to_query_string(&params)
        )
    }

    fn top_playlists_url(form: &NeteaseFormData) -> String {
        format!(
            "{}/weapi/toplist/detail?{}",
            Netease::HOST,
            to_query_string(form)
        )
    }

    fn create_secret_key<const N: usize>(size: u8) -> [u8; N] {
        let mut rng = rand::thread_rng();
        let mut result = vec![];
        let range = 0..SECRET_CHARS.len();

        for _i in 0..size {
            let index = rng.gen_range(range.clone());
            let char = SECRET_CHARS[index];
            result.push(char.clone())
        }

        result.try_into().unwrap_or_else(|v: Vec<u8>| {
            panic!("Expected a Vec of length {} but it was {}", N, v.len())
        })
    }

    fn encrypt_rsa(text: &str, public_key: &[u8], modulus: &[u8]) -> String {
        let n = BigInt::parse_bytes(modulus, 16).unwrap();
        let e = BigInt::parse_bytes(public_key, 16).unwrap();
        let text = reverse_acsii_str(text);
        let text = to_hex(&text);

        let b = BigInt::parse_bytes(text.as_bytes(), 16).unwrap();

        let encrypted = b.modpow(&e, &n);

        format!("{:0>256}", encrypted.to_str_radix(16))
    }

    fn encrypt_we_api_data(data: &[u8]) -> NeteaseFormData {
        let nonce = b"0CoJUm6Qyw8W8jud";

        let cipher = Cipher::new_128(nonce);
        let encrypted = cipher.cbc_encrypt(Netease::IV, data);
        let text = general_purpose::STANDARD.encode(encrypted);
        let bytes = text.as_bytes();

        let secret_key = Self::create_secret_key::<16>(16);
        // let secret_key = b"6fe1baacb9a0a6fa";
        let cipher = Cipher::new_128(&secret_key);
        let encrypted = cipher.cbc_encrypt(Netease::IV, bytes);
        let params = general_purpose::STANDARD.encode(encrypted);

        let text = String::from_utf8(Vec::from(secret_key)).unwrap();
        let security_key = Self::encrypt_rsa(&text, Netease::PUBLIC_KEY, MODULUS);

        NeteaseFormData {
            params,
            enc_sec_key: security_key.clone(),
        }
    }

    pub fn create_cookie_jar() -> cookie::Jar {
        let data = Vec::from(Netease::create_secret_key::<32>(32));
        let uid = String::from_utf8(data).unwrap();
        let timestamp = get_time();
        let nid = format!("{uid},{timestamp}");

        let expire_at = (timestamp + 1000 * 60 * 60 * 24 * 365 * 100) / 1000;
        println!("nid is {:?}", nid);

        let url = Netease::HOST.parse::<Url>().unwrap();
        let uid_cookie = format!(
            "_ntes_nuid={uid}; expires={expire_at}; Domain={}",
            url.domain().unwrap()
        );
        let nid_cookie = format!(
            "_ntes_nnid={uid}; expires={expire_at}; Domain={}",
            url.domain().unwrap()
        );

        let jar = cookie::Jar::default();
        jar.add_cookie_str(&uid_cookie, &url);
        jar.add_cookie_str(&nid_cookie, &url);

        jar
    }

    pub fn create_client() -> Client {
        let mut headers = header::HeaderMap::new();
        let refer = Netease::HOST;
        headers.insert("Referer", header::HeaderValue::from_static(refer));
        headers.insert("Origin", header::HeaderValue::from_static("music.163.com"));

        // let user_agent = fake_useragent::UserAgents::new();
        let jar = Netease::create_cookie_jar();
        // let new_agent = user_agent.random();
        // println!("agent: {new_agent}");
        Client::builder()
            .default_headers(headers)
            .cookie_provider(Arc::new(jar))
            .user_agent("Mozilla/5.0 (iPhone; CPU iPhone OS 14_3 like Mac OS X) AppleWebKit/534.30 (KHTML, like Gecko) Version/4.0 Mobile Safari/534.30")
            .build()
            .unwrap()
    }

    fn parse_playlist(node_ref: &NodeRef) -> NeteasePlaylist {
        let cover_node = node_ref.select_first("img").unwrap();
        let cover_url = cover_node
            .attributes
            .borrow()
            .get("src")
            .unwrap()
            .replace("140y140", "512y512");

        let title_container = node_ref.select_first("div").unwrap();
        let anchor_el = title_container.as_node().select_first("a").unwrap();
        let attrs = anchor_el.attributes.borrow();
        let title = attrs.get("title").unwrap().to_string();
        let href = attrs.get("href").unwrap();

        let pattern = Regex::new(r"^/playlist\?id=(?P<playlist_id>\d+)$").unwrap();
        let caps = pattern.captures(href).unwrap();
        let playlist_id = &caps["playlist_id"];
        let source_url = format!("{}/#/playlist?id={playlist_id}", Netease::HOST);
        let playlist = NeteasePlaylist {
            id: playlist_id.parse().unwrap(),
            cover_img_url: cover_url,
            source_url,
            title,
        };

        playlist
    }

    pub async fn get_playlist_detail(&self, params: NeteasePlaylistParams) -> PlaylistResponse {
        let url = Netease::playlist_url(&params);
        let response = self
            .client
            .post(url)
            .send()
            .await
            .unwrap()
            .json::<PlaylistResponse>()
            .await
            .unwrap();

        return response;
    }

    pub async fn get_album(&self, id: u64) -> NeteaseAlbumResponse {
        let url = Netease::album_url(id);
        let resp = self
            .client
            .get(url)
            .send()
            .await
            .unwrap()
            .json::<NeteaseAlbumResponse>()
            .await
            .unwrap();

        resp
    }

    pub async fn get_album_with_songs(&self, id: u64) {
        let album = self.get_album(id).await;
        // let ids = album.album.id;
        // let songs = self.get_playlist_songs(vec![ids]).await;
        // (album, songs)
    }

    pub async fn get_song_lyric(&self, song_id: u64) -> LyricResponse {
        let url = self.lyric_url(song_id);
        let resp = self
            .client
            .post(url)
            .send()
            .await
            .unwrap()
            .json::<LyricResponse>()
            .await
            .unwrap();

        resp
    }

    pub async fn get_artist(&self, artist_id: u64) -> ArtistResponse {
        let url = Netease::artist_url(artist_id);
        let resp = self
            .client
            .get(url)
            .send()
            .await
            .unwrap()
            .json::<ArtistResponse>()
            .await
            .unwrap();

        resp
    }

    pub async fn get_playlist_songs(&self, ids: Vec<u64>) -> NeteaseSongsResponse {
        let params = SongParams::new(ids);
        let url = Netease::playlist_song_url(&params);

        let resp = self
            .client
            .post(url)
            .send()
            .await
            .unwrap()
            .json::<NeteaseSongsResponse>()
            .await
            .unwrap();

        resp
    }

    pub async fn get_playlists(&self, params: NeteasePlaylistsParams) -> Vec<NeteasePlaylist> {
        let url = Netease::playlists_url(params);
        let resp = self
            .client
            .get(url)
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        let document = parse_html().one(resp);
        let list_element = document.select_first(".m-cvrlst").unwrap();
        let mut playlists: Vec<NeteasePlaylist> = Vec::new();
        for data in list_element.as_node().select("li").unwrap() {
            let playlist = Netease::parse_playlist(&data.as_node());
            playlists.push(playlist);
        }

        playlists
    }

    pub async fn get_top_playlists(&self) -> NeteaseTopPlaylistsResponse {
        let data = json!({}).to_string();
        let form = Netease::encrypt_we_api_data(data.as_bytes());
        let url = Netease::top_playlists_url(&form);

        // let resp = self
        //     .client
        //     .post(url)
        //     .send()
        //     .await
        //     .unwrap()
        //     .text()
        //     .await
        //     .unwrap();

        let resp = self
            .client
            .post(url)
            .send()
            .await
            .unwrap()
            .json::<NeteaseTopPlaylistsResponse>()
            .await
            .unwrap();

        resp
    }
}

impl Default for Netease {
    fn default() -> Self {
        Self {
            client: Netease::create_client(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_secret_key() {
        let secret_key = Netease::create_secret_key::<16>(16);
        let secret_key = String::from_utf8(Vec::from(secret_key)).unwrap();
        assert_eq!(secret_key.len(), 16);
    }

    #[test]
    fn test_aes_cbc() {
        let form = Netease::encrypt_we_api_data(b"Hello, world!");
        assert_ne!(form.params.len(), 0);
        assert_eq!(form.enc_sec_key.len(), 256);
    }

    #[test]
    fn test_encrypt_rsa() {
        let text = "6fe1baacb9a0a6fa";
        let encrypted = Netease::encrypt_rsa(text, Netease::PUBLIC_KEY, MODULUS);
        assert_eq!(encrypted, "6c7a1a02e9e5701ecbfd9658c8c0ae1419caf2bc30f7b1cb0218868a3aee5c0ead4dadf5bdb9984915c7d01966bda228e3e8621f85001d9fbe249988ff561a4d1d63feba2200e8fc3b22cc75bdf02cbf1f200ca303b3e115652a54f853d7346b582b0a743ef8316faf30d1c48f328533e571506debb90e22da53e7acd591e5d9")
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_get_playlists() {
        let netease = Netease::default();
        let params = NeteasePlaylistsParams {
            category_id: "".to_string(),
            order: "hot".to_string(),
            limit: 10,
            offset: Some(0),
        };
        let playlists = netease.get_playlists(params).await;
        assert_eq!(playlists.len(), 10);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_get_playlist_detail() {
        let netease = Netease::default();
        let mut params = NeteasePlaylistParams::default();
        params.id = 26467411;
        let resp = netease.get_playlist_detail(params).await;
        assert_ne!(resp.playlist.id, 0);
        assert_ne!(resp.playlist.description.len(), 0);
        assert_ne!(resp.playlist.track_ids.len(), 0);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_get_artist() {
        let netease = Netease::default();
        let artist_id = 12094419;
        let resp = netease.get_artist(artist_id).await;
        assert_eq!(resp.code, 200);
        assert_ne!(resp.artist.name.len(), 0);
        assert_ne!(resp.hot_songs.len(), 0);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_get_playlist_songs() {
        let netease = Netease::default();
        let ids = vec![430685732, 22707008, 16846091];
        let resp = netease.get_playlist_songs(ids).await;
        assert_eq!(resp.songs.len(), 3);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_get_album() {
        let netease = Netease::default();
        let album_id = 1552283;
        let resp = netease.get_album(album_id).await;
        println!("resp: {resp:?}");
        assert_eq!(resp.code, 200);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_get_song_lyric() {
        let netease = Netease::default();
        let song_id = 22707008;
        let resp = netease.get_song_lyric(song_id).await;
        assert_eq!(resp.code, 200);
        assert_eq!(resp.lrc.version, 1);
        assert_ne!(resp.lrc.lyric.len(), 0);
        assert_ne!(resp.brief_description.unwrap().len(), 0);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_get_top_playlists() {
        let netease = Netease::default();
        let resp = netease.get_top_playlists().await;
        assert_eq!(resp.code, 200);
        assert_ne!(resp.list.len(), 0);
    }
}
