use std::time::Duration;

use itertools::Itertools;
use rand::{seq::SliceRandom, thread_rng};
use rspotify::{
    clients::{BaseClient, OAuthClient}, model::{FullTrack, PlayableItem, PlaylistId}, scopes, AuthCodeSpotify, ClientError, Credentials, OAuth
};
use clap::Parser;

#[derive(Clone)]
struct PlaylistTrack {
    pub added_by: Option<String>,
    pub track: FullTrack,
    pub index: usize,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    playlist_url: String,

    /// If specified, no changes are actually made to the playlist
    #[arg(short, long)]
    dry_run: bool,
}

fn main() {
    let args = Args::parse();

    let mut rng = thread_rng();

    // There must be a .env file in the root director that has the environment variables
    // RSPOTIFY_CLIENT_ID, RSPOTIFY_CLIENT_SECRET, and RSPOTIFY_REDIRECT_URI set.
    let creds = Credentials::from_env().unwrap();

    let scopes = scopes!(
        "playlist-read-collaborative",
        "playlist-read-private",
        "playlist-modify-public",
        "playlist-modify-private"
    );
    println!("{:#?}", std::env::var("RSPOTIFY_REDIRECT_URI"));
    let oauth = OAuth::from_env(scopes).unwrap();

    let spotify = AuthCodeSpotify::new(creds, oauth);

    let url = spotify.get_authorize_url(false).unwrap();
    spotify.prompt_for_token(&url).unwrap();

    let id = get_playlist_id_from_url(&args.playlist_url);
    println!("Playlist id: {}", id);
    let playlist_uri = format!("spotify:playlist:{}", id);
    let playlist_id = PlaylistId::from_id_or_uri(&playlist_uri).unwrap();
    println!("Getting playlist...");
    let original_playlist = spotify
        .playlist_items(playlist_id.clone(), None, None)
        .map(|i| i.unwrap())
        .enumerate()
        .filter_map(|(index, item)| {
            let added_by = item.added_by.as_ref().map(|u| u.id.to_string());
            if let Some(PlayableItem::Track(track)) = item.track {
                Some(PlaylistTrack {
                    added_by,
                    track,
                    index,
                })
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    println!("Original");
    print_tracks(&original_playlist);
    let mut playlist_mirror = original_playlist.clone();

    let mut grouped_by_user = original_playlist
        .into_iter()
        .into_group_map_by(|i| i.added_by.clone());
    for (_, user_tracks) in grouped_by_user.iter_mut() {
        user_tracks.shuffle(&mut rng);
    }

    let mut reshuffled = Vec::new();
    while !grouped_by_user.is_empty() {
        for user_tracks in grouped_by_user.values_mut() {
            let last_track = user_tracks.pop();
            if let Some(track) = last_track {
                reshuffled.push(track);
            }
        }

        grouped_by_user.retain(|_, v| !v.is_empty());
    }
    println!("Reshuffled:");
    print_tracks(&reshuffled);

    let mut last_snapshot_id: Option<String> = None;
    for (target_index, track) in reshuffled.into_iter().enumerate() {
        let current_index = playlist_mirror
            .iter()
            .enumerate()
            .find_map(|(index, item)| {
                if track.track == item.track {
                    Some(index)
                } else {
                    None
                }
            })
            .unwrap();
        println!("({}/{}) moving {current_index} to {target_index}", target_index + 1, playlist_mirror.len());
        if !args.dry_run {
            reorder_respecting_rate_limiting(&spotify, &playlist_id, current_index, target_index, &mut last_snapshot_id);
        }
        
        let el = playlist_mirror.remove(current_index);
        playlist_mirror.insert(target_index, el);
    }

    println!("Reordering Complete!");
}

fn reorder_respecting_rate_limiting(spotify: &AuthCodeSpotify, playlist_id: &PlaylistId<'_>, current_index: usize, target_index: usize, last_snapshot_id: &mut Option<String>) {
    let mut attempts = 0;
    while attempts < 10 {
        match spotify.playlist_reorder_items(
            playlist_id.clone(),
            Some(current_index.try_into().unwrap()),
            Some(target_index.try_into().unwrap()),
            Some(1),
            last_snapshot_id.as_deref(),
        ) {
            Ok(res) => {
                *last_snapshot_id = Some(res.snapshot_id.to_owned());
                break;
            },
            Err(err) => {
                let ClientError::Http(err) = err else {
                    panic!("Error reordering: {}", err);
                };

                match *err {
                    rspotify::http::HttpError::StatusCode(res) => {
                        if res.status() == 429 {
                            let retry_after = res.header("Retry-After").map(|s| s.parse::<u64>().ok()).flatten();
                            if let Some(retry_after) = retry_after {
                                println!("Getting rate limited, told to retry after {} seconds (attempt #{})", retry_after, attempts + 1);
                                std::thread::sleep(Duration::from_secs(retry_after));
                                attempts += 1;
                                continue;
                            }
                        }

                        panic!("Error reordering: {:#?}", res);
                    },
                    _ => {
                        panic!("Error reordering: {}", err);
                    }
                }
            },
        }
    }
}

fn print_tracks(original_playlist: &Vec<PlaylistTrack>) {
    for track in original_playlist.iter() {
        println!(
            "{}: {} added by {:?}",
            track.index, track.track.name, track.added_by
        )
    }
}
//https://open.spotify.com/playlist/4gnFxHWeDZveC6COR3HWnv?si=018f62b3b9bf4513
fn get_playlist_id_from_url(url: &str) -> String {
    let split = url.split(&['/', '?']).collect_vec();
    split[4].to_owned()
}
