from client.web.app import create_app
from client.tracker import DownloadTracker
from client.api_client import APIClient
from client.downloader import Downloader
from client.storage import LocalStorage
import pytest


@pytest.fixture
def client(tmp_path):
    tracker = DownloadTracker(str(tmp_path / "test.db"))
    config = {
        "api_endpoint": "http://localhost:3000",
        "api_key": "",
        "storage_backend": "local",
        "storage_path": str(tmp_path / "downloads"),
        "proxy_list": [],
        "whisper_model": "tiny",
        "download_interval": 300,
        "web_port": 8080,
        "cookies_path": str(tmp_path / "cookies"),
    }
    api = APIClient("http://localhost:3000")
    storage = LocalStorage(config["storage_path"])
    dl = Downloader(config, tracker, storage)
    app = create_app(tracker, config, api, dl, tmp_path / "config.yaml")
    app.config["TESTING"] = True
    yield app.test_client()
    tracker.close()


def test_index(client):
    resp = client.get("/")
    assert resp.status_code == 200
    assert b"Bigshit Client" in resp.data


def test_list_channels_empty(client):
    resp = client.get("/api/channels")
    assert resp.status_code == 200
    assert resp.json == []


def test_add_channel(client):
    resp = client.post("/api/channels", json={
        "platform": "youtube", "platform_id": "UC123", "name": "Test"
    })
    assert resp.status_code == 200
    assert resp.json["status"] == "added"
    resp = client.get("/api/channels")
    assert len(resp.json) == 1
    assert resp.json[0]["platform_id"] == "UC123"


def test_remove_channel(client):
    client.post("/api/channels", json={"platform": "youtube", "platform_id": "UCx", "name": "T"})
    resp = client.delete("/api/channels/youtube/UCx")
    assert resp.status_code == 200
    resp = client.get("/api/channels")
    assert len(resp.json) == 0


def test_add_duplicate_channel(client):
    client.post("/api/channels", json={"platform": "youtube", "platform_id": "UCx", "name": "A"})
    client.post("/api/channels", json={"platform": "youtube", "platform_id": "UCx", "name": "B"})
    resp = client.get("/api/channels")
    assert len(resp.json) == 1


def test_config_get(client):
    resp = client.get("/api/config")
    assert resp.status_code == 200
    assert "api_endpoint" in resp.json


def test_config_post(client):
    resp = client.post("/api/config", json={"download_interval": 600})
    assert resp.status_code == 200


def test_downloads_empty(client):
    resp = client.get("/api/downloads")
    assert resp.status_code == 200
    assert resp.json == []


def test_status(client):
    resp = client.get("/api/status")
    assert resp.status_code == 200
    assert resp.json["running"] is True
    assert resp.json["channels_count"] == 0


def test_connect_bad_key(client):
    resp = client.post("/api/connect", json={"api_key": "bad"})
    assert resp.status_code == 401


def test_disconnect(client):
    resp = client.post("/api/disconnect")
    assert resp.status_code == 200