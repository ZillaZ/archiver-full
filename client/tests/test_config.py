from client.config import load_config
import yaml
import tempfile
from pathlib import Path


def test_load_defaults():
    config = load_config()
    assert config["api_endpoint"] == "http://localhost:3000"
    assert config["storage_backend"] == "local"
    assert config["download_interval"] == 300


def test_load_from_file(tmp_config):
    user_config, path = tmp_config
    loaded = load_config(path)
    assert loaded["api_key"] == "test-key"
    assert loaded["storage_backend"] == "local"
    assert loaded["proxy_list"] == ["socks5://127.0.0.1:9050"]


def test_override_defaults():
    data = {"api_endpoint": "https://custom.api.com", "download_interval": 600}
    with tempfile.NamedTemporaryFile(mode="w", suffix=".yaml", delete=False) as f:
        yaml.dump(data, f)
        path = Path(f.name)
    config = load_config(path)
    assert config["api_endpoint"] == "https://custom.api.com"
    assert config["download_interval"] == 600
    assert config["storage_backend"] == "local"
    path.unlink()


def test_expands_paths():
    data = {"storage_path": "~/mydownloads"}
    with tempfile.NamedTemporaryFile(mode="w", suffix=".yaml", delete=False) as f:
        yaml.dump(data, f)
        path = Path(f.name)
    config = load_config(path)
    assert "~" not in config["storage_path"]
    path.unlink()