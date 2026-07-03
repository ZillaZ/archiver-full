import json
import logging
import os
import subprocess
from pathlib import Path

logger = logging.getLogger(__name__)


class Downloader:
    def __init__(self, config, tracker, storage):
        self.config = config
        self.tracker = tracker
        self.storage = storage
        self.proxy_list = config.get("proxy_list", [])
        self.proxy_index = 0

    def _get_proxy(self):
        if not self.proxy_list:
            return None
        proxy = self.proxy_list[self.proxy_index % len(self.proxy_list)]
        self.proxy_index += 1
        return proxy

    def fetch_channel_videos(self, platform, platform_id):
        if platform == "youtube":
            url = f"https://www.youtube.com/@{platform_id}/videos" if not platform_id.startswith("UC") else f"https://www.youtube.com/channel/{platform_id}/videos"
        elif platform == "tiktok":
            url = f"https://www.tiktok.com/@{platform_id}"
        elif platform == "instagram":
            url = f"https://www.instagram.com/{platform_id}/"
        else:
            return None

        args = ["yt-dlp", "--no-warnings", "--ignore-errors", "--dump-json", "--flat-playlist"]
        proxy = self._get_proxy()
        if proxy:
            args.extend(["--proxy", proxy])
        cookies = self.config.get("cookies_path")
        if cookies and os.path.exists(cookies):
            args.extend(["--cookies", cookies])
        args.append(url)

        logger.info(f"fetching channel: {url}")
        try:
            result = subprocess.run(args, capture_output=True, text=True, timeout=120)
            if result.returncode != 0:
                return None
            videos = []
            for line in result.stdout.strip().split("\n"):
                if not line.strip():
                    continue
                try:
                    entry = json.loads(line)
                    videos.append({"id": entry.get("id"), "title": entry.get("title", ""), "duration": entry.get("duration", 0)})
                except json.JSONDecodeError:
                    continue
            return {"videos": videos}
        except Exception as e:
            logger.error(f"fetch failed: {e}")
            return None

    def download_video(self, platform, platform_id, title):
        if self.tracker.is_downloaded(platform, platform_id):
            return None

        output = str(Path(self.config["storage_path"]) / f"{platform}/%(id)s.%(ext)s")

        if platform == "youtube":
            url = f"https://www.youtube.com/watch?v={platform_id}"
        elif platform == "tiktok":
            url = f"https://www.tiktok.com/@i/video/{platform_id}"
        elif platform == "instagram":
            url = f"https://www.instagram.com/p/{platform_id}/"
        else:
            return None

        args = ["yt-dlp", "--no-warnings", "--ignore-errors", "--print", "after_move:filepath", "-o", output]
        proxy = self._get_proxy()
        if proxy:
            args.extend(["--proxy", proxy])
        cookies = self.config.get("cookies_path")
        if cookies and os.path.exists(cookies):
            args.extend(["--cookies", cookies])
        args.append(url)

        logger.info(f"downloading: {title}")
        try:
            result = subprocess.run(args, capture_output=True, text=True, timeout=7200)
            if result.returncode != 0:
                if "403" in result.stderr.lower():
                    return "retry"
                return None
            filepath = result.stdout.strip().split("\n")[-1].strip()
            if filepath:
                stored = self.storage.save(filepath, f"{platform}/{platform_id}.%(ext)s")
                self.tracker.mark_downloaded(platform, platform_id, title, stored)
                return stored
        except Exception as e:
            logger.error(f"download failed: {e}")
        return None

    def download_live(self, platform, platform_id, title):
        logger.info(f"live dl not implemented: {platform}/{platform_id}")
        return None