pub mod playlist;

use super::{L1PlaylistParams, MusicService};
use crate::provider::kugou::playlist::{ExtraSongInfo, KugouPlaylistSong,PlaylistInfo, PlaylistList, PlaylistsList, SongAuthor};
use async_trait::async_trait;
use reqwest::{Client, header};
use std::string::ToString;
use serde::{Deserialize, Serialize};
use std::slice::Iter;


#[derive(Serialize)]
pub struct KuGouPlaylistParams {
    pub page: u64,
    pub json: bool,
}

#[derive(Debug, Deserialize)]
pub struct KuGouAlbum {
    pub albumid: u64,
    pub singerid: u64,
    pub songcount: u8,
    pub category: u8,
    pub singername: String,
    pub publishtime: String,
    pub albumname: Option<String>,
    pub imgurl: String,
}

#[derive(Debug, Deserialize)]
pub struct KuGouAlbumResponse {
    pub data: KuGouAlbum
}

impl From<&L1PlaylistParams> for KuGouPlaylistParams {
    fn from(value: &L1PlaylistParams) -> Self {
        let page = value.offset.unwrap_or(0) / 30 + 1;
        Self { page, json: true }
    }
}

impl KuGouPlaylistParams {
    pub fn build_url(&self) -> String {
        let params = serde_qs::to_string(&self).unwrap();
        format!("http://m.kugou.com/plist/index?{}", params)
    }
}

#[derive(Debug, Deserialize)]
pub struct KugouPlaylistsResponse {
    pub src: String,
    pub ver: String,
    pub kg_domain: String,
    pub plist: PlaylistsList,
    JS_CSS_DATE: u64,
    #[serde(rename(deserialize = "pagesize"))]
    pub page_size: usize,
}

#[derive(Debug, Deserialize)]
pub struct PlaylistResponse {
    #[serde(rename(deserialize = "pagesize"))]
    pub page_size: usize,
    pub src: String,
    pub list: PlaylistList,
    pub info: PlaylistInfo,
    pub ver: String,
    pub kg_domain: String,
    JS_CSS_DATE: u64,
}

impl PlaylistResponse {
    pub(crate) fn get_songs(&self) -> Iter<'_, KugouPlaylistSong> {
        self.list.list.info.iter()
    }

    pub(crate) fn get_song_hashes(&self) -> Vec<String> {
        self.list.list.info.iter().map(|x| x.hash.clone()).collect()
    }
}

#[derive(Debug, Deserialize)]
pub struct KugouPlaylistSongResponse {
    #[serde(rename(deserialize = "req_hash"))]
    pub hash: String,
    pub albumid: u64,
    #[serde(rename(deserialize = "songName"))]
    pub title: String,
    #[serde(rename(deserialize = "singerId"))]
    pub singer_id: u64,
    #[serde(rename(deserialize = "singerName"))]
    pub singer_name: String,
    pub authors: Vec<SongAuthor>,
    pub audio_id: u64,
    pub album_audio_id: u64,
    pub audio_group_id: u64,
    pub ctype: u32,
    pub stype: u32,
    pub album_category: u32,
    #[serde(rename(deserialize = "imgUrl"))]
    pub img_url: String,
    pub album_img: Option<String>,
    extra: ExtraSongInfo,
}



#[derive(Debug, Deserialize)]
pub struct KugouArtist {
    #[serde(rename(deserialize = "imgUrl"))]
    pub img_url: String,
    #[serde(rename(deserialize = "singername"))]
    pub singer_name: String,
}

#[derive(Debug, Deserialize)]
pub struct KugouArtistData {
    pub data: KugouArtist,
}

#[derive(Debug, Deserialize)]
pub struct KugouArtistResponse {
    data: KugouArtistData
}

#[derive(Debug, Deserialize)]
pub struct KugouArtistSongItem {
    pub hash: String,
    pub album_id: u64,
}

#[derive(Debug, Deserialize)]
pub struct KugouArtistSongsData {
    data: Vec<KugouArtistSongItem>
}

#[derive(Debug, Deserialize)]
pub struct KugouArtistSongsResponse {
    data: KugouArtistSongsData
}

#[derive(Debug, Deserialize)]
pub struct KugouAlbumSongItem {
    pub hash: String,
    pub bitrate: u32,
    pub album_id: String,
    pub filename: String,
    pub duration: u32,
    pub extname: String,
    audio_id: u64,
    album_audio_id: u64,
}

#[derive(Debug, Deserialize)]
pub struct KugouAlbumSongsData {
    total: u32,
    timestamp: u64,
    info: Vec<KugouAlbumSongItem>,
}

#[derive(Debug, Deserialize)]
pub struct KugouAlbumSongsResponse {
    data: KugouAlbumSongsData,
    errcode: u32,
    status: u8,
    error: String,
}

pub struct Kugou {
    pub client: Client,
}

pub fn create_client() -> Client {
    let mut headers = header::HeaderMap::new();
    headers.insert(
        "Referer",
        header::HeaderValue::from_static("https://www.kugou.com/"),
    );
    headers.insert(
        "Origin",
        header::HeaderValue::from_static("https://www.kugou.com/"),
    );

    Client::builder()
        .default_headers(headers)
        .user_agent("Mozilla/5.0 (iPhone; CPU iPhone OS 14_3 like Mac OS X) AppleWebKit/534.30 (KHTML, like Gecko) Version/4.0 Mobile Safari/534.30")
        .connection_verbose(true)
        .build()
        .unwrap()
}

impl Kugou {
    pub(crate) const HOST: &'static str = "https://www.kugou.com";
    const H5_HOST: &'static str = "https://m.kugou.com";
    const MOBILE_CDN_HOST: &'static str = "http://mobilecdnbj.kugou.com";

    pub fn new() -> Kugou {
        Self {
            client: create_client(),
        }
    }
    pub fn artist_url(id: u64) -> String {
        format!("{}/api/v3/singer/info?singer_id={id}", Kugou::MOBILE_CDN_HOST)
    }
    pub fn artist_songs_url(id: u64) -> String {
        format!("{}/api/v3/singer/song?singer_id={id}&page=1&pagesize=30", Kugou::MOBILE_CDN_HOST)
    }
    pub fn album_songs_url(id: u64) -> String {
        format!("{}/api/v3/album/song?albumid={id}&page=1&pagesize=-1", Kugou::MOBILE_CDN_HOST)
    }
    pub fn get_playlist_url(id: u64) -> String {
        let source_url = format!("{}/yy/special/single/{{size}}.html", Kugou::HOST);
        source_url.replace("{size}", &*id.to_string())
    }
    fn playlist_song_url(hash: &str) -> String {
        format!(
            "{}/app/i/getSongInfo.php?cmd=playInfo&hash={}",
            Kugou::H5_HOST,
            hash
        )
    }
    pub fn album_url(album_id: u64) -> String {
        format!("{}/api/v3/album/info?albumid={album_id}", Kugou::MOBILE_CDN_HOST)
    }

    // no special client
    pub async fn get_playlists(&self, params: &KuGouPlaylistParams) -> KugouPlaylistsResponse {
        let url = params.build_url();

        let resp = self
            .client
            .get(url)
            .send()
            .await
            .unwrap()
            .json::<KugouPlaylistsResponse>()
            .await
            .unwrap();

        resp
    }

    // no special client
    pub async fn get_playlist(&self, id: u64) -> PlaylistResponse {
        let url = format!("{}/plist/list/{}?json=true", Kugou::H5_HOST, id);

        let resp = self
            .client
            .get(url)
            .send()
            .await
            .unwrap()
            .json::<PlaylistResponse>()
            .await
            .unwrap();

        resp
    }

    pub async fn get_album(&self, album_id: u64) -> KuGouAlbumResponse {
        let url = Kugou::album_url(album_id);
        let resp = self.client.get(url)
            .send()
            .await
            .unwrap()
            .json::<KuGouAlbumResponse>()
            .await
            .unwrap();

        resp
    }

    pub async fn get_playlist_song(&self, hash: &str) -> KugouPlaylistSongResponse {
        let url = Self::playlist_song_url(hash);

        self.client
            .get(url)
            .send()
            .await
            .unwrap()
            .json::<KugouPlaylistSongResponse>()
            .await
            .unwrap()
    }

    pub async fn get_album_songs(&self, id: u64) -> KugouAlbumSongsResponse {
        let url = Self::album_songs_url(id);

        let resp = self.client
            .get(url)
            .send()
            .await
            .unwrap()
            .json::<KugouAlbumSongsResponse>()
            .await
            .unwrap();

        resp
    }

    pub async fn get_artist(&self, id: u64) {
        let url = Self::artist_url(id);
        println!("get_artist {url}");

        // self.client
        //     .get(url)
        //     .send()
        //     .await
        //     .unwrap()
        //     .json::<KugouArtistData>()
        //     .await
        //     .unwrap()

        println!("{:?}", self.client
            .get(url)
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap());
    }

    pub async fn get_artist_songs(&self, id: u64) {
        let url = Self::artist_url(id);

        let resp = self.client
            .get(url)
            .send()
            .await
            .unwrap()
            .json::<KugouArtistResponse>()
            .await
            .unwrap();

        // self.client
        //     .get(url)
        //     .send()
        //     .await
        //     .unwrap()
        //     .json::<KugouArtistResponse>()
        //     .await
        //     .unwrap()

        println!("resp: {resp:?}")
    }
}

#[async_trait]
impl MusicService for Kugou {
    // fn provider(&self) -> Provider {
    //     super::Provider::Kugou
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn it_get_playlists() {
        let provider = Kugou::new();
        let params = KuGouPlaylistParams {
            page: 1,
            json: true,
        };
        let resp = provider.get_playlists(&params).await;
        assert_eq!(resp.plist.list.total, 600);
        assert_eq!(resp.plist.list.info.len(), resp.page_size);
    }
}
