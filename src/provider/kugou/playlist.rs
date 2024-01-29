use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct KugouPlaylistSong {
    pub hash: String,
    pub album_id: String,
    pub extname: String,
    pub filename: String,
    pub audio_id: u64,
    pub duration: u32,
}

#[derive(Debug, Deserialize)]
pub struct KugouPlaylistDetail {
    #[serde(rename(deserialize = "specialid"))]
    pub id: u64,
    #[serde(rename(deserialize = "specialname"))]
    pub title: String,
    #[serde(rename(deserialize = "imgurl"))]
    pub image_url: String,
    intro: String,
    songs: Vec<KugouPlaylistSong>,
}

#[derive(Debug, Deserialize)]
pub struct PlSongList {
    pub total: u64,
    pub timestamp: u64,
    pub info: Vec<KugouPlaylistSong>,
}

#[derive(Debug, Deserialize)]
pub struct PlaylistsList {
    pub list: KugouPlaylist,
    #[serde(rename(deserialize = "pagesize"))]
    pub page_size: usize,
}

#[derive(Debug, Deserialize)]
pub struct KugouPlaylist {
    pub total: u64,
    pub has_next: u8,
    pub timestamp: u64,
    pub info: Vec<KugouPlaylistDetail>,
}

#[derive(Debug, Deserialize)]
pub struct PlaylistList {
    pub list: PlSongList,
    pub page: usize,
    #[serde(rename(deserialize = "pagesize"))]
    pub page_size: usize,
}

#[derive(Debug, Deserialize)]
pub struct KuGouPlaylistInfo {
    #[serde(rename(deserialize = "specialid"))]
    pub id: u64,
    #[serde(rename(deserialize = "specialname"))]
    pub title: String,
    #[serde(rename(deserialize = "imgurl"))]
    pub image_url: String,
}

#[derive(Debug, Deserialize)]
pub struct PlaylistInfo {
    pub list: KuGouPlaylistInfo,
}

#[derive(Debug, Deserialize)]
pub struct SongAuthor {
    pub identity: u64,
    pub author_id: u64,
    pub country: String,
    pub language: String,
    pub is_publish: u8,
    #[serde(rename(deserialize = "type"))]
    pub kind: u8,
    pub birthday: String,
}

#[derive(Debug, Deserialize)]
pub struct ExtraSongInfo {
    sqhash: String,
    #[serde(rename(deserialize = "128hash", serialize = "128hash"))]
    hash128: String,
    #[serde(rename(deserialize = "320hash", serialize = "320hash"))]
    hash320: String,
}
