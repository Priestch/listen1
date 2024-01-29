use std::ops::Deref;
use async_trait::async_trait;
use futures::future::join_all;
use kugou::{Kugou, KugouPlaylistSongResponse, KugouPlaylistsResponse};
use kugou::playlist::KugouPlaylistDetail;
use crate::provider::kugou::playlist::KuGouPlaylistInfo;
use crate::provider::netease::{Netease, NeteasePlaylist, NeteasePlaylistsParams};

pub mod kugou;
pub mod kuwo;
pub mod netease;
mod utils;

#[derive(Debug)]
pub struct L1Track {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub artist_id: String,
    pub album: String,
    pub album_id: String,
    pub source: String,
    pub source_url: String,
    pub img_url: String,
    pub lyric_url: String,
}

impl L1Track {
    fn id(hash: &str) -> String {
        format!("kgtrack_{hash}")
    }
    fn album_id(id: u64) -> String {
        format!("kgalbum_{id}")
    }
    fn artist_id(singer_id: u64) -> String {
        format!("kgartist_{singer_id}")
    }
    fn get_source_url(hash: &str, album_id: u64) -> String {
        format!("{}/song/#hash={hash}&album_id={album_id}", Kugou::HOST)
    }
}

impl Default for L1Track {
    fn default() -> Self {
        L1Track {
            id: "".to_string(),
            title: "".to_string(),
            artist: "".to_string(),
            artist_id: "".to_string(),
            album: "".to_string(),
            album_id: "".to_string(),
            source: "kugou".to_string(),
            source_url: "".to_string(),
            img_url: "".to_string(),
            lyric_url: "".to_string(),
        }
    }
}

impl From<&KugouPlaylistSongResponse> for L1Track {
    fn from(value: &KugouPlaylistSongResponse) -> Self {
        let mut track = L1Track::default();
        track.id = L1Track::id(&value.hash);
        track.album_id = L1Track::album_id(value.albumid);
        track.source_url = L1Track::get_source_url(&value.hash, value.albumid);
        track.title = value.title.clone();
        track.artist = if value.singer_id == 0 {
            "未知".to_string()
        } else {
            value.singer_name.clone()
        };
        track.artist_id = L1Track::artist_id(value.singer_id);
        match &value.album_img {
            None => {}
            Some(url) => {
                track.img_url = url.clone();
            }
        }
        track
    }
}

#[derive(Debug)]
pub struct L1PlaylistInfo {
    pub id: String,
    pub cover_img_url: String,
    pub source_url: String,
    pub title: String,
}

#[derive(Debug)]
pub struct L1Playlist {
    pub info: L1PlaylistInfo,
    pub tracks: Vec<L1Track>,
}

impl From<&KugouPlaylistDetail> for L1Playlist {
    fn from(value: &KugouPlaylistDetail) -> Self {
        let cover_url = if value.image_url.len() > 0 {
            value.image_url.replace("{size}", "400")
        } else {
            "".to_string()
        };
        let info = L1PlaylistInfo {
            id: L1Playlist::get_playlist_id(value.id, "kugou"),
            cover_img_url: cover_url,
            source_url: Kugou::get_playlist_url(value.id),
            title: value.title.clone(),
        };
        L1Playlist {
            info,
            tracks: vec![],
        }
    }
}

impl From<&NeteasePlaylist> for L1Playlist {
    fn from(value: &NeteasePlaylist) -> Self {
        L1Playlist {
            info: L1PlaylistInfo {
                id: L1Playlist::get_playlist_id(value.id, "netease"),
                cover_img_url: value.cover_img_url.clone(),
                source_url: value.source_url.clone(),
                title: value.title.clone(),
            },
            tracks: vec![],
        }
    }
}

impl From<&KuGouPlaylistInfo> for L1Playlist {
    fn from(value: &KuGouPlaylistInfo) -> Self {
        let cover_url = if value.image_url.len() > 0 {
            value.image_url.replace("{size}", "400")
        } else {
            "".to_string()
        };
        let info = L1PlaylistInfo {
            id: L1Playlist::get_playlist_id(value.id, "kugou"),
            cover_img_url: cover_url,
            source_url: Kugou::get_playlist_url(value.id),
            title: value.title.clone(),
        };
        L1Playlist {
            info,
            tracks: vec![],
        }
    }
}

impl L1Playlist {
    fn get_playlist_id(specialid: u64, provider: &str) -> String {
        match provider {
            "kugou" => format!("kgplaylist_{}", specialid).to_string(),
            "netease" => format!("neplaylist_{}", specialid).to_string(),
            _ => format!("kgplaylist_{}", specialid).to_string()
        }
    }
}

#[derive(Debug)]
pub struct L1Playlists {
    pub page: u64,
    pub page_size: usize,
    pub items: Vec<L1Playlist>,
    pub total: u64,
}

impl From<KugouPlaylistsResponse> for L1Playlists {
    fn from(value: KugouPlaylistsResponse) -> Self {
        let items = value.plist.list.info.iter().map(|item| {
            L1Playlist::from(item)
        }).collect();
        L1Playlists {
            page: 1,
            page_size: value.page_size,
            total: value.plist.list.total,
            items,
        }
    }
}


pub struct L1PlaylistParams {
    pub filter_id: Option<String>,
    pub offset: Option<u64>,
}

#[async_trait]
pub trait MusicService {
    // fn provider(&self) -> Provider;
}


pub struct Listen1 {}

impl Listen1 {
    pub async fn get_playlists(name: &str, params: &L1PlaylistParams) -> L1Playlists {
        match name {
            "kugou" => {
                let kugou = Kugou::new();
                let kg_params = kugou::KuGouPlaylistParams::from(params);
                let kg_response = kugou.get_playlists(&kg_params).await;
                let mut resp = L1Playlists::from(kg_response);
                resp.page = kg_params.page;
                resp
            }
            "netease" => {
                let netease = Netease::default();
                let params = NeteasePlaylistsParams {
                    category_id: "".to_string(),
                    order: "hot".to_string(),
                    limit: 35,
                    offset: Some(0),
                };
                let playlists = netease.get_playlists(params).await;
                let items = playlists.iter().map(|x| {
                    L1Playlist::from(x)
                }).collect();
                L1Playlists {
                    page: 0,
                    page_size: 0,
                    items,
                    total: 0,
                }
            }
            _ => {
                let kugou = Kugou::new();
                let kg_params = kugou::KuGouPlaylistParams::from(params);
                let kg_response = kugou.get_playlists(&kg_params).await;
                let mut resp = L1Playlists::from(kg_response);
                resp.page = kg_params.page;
                resp
            }
        }
    }

    async fn get_kugou_playlist(id: u64) -> L1Playlist {
        let kugou = Kugou::new();
        let resp = kugou.get_playlist(id).await;
        let hashes = resp.get_song_hashes();
        let futures = hashes.iter()
            .map(|hash| async {
                let song = kugou.get_playlist_song(hash).await;
                let album = kugou.get_album(song.albumid).await;
                (song, album)
            });

        let pairs = join_all(futures).await;
        let tracks = pairs.iter().map(|(song, album)| {
            let mut track = L1Track::from(song);
            match &album.data.albumname {
                None => {}
                Some(val) => {
                    track.album = val.to_string();
                }
            }
            track
        }).collect();

        let mut playlist: L1Playlist = (&resp.info.list).into();
        playlist.tracks = tracks;

        playlist
    }

    pub async fn get_playlist_with_tracks(name: &str, id: u64) -> L1Playlist {
        match name {
            "kugou" => {
                Self::get_kugou_playlist(id).await
            }
            _ => {
                Self::get_kugou_playlist(id).await
            }
        }
    }
}
