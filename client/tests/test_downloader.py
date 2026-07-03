from unittest.mock import patch, MagicMock
from client.downloader import Downloader
from client.tracker import DownloadTracker
from client.storage import LocalStorage
import tempfile
import json


def test_fetch_channel_parses_output():
    with tempfile.TemporaryDirectory() as tmp:
        config = {"storage_path": tmp, "cookies_path": None, "proxy_list": []}
        tracker = DownloadTracker(f"{tmp}/db.sqlite")
        storage = LocalStorage(tmp)
        dl = Downloader(config, tracker, storage)

        fake_json = json.dumps({"id": "abc123", "title": "Test Vid", "duration": 120})
        with patch("subprocess.run") as mock_run:
            mock_run.return_value.returncode = 0
            mock_run.return_value.stdout = fake_json
            mock_run.return_value.stderr = ""

            result = dl.fetch_channel_videos("youtube", "UC123")
            assert result is not None
            assert len(result["videos"]) == 1
            assert result["videos"][0]["id"] == "abc123"


def test_fetch_channel_handles_failure():
    with tempfile.TemporaryDirectory() as tmp:
        config = {"storage_path": tmp, "cookies_path": None, "proxy_list": []}
        tracker = DownloadTracker(f"{tmp}/db.sqlite")
        storage = LocalStorage(tmp)
        dl = Downloader(config, tracker, storage)

        with patch("subprocess.run") as mock_run:
            mock_run.return_value.returncode = 1
            mock_run.return_value.stderr = "error"
            result = dl.fetch_channel_videos("youtube", "UC123")
            assert result is None


def test_download_skips_existing():
    with tempfile.TemporaryDirectory() as tmp:
        config = {"storage_path": tmp, "cookies_path": None, "proxy_list": []}
        tracker = DownloadTracker(f"{tmp}/db.sqlite")
        tracker.mark_downloaded("youtube", "abc", "Title", "/path")
        storage = LocalStorage(tmp)
        dl = Downloader(config, tracker, storage)

        result = dl.download_video("youtube", "abc", "Title")
        assert result is None  # already downloaded


def test_download_returns_retry_on_403():
    with tempfile.TemporaryDirectory() as tmp:
        config = {"storage_path": tmp, "cookies_path": None, "proxy_list": ["socks5://localhost:9050"]}
        tracker = DownloadTracker(f"{tmp}/db.sqlite")
        storage = LocalStorage(tmp)
        dl = Downloader(config, tracker, storage)

        with patch("subprocess.run") as mock_run:
            mock_run.return_value.returncode = 1
            mock_run.return_value.stderr = "403 Forbidden"
            mock_run.return_value.stdout = ""
            result = dl.download_video("youtube", "abc", "Title")
            assert result == "retry"


def test_proxy_rotation():
    with tempfile.TemporaryDirectory() as tmp:
        config = {"storage_path": tmp, "cookies_path": None, "proxy_list": ["p1", "p2", "p3"]}
        tracker = DownloadTracker(f"{tmp}/db.sqlite")
        storage = LocalStorage(tmp)
        dl = Downloader(config, tracker, storage)

        proxies = []
        for _ in range(5):
            p = dl._get_proxy()
            proxies.append(p)
        assert proxies == ["p1", "p2", "p3", "p1", "p2"]


def test_download_live_not_implemented():
    with tempfile.TemporaryDirectory() as tmp:
        config = {"storage_path": tmp, "cookies_path": None, "proxy_list": []}
        tracker = DownloadTracker(f"{tmp}/db.sqlite")
        storage = LocalStorage(tmp)
        dl = Downloader(config, tracker, storage)
        assert dl.download_live("youtube", "abc", "Live") is None