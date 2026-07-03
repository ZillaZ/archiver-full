import json
import logging
import os
import threading
from pathlib import Path

from flask import Flask, jsonify, request, render_template, send_from_directory

logger = logging.getLogger(__name__)


def create_app(tracker, config, api_client, downloader, config_path):
    app = Flask(__name__, static_folder=None)

    @app.route("/")
    def index():
        return render_template("index.html")

    @app.route("/api/channels", methods=["GET"])
    def list_channels():
        return jsonify(tracker.list_channels())

    @app.route("/api/channels", methods=["POST"])
    def add_channel():
        data = request.get_json()
        platform = data.get("platform")
        platform_id = data.get("platform_id")
        name = data.get("name", platform_id)
        if not platform or not platform_id:
            return jsonify({"error": "platform and platform_id required"}), 400

        tracker.add_channel(platform, platform_id, name)
        if api_client.is_authenticated():
            threading.Thread(
                target=api_client.add_channel,
                args=(platform, platform_id, name),
                daemon=True,
            ).start()
        return jsonify({"status": "added"})

    @app.route("/api/channels/<platform>/<platform_id>", methods=["DELETE"])
    def remove_channel(platform, platform_id):
        tracker.remove_channel(platform, platform_id)
        if api_client.is_authenticated():
            threading.Thread(
                target=api_client.remove_channel,
                args=(platform, platform_id),
                daemon=True,
            ).start()
        return jsonify({"status": "removed"})

    @app.route("/api/downloads")
    def list_downloads():
        limit = request.args.get("limit", 50, type=int)
        return jsonify(tracker.list_downloads(limit))

    @app.route("/api/config", methods=["GET"])
    def get_config():
        public = {k: v for k, v in config.items() if k != "api_key"}
        public["api_connected"] = api_client.is_authenticated()
        return jsonify(public)

    @app.route("/api/config", methods=["POST"])
    def update_config():
        data = request.get_json()
        for key in ["api_endpoint", "api_key", "cookies_path", "storage_backend",
                      "storage_path", "whisper_model", "download_interval", "web_port"]:
            if key in data:
                config[key] = data[key]

        config_path.parent.mkdir(parents=True, exist_ok=True)
        import yaml
        with open(config_path, "w") as f:
            yaml.dump(config, f)

        if "api_key" in data:
            api_client.api_key = data["api_key"]
            api_client.authenticate()

        return jsonify({"status": "saved"})

    @app.route("/api/connect", methods=["POST"])
    def connect_api():
        data = request.get_json()
        api_key = data.get("api_key", config.get("api_key"))
        api_client.api_key = api_key
        ok = api_client.authenticate()
        if ok:
            channels = tracker.list_channels()
            threading.Thread(
                target=api_client.sync_channels,
                args=(channels,),
                daemon=True,
            ).start()
            return jsonify({"status": "connected"})
        return jsonify({"error": "authentication failed"}), 401

    @app.route("/api/disconnect", methods=["POST"])
    def disconnect_api():
        api_client.token = None
        api_client.api_key = ""
        config["api_key"] = ""
        import yaml
        config_path.parent.mkdir(parents=True, exist_ok=True)
        with open(config_path, "w") as f:
            yaml.dump(config, f)
        return jsonify({"status": "disconnected"})

    @app.route("/api/status")
    def daemon_status():
        return jsonify({
            "running": True,
            "api_connected": api_client.is_authenticated(),
            "channels_count": len(tracker.list_channels()),
            "downloads_count": len(tracker.list_downloads(5)),
        })

    @app.route("/api/files")
    def list_files():
        base = Path(config["storage_path"]).resolve()
        sub = request.args.get("path", "")
        target = (base / sub).resolve()
        if not str(target).startswith(str(base)):
            return jsonify({"error": "invalid path"}), 400
        if not target.exists():
            return jsonify({"files": [], "dirs": [], "path": sub})
        files, dirs = [], []
        for entry in sorted(target.iterdir()):
            if entry.is_dir():
                dirs.append({"name": entry.name, "path": str(entry.relative_to(base))})
            else:
                size = entry.stat().st_size
                ext = entry.suffix.lower()
                files.append({"name": entry.name, "path": str(entry.relative_to(base)), "size": size, "ext": ext})
        return jsonify({"files": files, "dirs": dirs, "path": sub})

    @app.route("/api/files/download/<path:filepath>")
    def serve_file(filepath):
        base = Path(config["storage_path"]).resolve()
        target = (base / filepath).resolve()
        if not str(target).startswith(str(base)) or not target.exists():
            return jsonify({"error": "not found"}), 404
        return send_from_directory(str(base), filepath)

    @app.route("/api/files/delete/<path:filepath>", methods=["DELETE"])
    def delete_file(filepath):
        base = Path(config["storage_path"]).resolve()
        target = (base / filepath).resolve()
        if not str(target).startswith(str(base)):
            return jsonify({"error": "invalid path"}), 400
        if target.is_file():
            target.unlink()
        elif target.is_dir():
            import shutil
            shutil.rmtree(target)
        return jsonify({"status": "deleted"})

    @app.route("/api/search")
    def search():
        if not api_client.is_authenticated():
            return jsonify({"error": "API not connected"}), 401
        q = request.args.get("q", "")
        stype = request.args.get("type", "all")
        limit = request.args.get("limit", 20, type=int)
        offset = request.args.get("offset", 0, type=int)
        try:
            if stype == "videos":
                url = f"{api_client.endpoint}/v1/search/videos"
            elif stype == "lives":
                url = f"{api_client.endpoint}/v1/search/lives"
            else:
                url = f"{api_client.endpoint}/v1/search/all"
            import requests as req_lib
            resp = req_lib.get(url, params={"q": q, "limit": limit, "offset": offset},
                                headers=api_client._headers(), timeout=10)
            resp.raise_for_status()
            return jsonify(resp.json())
        except Exception as e:
            return jsonify({"error": str(e)}), 502

    @app.route("/api/video/<platform>/<platform_id>")
    def video_detail(platform, platform_id):
        downloads = tracker.list_downloads(100)
        vid = next((d for d in downloads if d["platform"] == platform and d["platform_id"] == platform_id), None)
        return jsonify(vid or {})

    @app.route("/static/<path:filename>")
    def static_files(filename):
        return send_from_directory(str(Path(__file__).parent / "static"), filename)

    return app


def run_web(app, host="127.0.0.1", port=8080):
    logger.info(f"web UI starting on http://{host}:{port}")
    app.run(host=host, port=port, debug=False, use_reloader=False)