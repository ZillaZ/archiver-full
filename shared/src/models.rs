use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Video {
    pub id: Uuid,
    pub platform: String,
    pub platform_id: String,
    pub title: String,
    pub uploader: String,
    pub uploader_id: String,
    pub duration: i32,
    pub upload_date: Option<DateTime<Utc>>,
    pub subtitles_path: Option<String>,
    pub transcription_path: Option<String>,
    pub heatmap_path: Option<String>,
    pub thumbnail_path: Option<String>,
    pub thumbnail_url: Option<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Live {
    pub id: Uuid,
    pub platform: String,
    pub platform_id: String,
    pub title: String,
    pub uploader: String,
    pub uploader_id: String,
    pub duration: Option<i32>,
    pub live_date: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub transcription_path: Option<String>,
    pub thumbnail_path: Option<String>,
    pub thumbnail_url: Option<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Channel {
    pub id: Uuid,
    pub platform: String,
    pub platform_id: String,
    pub name: String,
    pub watcher_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WatchedChannel {
    pub id: Uuid,
    pub client_id: Uuid,
    pub platform: String,
    pub platform_id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Cookie {
    pub id: Uuid,
    pub platform: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ApiKey {
    pub id: Uuid,
    pub key: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VideoMetadata {
    pub platform_id: String,
    pub title: String,
    pub uploader: String,
    pub uploader_id: String,
    pub duration: i32,
    pub upload_date: Option<DateTime<Utc>>,
    pub thumbnail_url: Option<String>,
    pub description: Option<String>,
    pub is_live: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_serialize() {
        let c = Channel {
            id: Uuid::new_v4(),
            platform: "youtube".into(),
            platform_id: "UC123".into(),
            name: "Test".into(),
            watcher_count: 5,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let json = serde_json::to_string(&c).unwrap();
        let back: Channel = serde_json::from_str(&json).unwrap();
        assert_eq!(back.platform, "youtube");
        assert_eq!(back.watcher_count, 5);
    }

    #[test]
    fn test_watched_channel_serialize() {
        let w = WatchedChannel {
            id: Uuid::new_v4(),
            client_id: Uuid::new_v4(),
            platform: "tiktok".into(),
            platform_id: "@user".into(),
            name: "User".into(),
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&w).unwrap();
        let back: WatchedChannel = serde_json::from_str(&json).unwrap();
        assert_eq!(back.platform, "tiktok");
    }

    #[test]
    fn test_cookie_serialize() {
        let c = Cookie {
            id: Uuid::new_v4(),
            platform: "instagram".into(),
            content: "sessionid=abc".into(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let json = serde_json::to_string(&c).unwrap();
        let back: Cookie = serde_json::from_str(&json).unwrap();
        assert_eq!(back.content, "sessionid=abc");
    }

    #[test]
    fn test_video_metadata_serialize() {
        let m = VideoMetadata {
            platform_id: "vid1".into(),
            title: "Test Video".into(),
            uploader: "Creator".into(),
            uploader_id: "uc1".into(),
            duration: 120,
            upload_date: Some(Utc::now()),
            thumbnail_url: Some("https://example.com/t.jpg".into()),
            description: Some("desc".into()),
            is_live: false,
        };
        let json = serde_json::to_string(&m).unwrap();
        let back: VideoMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(back.title, "Test Video");
        assert!(!back.is_live);
    }

    #[test]
    fn test_apikey_serialize() {
        let k = ApiKey {
            id: Uuid::new_v4(),
            key: "hash123".into(),
            name: "my-client".into(),
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&k).unwrap();
        let back: ApiKey = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "my-client");
    }
}