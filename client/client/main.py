import logging
import os
import signal
import sys
import threading
import time
from pathlib import Path

from .config import load_config
from .tracker import DownloadTracker
from .storage import create_storage
from .downloader import Downloader
from .transcriber import Transcriber, NoOpTranscriber
from .api_client import APIClient
from .web.app import create_app, run_web

logger = logging.getLogger(__name__)

running = True


def handle_signal(signum, frame):
    global running
    logger.info(f"received signal {signum}, shutting down")
    running = False


def scrape_channels(tracker, downloader, api_client):
    channels = tracker.list_channels()
    for ch in channels:
        if not running:
            break
        platform = ch["platform"]
        platform_id = ch["platform_id"]
        name = ch["name"]

        logger.info(f"checking channel: {name} ({platform})")

        result = downloader.fetch_channel_videos(platform, platform_id)
        if result and "videos" in result:
            for video in result["videos"]:
                if not running:
                    break
                vid = video.get("id") or video.get("platform_id")
                if vid and not tracker.is_downloaded(platform, vid):
                    title = video.get("title", "")
                    filepath = downloader.download_video(platform, vid, title)
                    if filepath and filepath != "retry":
                        tracker.update_last_scrape(platform, platform_id)

        tracker.update_last_scrape(platform, platform_id)


def main():
    global running
    signal.signal(signal.SIGTERM, handle_signal)
    signal.signal(signal.SIGINT, handle_signal)

    logging.basicConfig(
        level=logging.INFO,
        format="%(asctime)s [%(levelname)s] %(name)s: %(message)s",
    )

    config = load_config()

    data_dir = Path("~/.local/share/bigshit").expanduser()
    os.makedirs(data_dir, exist_ok=True)

    tracker = DownloadTracker(str(data_dir / "tracker.db"))
    storage = create_storage(config["storage_backend"], config)
    downloader = Downloader(config, tracker, storage)
    transcriber = Transcriber(model=config["whisper_model"])

    api = APIClient(config["api_endpoint"], config.get("api_key"))
    api.authenticate()

    cookies_path = config.get("cookies_path")
    if cookies_path and os.path.exists(cookies_path):
        for platform in ["youtube", "tiktok", "instagram"]:
            platform_cookies = os.path.join(cookies_path, f"{platform}.txt")
            if os.path.exists(platform_cookies):
                with open(platform_cookies) as f:
                    content = f.read()
                api.upload_cookies(platform, content)

    if api.is_authenticated():
        channels = tracker.list_channels()
        api.sync_channels(channels)

        def handle_notification(msg):
            logger.info(f"received notification: {msg}")

        sse_thread = threading.Thread(
            target=api.listen_events,
            args=(handle_notification,),
            daemon=True,
        )
        sse_thread.start()

    config_path = Path("~/.config/bigshit/config.yaml").expanduser()
    web_app = create_app(tracker, config, api, downloader, config_path)
    web_port = config.get("web_port", 8080)
    web_thread = threading.Thread(
        target=run_web,
        args=(web_app, "127.0.0.1", web_port),
        daemon=True,
    )
    web_thread.start()
    logger.info(f"web UI on http://127.0.0.1:{web_port}")

    logger.info("client daemon started")
    interval = config.get("download_interval", 300)
    while running:
        time.sleep(interval)
        if running:
            scrape_channels(tracker, downloader, api)

    tracker.close()
    logger.info("client shut down")


if __name__ == "__main__":
    main()