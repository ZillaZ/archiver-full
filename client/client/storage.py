import logging
import os
import shutil
from pathlib import Path

logger = logging.getLogger(__name__)


class BaseStorage:
    def save(self, source_path, dest_name):
        raise NotImplementedError

    def delete(self, path):
        raise NotImplementedError


class LocalStorage(BaseStorage):
    def __init__(self, base_path):
        self.base_path = Path(base_path)
        os.makedirs(self.base_path, exist_ok=True)

    def save(self, source_path, dest_name):
        dest = self.base_path / dest_name
        os.makedirs(dest.parent, exist_ok=True)
        shutil.move(str(source_path), str(dest))
        logger.info(f"stored locally: {dest}")
        return str(dest)

    def delete(self, path):
        p = Path(path)
        if p.exists():
            p.unlink()


class S3Storage(BaseStorage):
    def __init__(self, bucket, prefix="", endpoint_url=None):
        import boto3
        kwargs = {}
        if endpoint_url:
            kwargs["endpoint_url"] = endpoint_url
        self.client = boto3.client("s3", **kwargs)
        self.bucket = bucket
        self.prefix = prefix.rstrip("/")

    def save(self, source_path, dest_name):
        key = f"{self.prefix}/{dest_name}" if self.prefix else dest_name
        self.client.upload_file(str(source_path), self.bucket, key)
        logger.info(f"stored in s3://{self.bucket}/{key}")
        return f"s3://{self.bucket}/{key}"

    def delete(self, path):
        key = path.replace(f"s3://{self.bucket}/", "")
        self.client.delete_object(Bucket=self.bucket, Key=key)


def create_storage(backend, config):
    if backend == "local":
        return LocalStorage(config.get("storage_path", "~/Downloads/bigshit"))
    elif backend == "s3":
        return S3Storage(
            bucket=config.get("s3_bucket", "bigshit"),
            prefix=config.get("s3_prefix", ""),
            endpoint_url=config.get("s3_endpoint"),
        )
    else:
        raise ValueError(f"unknown storage backend: {backend}")