use std::collections::HashMap;

pub struct CookieStore {
    cookies: HashMap<String, String>,
}

impl CookieStore {
    pub fn new(cookies: HashMap<String, String>) -> Self {
        Self { cookies }
    }

    pub fn get(&self, platform: &str) -> Option<&String> {
        self.cookies.get(platform)
    }

    pub fn set(&mut self, platform: String, content: String) {
        self.cookies.insert(platform, content);
    }
}

pub async fn load_cookies_from_db(db: &sqlx::PgPool) -> HashMap<String, String> {
    sqlx::query_as::<_, (String, String)>("SELECT platform, content FROM cookies")
        .fetch_all(db)
        .await
        .unwrap_or_default()
        .into_iter()
        .collect()
}