use sqlx::{prelude::FromRow, Pool, Postgres};
use tracing::error;




#[derive(FromRow, Debug)]
pub struct Beatmap {
    pub title: String,
    pub artist: String,
    pub creator: String,
    pub version: String,
    #[sqlx(rename = "beatmapId")]
    pub beatmap_id: i32,
    pub stars: f64,
    pub bpm: f64,
    pub ar: f64,
    pub od: f64,
    pub cs: f64,
    pub hp: f64,
    #[sqlx(rename = "totalLength")]
    pub total_length: i32,
    pub status: i32,
}

fn humanize_seconds(seconds: i32) -> String {
    let minutes = seconds / 60;
    let seconds = seconds % 60;
    format!("{}:{}", minutes, seconds)
}

impl Beatmap {
    pub fn format(&self) -> String {
        format!("{} - {} [{}] by \"{}\" | {:.2}⭐ | {:.1} AR / {:.1} OD / {:.1} HP / {:.0} BPM | {}", self.artist, self.title, self.version, self.creator, self.stars, self.ar, self.od, self.hp, self.bpm, humanize_seconds(self.total_length)).to_string()
    }

    pub fn format_osu(&self, from: String) -> String {
        format!("{}: [osu://b/{} {} - {} [{}] by {}] {:.2}⭐/ {:.1} AR / {:.1} OD / {:.1} HP / {:.0} BPM", from, self.beatmap_id, self.artist, self.title, self.version, self.creator, self.stars, self.ar, self.od, self.hp, self.bpm)
    }

    pub async fn fetch_from_db_by_id(connection: &Pool<Postgres>, id: i64) -> Option<Beatmap> {
        let beatmap = sqlx::query_as::<_, Beatmap>("SELECT * FROM \"Beatmap\" WHERE \"beatmapId\" = $1")
            .bind(id)
            .fetch_one(connection)
            .await;

        if let Err(error) = beatmap {
            error!("Failed to fetch beatmap: {}", error);
            return None
        }
        
        beatmap.ok()
    }
        
}