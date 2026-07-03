import logging
import sqlite3
import os
from pathlib import Path

logger = logging.getLogger(__name__)


class DownloadTracker:
    def __init__(self, db_path=None):
        if db_path is None:
            db_path = Path("~/.local/share/bigshit/tracker.db").expanduser()
        os.makedirs(os.path.dirname(str(db_path)), exist_ok=True)
        self.conn = sqlite3.connect(str(db_path))
        self.conn.row_factory = sqlite3.Row
        self._init_db()

    def _init_db(self):
        self.conn.executescript("""
            CREATE TABLE IF NOT EXISTS downloads (
                id TEXT PRIMARY KEY,
                platform TEXT NOT NULL,
                platform_id TEXT NOT NULL,
                title TEXT,
                file_path TEXT,
                status TEXT NOT NULL DEFAULT 'downloaded',
                downloaded_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(platform, platform_id)
            );
            CREATE TABLE IF NOT EXISTS transcriptions (
                video_id TEXT PRIMARY KEY,
                model TEXT,
                language TEXT,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );
            CREATE TABLE IF NOT EXISTS watched_channels (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                platform TEXT NOT NULL,
                platform_id TEXT NOT NULL,
                name TEXT NOT NULL,
                added_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                last_scrape TIMESTAMP,
                UNIQUE(platform, platform_id)
            );
        """)
        self.conn.commit()

    def is_downloaded(self, platform, platform_id):
        cursor = self.conn.execute(
            "SELECT 1 FROM downloads WHERE platform = ? AND platform_id = ?",
            (platform, platform_id),
        )
        return cursor.fetchone() is not None

    def mark_downloaded(self, platform, platform_id, title, file_path):
        self.conn.execute(
            "INSERT OR REPLACE INTO downloads (id, platform, platform_id, title, file_path) "
            "VALUES (?, ?, ?, ?, ?)",
            (f"{platform}_{platform_id}", platform, platform_id, title, file_path),
        )
        self.conn.commit()

    def is_transcribed(self, video_id):
        cursor = self.conn.execute(
            "SELECT 1 FROM transcriptions WHERE video_id = ?", (video_id,)
        )
        return cursor.fetchone() is not None

    def mark_transcribed(self, video_id, model, language=None):
        self.conn.execute(
            "INSERT OR REPLACE INTO transcriptions (video_id, model, language) VALUES (?, ?, ?)",
            (video_id, model, language),
        )
        self.conn.commit()

    def list_channels(self):
        cursor = self.conn.execute(
            "SELECT platform, platform_id, name, added_at, last_scrape FROM watched_channels ORDER BY name"
        )
        return [dict(row) for row in cursor.fetchall()]

    def add_channel(self, platform, platform_id, name):
        self.conn.execute(
            "INSERT OR IGNORE INTO watched_channels (platform, platform_id, name) VALUES (?, ?, ?)",
            (platform, platform_id, name),
        )
        self.conn.commit()

    def remove_channel(self, platform, platform_id):
        self.conn.execute(
            "DELETE FROM watched_channels WHERE platform = ? AND platform_id = ?",
            (platform, platform_id),
        )
        self.conn.commit()

    def update_last_scrape(self, platform, platform_id):
        self.conn.execute(
            "UPDATE watched_channels SET last_scrape = CURRENT_TIMESTAMP WHERE platform = ? AND platform_id = ?",
            (platform, platform_id),
        )
        self.conn.commit()

    def list_downloads(self, limit=50):
        cursor = self.conn.execute(
            "SELECT platform, platform_id, title, file_path, status, downloaded_at FROM downloads ORDER BY downloaded_at DESC LIMIT ?",
            (limit,),
        )
        return [dict(row) for row in cursor.fetchall()]

    def close(self):
        self.conn.close()