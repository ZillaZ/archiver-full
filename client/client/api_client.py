import json
import logging
import time
import requests
from urllib.parse import urljoin

logger = logging.getLogger(__name__)


class APIClient:
    def __init__(self, endpoint, api_key=None):
        self.endpoint = endpoint.rstrip("/")
        self.api_key = api_key
        self.token = None
        self.session = requests.Session()
        self.session.headers.update({"User-Agent": "bigshit-client/1.0"})

    def _headers(self):
        h = {}
        if self.token:
            h["Authorization"] = f"Bearer {self.token}"
        return h

    def authenticate(self):
        if not self.api_key:
            logger.warning("no API key configured, skipping auth")
            return False
        try:
            resp = self.session.post(
                urljoin(self.endpoint, "/v1/auth/login"),
                json={"api_key": self.api_key},
                timeout=10,
            )
            resp.raise_for_status()
            self.token = resp.json()["token"]
            logger.info("authenticated successfully")
            return True
        except Exception as e:
            logger.error(f"auth failed: {e}")
            self.token = None
            return False

    def is_authenticated(self):
        return self.token is not None

    def upload_cookies(self, platform, cookie_content):
        try:
            resp = self.session.post(
                urljoin(self.endpoint, "/v1/cookies"),
                json={"platform": platform, "content": cookie_content},
                headers=self._headers(),
                timeout=30,
            )
            resp.raise_for_status()
            logger.info(f"uploaded cookies for {platform}")
            return True
        except Exception as e:
            logger.error(f"failed to upload cookies for {platform}: {e}")
            return False

    def upload_transcription(self, video_id, transcription):
        try:
            resp = self.session.post(
                urljoin(self.endpoint, f"/v1/transcriptions/{video_id}"),
                json={"transcription": transcription},
                headers=self._headers(),
                timeout=30,
            )
            resp.raise_for_status()
            logger.info(f"uploaded transcription for video {video_id}")
            return True
        except Exception as e:
            logger.error(f"failed to upload transcription: {e}")
            return False

    def add_channel(self, platform, platform_id, name):
        try:
            resp = self.session.post(
                urljoin(self.endpoint, "/v1/channels"),
                json={"platform": platform, "platform_id": platform_id, "name": name},
                headers=self._headers(),
                timeout=10,
            )
            resp.raise_for_status()
            logger.info(f"synced channel: {name} ({platform})")
            return True
        except Exception as e:
            logger.error(f"failed to sync channel: {e}")
            return False

    def remove_channel(self, platform, platform_id):
        try:
            resp = self.session.delete(
                urljoin(self.endpoint, f"/v1/channels/{platform}/{platform_id}"),
                headers=self._headers(),
                timeout=10,
            )
            if resp.status_code == 404:
                logger.info(f"channel not found on server (already removed): {platform}/{platform_id}")
                return True
            resp.raise_for_status()
            logger.info(f"removed channel from server: {platform}/{platform_id}")
            return True
        except Exception as e:
            logger.error(f"failed to remove channel from server: {e}")
            return False

    def sync_channels(self, local_channels):
        if not self.is_authenticated():
            logger.warning("not authenticated, skipping channel sync")
            return False

        remote_ok = True
        for ch in local_channels:
            if not self.add_channel(ch["platform"], ch["platform_id"], ch["name"]):
                remote_ok = False
        return remote_ok

    def listen_events(self, on_notification):
        if not self.is_authenticated():
            logger.info("not authenticated, skipping SSE")
            return

        url = urljoin(self.endpoint, "/v1/events")
        while True:
            try:
                logger.info("connecting to SSE stream")
                resp = self.session.get(
                    url,
                    headers=self._headers(),
                    stream=True,
                    timeout=None,
                )
                resp.raise_for_status()

                buffer = ""
                for chunk in resp.iter_content(chunk_size=1, decode_unicode=True):
                    if chunk is None:
                        continue
                    buffer += chunk
                    if buffer.endswith("\n\n"):
                        for line in buffer.strip().split("\n"):
                            if line.startswith("data: "):
                                data = line[6:]
                                if data == "keep-alive":
                                    continue
                                try:
                                    msg = json.loads(data)
                                    on_notification(msg)
                                except json.JSONDecodeError:
                                    logger.warning(f"invalid SSE data: {data}")
                        buffer = ""

            except requests.exceptions.ConnectionError as e:
                logger.warning(f"SSE connection lost: {e}, reconnecting in 10s")
                time.sleep(10)
            except Exception as e:
                logger.error(f"SSE error: {e}, reconnecting in 30s")
                time.sleep(30)