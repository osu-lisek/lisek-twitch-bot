use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{Pool, Postgres};
use tracing::error;




#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OnlineBeatmapset {
    pub artist: String,
    #[serde(rename = "artist_unicode")]
    pub artist_unicode: String,
    pub covers: Covers,
    pub creator: String,
    #[serde(rename = "favourite_count")]
    pub favourite_count: i64,
    pub hype: Option<Hype>,
    pub id: i64,
    pub nsfw: bool,
    pub offset: i64,
    #[serde(rename = "play_count")]
    pub play_count: i64,
    #[serde(rename = "preview_url")]
    pub preview_url: String,
    pub source: String,
    pub spotlight: bool,
    pub status: String,
    pub title: String,
    #[serde(rename = "title_unicode")]
    pub title_unicode: String,
    #[serde(rename = "track_id")]
    pub track_id: Value,
    #[serde(rename = "user_id")]
    pub user_id: i64,
    pub video: bool,
    pub bpm: f64,
    #[serde(rename = "can_be_hyped")]
    pub can_be_hyped: bool,
    #[serde(rename = "deleted_at")]
    pub deleted_at: Value,
    #[serde(rename = "discussion_enabled")]
    pub discussion_enabled: bool,
    #[serde(rename = "discussion_locked")]
    pub discussion_locked: bool,
    #[serde(rename = "is_scoreable")]
    pub is_scoreable: bool,
    #[serde(rename = "last_updated")]
    pub last_updated: String,
    #[serde(rename = "legacy_thread_url")]
    pub legacy_thread_url: String,
    #[serde(rename = "nominations_summary")]
    pub nominations_summary: Option<NominationsSummary>,
    pub ranked: i64,
    #[serde(rename = "ranked_date")]
    pub ranked_date: Value,
    pub storyboard: bool,
    #[serde(rename = "submitted_date")]
    pub submitted_date: String,
    pub tags: String,
    pub availability: Availability,
    #[serde(rename = "has_favourited")]
    pub has_favourited: bool,
    pub beatmaps: Vec<OnlineBeatmap>,
    #[serde(rename = "pack_tags")]
    pub pack_tags: Vec<Value>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Covers {
    pub cover: String,
    #[serde(rename = "cover@2x")]
    pub cover_2x: String,
    pub card: String,
    #[serde(rename = "card@2x")]
    pub card_2x: String,
    pub list: String,
    #[serde(rename = "list@2x")]
    pub list_2x: String,
    pub slimcover: String,
    #[serde(rename = "slimcover@2x")]
    pub slimcover_2x: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Hype {
    pub current: i64,
    pub required: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NominationsSummary {
    pub current: i64,
    pub required: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Availability {
    #[serde(rename = "download_disabled")]
    pub download_disabled: bool,
    #[serde(rename = "more_information")]
    pub more_information: Value,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OnlineBeatmap {
    #[serde(rename = "beatmapset_id")]
    pub beatmapset_id: i64,
    #[serde(rename = "difficulty_rating")]
    pub difficulty_rating: f64,
    pub id: i64,
    pub mode: String,
    pub status: String,
    #[serde(rename = "total_length")]
    pub total_length: i32,
    #[serde(rename = "user_id")]
    pub user_id: i64,
    pub version: String,
    pub accuracy: f64,
    pub ar: f64,
    pub bpm: f32,
    pub convert: bool,
    #[serde(rename = "count_circles")]
    pub count_circles: i64,
    #[serde(rename = "count_sliders")]
    pub count_sliders: i64,
    #[serde(rename = "count_spinners")]
    pub count_spinners: i64,
    pub cs: f64,
    #[serde(rename = "deleted_at")]
    pub deleted_at: Value,
    pub drain: f64,
    #[serde(rename = "hit_length")]
    pub hit_length: i64,
    #[serde(rename = "is_scoreable")]
    pub is_scoreable: bool,
    #[serde(rename = "last_updated")]
    pub last_updated: String,
    #[serde(rename = "mode_int")]
    pub mode_int: i64,
    pub passcount: i64,
    pub playcount: i64,
    pub ranked: i64,
    pub url: String,
    pub checksum: String,
    #[serde(rename = "max_combo")]
    pub max_combo: i64,
}


fn humanize_seconds(seconds: i32) -> String {
    let minutes = seconds / 60;
    let seconds = seconds % 60;
    format!("{}:{}", minutes, seconds)
}

impl OnlineBeatmapset {
    pub fn format(&self, difficulty_id: i64) -> String {
        let beatmap = self.beatmaps.iter().find(|&x| x.id == difficulty_id);

        if let None = beatmap {
            return String::new();
        }

        let beatmap = beatmap.unwrap();
        
        format!("{} - {} [{}] by \"{}\" | {:.2}⭐ | {:.1} AR / {:.1} OD / {:.1} HP / {:.0} BPM | {}", self.artist, self.title, beatmap.version, self.creator, beatmap.difficulty_rating, beatmap.ar, beatmap.accuracy, beatmap.drain, beatmap.bpm, humanize_seconds(beatmap.total_length)).to_string()
    }

    pub fn format_osu(&self, from: String, difficulty_id: i64) -> String {
        let beatmap = self.beatmaps.iter().find(|&x| x.id == difficulty_id);

        if let None = beatmap {
            return String::new();
        }

        let beatmap = beatmap.unwrap();
        
        format!("{}: [osu://b/{} {} - {} [{}] by {}] {:.2}⭐/ {:.1} AR / {:.1} OD / {:.1} HP / {:.0} BPM", from, beatmap.id, self.artist, self.title, beatmap.version, self.creator, beatmap.difficulty_rating, beatmap.ar, beatmap.accuracy, beatmap.drain, beatmap.bpm)
    }

    pub async fn fetch_from_db_by_id(_connection: &Pool<Postgres>, id: i64) -> Option<OnlineBeatmapset> {
        let request = reqwest::get(format!("https://mirror.lisek.cc/api/v1/beatmapsets/beatmap/{}", id)).await;

        if let Err(error) = request {
            error!("Failed to fetch beatmap: {}", error);
            return None;
        }

        let request = request.unwrap();

        let data = request.json::<OnlineBeatmapset>().await;

        if let Err(error) = data {
            error!("Failed to deserialize beatmap: {}", error);
            return None
        }

        Some(data.unwrap())
    }
        
}