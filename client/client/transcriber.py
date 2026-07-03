import json
import logging
import os
import subprocess
import tempfile

logger = logging.getLogger(__name__)


class Transcriber:
    def __init__(self, model="small"):
        self.model = model

    def transcribe(self, audio_path):
        logger.info(f"transcribing {audio_path} with whisper-cpp (model={self.model})")
        try:
            result = subprocess.run(
                [
                    "whisper-cpp",
                    "--model", self.model,
                    "--output-json",
                    "--file", audio_path,
                ],
                capture_output=True,
                text=True,
                timeout=3600,
            )
            if result.returncode != 0:
                logger.error(f"whisper-cpp failed: {result.stderr}")
                return None

            for line in result.stdout.split("\n"):
                if line.strip():
                    try:
                        data = json.loads(line)
                        return data.get("text", "")
                    except json.JSONDecodeError:
                        continue

            return result.stdout
        except FileNotFoundError:
            logger.error("whisper-cpp not found in PATH")
            return None
        except subprocess.TimeoutExpired:
            logger.error("whisper-cpp timed out")
            return None
        except Exception as e:
            logger.error(f"transcription failed: {e}")
            return None


class NoOpTranscriber:
    def transcribe(self, audio_path):
        logger.warning("transcriber not configured, skipping transcription")
        return None