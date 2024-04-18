# Spotify Round Robin Shuffler
A CLI tool that, given a spotify playlist URL, reorders the songs in the playlist to round-robin based on who added the song.

After execution, a playlist that looks like this:

| Song | Added By |
| ---- | -------- |
| A    | User 1   |
| B    | User 1   |
| C    | User 1   |
| D    | User 2   |
| E    | User 2   |
| F    | User 2   |
| G    | User 3   |
| H    | User 3   |
| I    | User 3   |

Will end up looking something like this:
| Song | Added By |
| ---- | -------- |
| C    | User 1   |
| F    | User 2   |
| G    | User 3   |
| A    | User 1   |
| E    | User 2   |
| I    | User 3   |
| B    | User 1   |
| D    | User 2   |
| H    | User 3   |

The order of each user's songs is randomized each run.

## How to Run
You will need to populate a .env file with the environment variables RSPOTIFY_CLIENT_ID, RSPOTIFY_CLIENT_SECRET, and RSPOTIFY_REDIRECT_URI. Place this file in the folder the tool is run in.

```
cargo run -- --help
```
