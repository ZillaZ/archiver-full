import os
import yaml
from pathlib import Path

DEFAULT_CONFIG_PATH = Path("~/.config/bigshit/config.yaml").expanduser()


def load_config(path=None):
    path = path or DEFAULT_CONFIG_PATH
    defaults = {
        "api_endpoint": "http://localhost:3000",
        "api_key": "",
        "cookies_path": str(Path("~/.config/bigshit/cookies").expanduser()),
        "storage_backend": "local",
        "storage_path": str(Path("~/Downloads/bigshit").expanduser()),
        "proxy_list": [],
        "whisper_model": "small",
        "download_interval": 300,
    }

    if path.exists():
        with open(path) as f:
            user_config = yaml.safe_load(f) or {}
        defaults.update(user_config)

    defaults["storage_path"] = str(Path(defaults["storage_path"]).expanduser())
    defaults["cookies_path"] = str(Path(defaults["cookies_path"]).expanduser())

    return defaults