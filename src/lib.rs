pub mod provider;

#[cfg(test)]
mod tests {
    use crate::provider::netease::{Netease, NeteasePlaylistsParams};
    use super::provider::{kugou, L1PlaylistParams, Listen1};

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn it_get_kugou_playlists() {
        let params = L1PlaylistParams {
            filter_id: Option::from("".to_string()),
            offset: Some(30),
        };
        let resp = Listen1::get_playlists("kugou", &params).await;
        println!("resp: {resp:?}");
        assert_eq!(resp.items.len(), 30);
        assert_eq!(resp.total, 600);
        assert_eq!(resp.page, 2);
        assert_eq!(resp.page_size, 30);
        assert_ne!(resp.items[0].info.id, "");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn it_get_netease_playlists() {
        let params = L1PlaylistParams {
            filter_id: Option::from("".to_string()),
            offset: Some(30),
        };
        let resp = Listen1::get_playlists("netease", &params).await;
        println!("resp: {resp:?}");
        assert_eq!(resp.items.len(), 35);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn it_get_kugou_playlist() {
        let id = 4025095u64;
        let resp = Listen1::get_playlist_with_tracks("kugou", id).await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn it_get_kugou_playlist_tracks() {
        let id = 4025095u64;
        let l1_playlist = Listen1::get_playlist_with_tracks("kugou", id).await;

        println!("resp: {l1_playlist:?}");
        // assert_ne!(0, 0);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn it_get_kugou_playlist_song() {
        let hash = "8FD5DE7E1BFF24219DFF7700E7B4A0EB";
        let kugou = kugou::Kugou::new();
        let resp = kugou.get_playlist_song(hash).await;
        let album_resp = kugou.get_album(resp.albumid).await;
        let album_songs_resp = kugou.get_album_songs(resp.albumid).await;
        println!("resp {album_songs_resp:?}");
        // assert_ne!(resp.hash, hash);
    }
}