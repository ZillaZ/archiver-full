use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    ScrapeChannel(ScrapeChannel),
    RemoveChannel(RemoveChannel),
    ScrapeResult(ScrapeResult),
    NewVideoNotification(NewVideoNotification),
    NewLiveNotification(NewLiveNotification),
    CookieUpdate(CookieUpdate),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapeChannel {
    pub channel_id: Uuid,
    pub platform: String,
    pub platform_id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveChannel {
    pub channel_id: Uuid,
    pub platform: String,
    pub platform_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapeResult {
    pub channel_id: Uuid,
    pub videos: Vec<ScrapedVideo>,
    pub active_lives: Vec<ScrapedLive>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapedVideo {
    pub platform_id: String,
    pub title: String,
    pub uploader: String,
    pub uploader_id: String,
    pub duration: i32,
    pub upload_date: Option<String>,
    pub thumbnail_url: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapedLive {
    pub platform_id: String,
    pub title: String,
    pub uploader: String,
    pub uploader_id: String,
    pub thumbnail_url: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewVideoNotification {
    pub video_id: Uuid,
    pub platform: String,
    pub platform_id: String,
    pub title: String,
    pub uploader: String,
    pub duration: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewLiveNotification {
    pub live_id: Uuid,
    pub platform: String,
    pub platform_id: String,
    pub title: String,
    pub uploader: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CookieUpdate {
    pub platform: String,
    pub content: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scrape_channel_roundtrip() {
        let msg = Message::ScrapeChannel(ScrapeChannel {
            channel_id: Uuid::new_v4(),
            platform: "youtube".into(),
            platform_id: "UC123".into(),
            name: "Test Channel".into(),
        });
        let json = serde_json::to_string(&msg).unwrap();
        let back: Message = serde_json::from_str(&json).unwrap();
        match back {
            Message::ScrapeChannel(c) => {
                assert_eq!(c.platform, "youtube");
                assert_eq!(c.platform_id, "UC123");
                assert_eq!(c.name, "Test Channel");
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn test_remove_channel_roundtrip() {
        let msg = Message::RemoveChannel(RemoveChannel {
            channel_id: Uuid::new_v4(),
            platform: "youtube".into(),
            platform_id: "UC123".into(),
        });
        let json = serde_json::to_string(&msg).unwrap();
        let back: Message = serde_json::from_str(&json).unwrap();
        assert!(matches!(back, Message::RemoveChannel(_)));
    }

    #[test]
    fn test_scraped_video_roundtrip() {
        let v = ScrapedVideo {
            platform_id: "abc123".into(),
            title: "My Video".into(),
            uploader: "Creator".into(),
            uploader_id: "uc456".into(),
            duration: 300,
            upload_date: Some("20240101".into()),
            thumbnail_url: Some("https://example.com/thumb.jpg".into()),
            description: Some("A description".into()),
        };
        let json = serde_json::to_string(&v).unwrap();
        let back: ScrapedVideo = serde_json::from_str(&json).unwrap();
        assert_eq!(back.title, "My Video");
        assert_eq!(back.duration, 300);
    }

    #[test]
    fn test_scraped_live_roundtrip() {
        let l = ScrapedLive {
            platform_id: "live789".into(),
            title: "Live Stream".into(),
            uploader: "Streamer".into(),
            uploader_id: "uc999".into(),
            thumbnail_url: None,
            description: None,
        };
        let json = serde_json::to_string(&l).unwrap();
        let back: ScrapedLive = serde_json::from_str(&json).unwrap();
        assert_eq!(back.title, "Live Stream");
        assert!(back.thumbnail_url.is_none());
    }

    #[test]
    fn test_cookie_update_roundtrip() {
        let msg = Message::CookieUpdate(CookieUpdate {
            platform: "tiktok".into(),
            content: "cookie=value".into(),
        });
        let json = serde_json::to_string(&msg).unwrap();
        let back: Message = serde_json::from_str(&json).unwrap();
        match back {
            Message::CookieUpdate(c) => {
                assert_eq!(c.platform, "tiktok");
                assert_eq!(c.content, "cookie=value");
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn test_notification_roundtrip() {
        let n = NewVideoNotification {
            video_id: Uuid::new_v4(),
            platform: "youtube".into(),
            platform_id: "vid001".into(),
            title: "New Video".into(),
            uploader: "Creator".into(),
            duration: 600,
        };
        let json = serde_json::to_string(&n).unwrap();
        let back: NewVideoNotification = serde_json::from_str(&json).unwrap();
        assert_eq!(back.title, "New Video");
        assert_eq!(back.duration, 600);
    }

    #[test]
    fn test_scrape_result() {
        let result = ScrapeResult {
            channel_id: Uuid::new_v4(),
            videos: vec![ScrapedVideo {
                platform_id: "v1".into(),
                title: "V1".into(),
                uploader: "U".into(),
                uploader_id: "u1".into(),
                duration: 100,
                upload_date: None,
                thumbnail_url: None,
                description: None,
            }],
            active_lives: vec![],
        };
        let json = serde_json::to_string(&result).unwrap();
        let back: ScrapeResult = serde_json::from_str(&json).unwrap();
        assert_eq!(back.videos.len(), 1);
        assert_eq!(back.active_lives.len(), 0);
    }
}