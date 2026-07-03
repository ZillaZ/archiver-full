import pytest
import tempfile
import os
from pathlib import Path


@pytest.fixture
def tmp_db(tmp_path):
    return str(tmp_path / "test.db")


@pytest.fixture
def tmp_config(tmp_path):
    config = {
        "api_endpoint": "http://localhost:3000",
        "api_key": "test-key",
        "cookies_path": str(tmp_path / "cookies"),
        "storage_backend": "local",
        "storage_path": str(tmp_path / "downloads"),
        "proxy_list": ["socks5://127.0.0.1:9050"],
        "whisper_model": "tiny",
        "download_interval": 60,
        "web_port": 8080,
    }
    config_path = tmp_path / "config.yaml"
    import yaml
    with open(config_path, "w") as f:
        yaml.dump(config, f)
    return config, config_path