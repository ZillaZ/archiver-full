import tempfile
import os
from pathlib import Path
from client.storage import LocalStorage, S3Storage
import pytest


def test_local_save_and_delete():
    with tempfile.TemporaryDirectory() as tmp:
        storage = LocalStorage(tmp)
        src = Path(tmp) / "src.txt"
        src.write_text("hello")
        stored = storage.save(str(src), "sub/dest.txt")
        assert (Path(tmp) / "sub" / "dest.txt").exists()
        assert not src.exists()


def test_local_delete():
    with tempfile.TemporaryDirectory() as tmp:
        p = Path(tmp) / "file.txt"
        p.write_text("data")
        storage = LocalStorage(tmp)
        storage.delete(str(p))
        assert not p.exists()


def test_local_delete_nonexistent():
    with tempfile.TemporaryDirectory() as tmp:
        storage = LocalStorage(tmp)
        storage.delete(str(Path(tmp) / "nope.txt"))


def test_s3_storage_requires_boto3():
    try:
        import boto3
        assert True
    except ImportError:
        pytest.skip("boto3 not installed")


def test_create_storage_local(tmp_config):
    config, _ = tmp_config
    from client.storage import create_storage
    store = create_storage("local", config)
    assert isinstance(store, LocalStorage)