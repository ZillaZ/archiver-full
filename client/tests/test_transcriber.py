import tempfile
import os
from pathlib import Path
from unittest.mock import patch
from client.transcriber import Transcriber


def test_no_op_fallback():
    from client.transcriber import NoOpTranscriber
    t = NoOpTranscriber()
    assert t.transcribe("/fake/path") is None


def test_transcriber_not_found():
    t = Transcriber("tiny")
    with patch("subprocess.run") as mock_run:
        mock_run.side_effect = FileNotFoundError()
        result = t.transcribe("/fake/audio.mp3")
        assert result is None


def test_transcriber_timeout():
    t = Transcriber("tiny")
    with patch("subprocess.run") as mock_run:
        import subprocess
        mock_run.side_effect = subprocess.TimeoutExpired("cmd", 3600)
        result = t.transcribe("/fake/audio.mp3")
        assert result is None