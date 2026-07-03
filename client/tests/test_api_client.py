from client.api_client import APIClient
import responses
import json


@responses.activate
def test_authenticate_success():
    api = APIClient("http://localhost:3000", "test-key")
    responses.add(
        responses.POST, "http://localhost:3000/v1/auth/login",
        json={"token": "jwt-token"}, status=200,
    )
    assert api.authenticate()
    assert api.is_authenticated()


@responses.activate
def test_authenticate_failure():
    api = APIClient("http://localhost:3000", "bad-key")
    responses.add(
        responses.POST, "http://localhost:3000/v1/auth/login",
        json={"error": "unauthorized"}, status=401,
    )
    assert not api.authenticate()
    assert not api.is_authenticated()


@responses.activate
def test_authenticate_no_key():
    api = APIClient("http://localhost:3000")
    assert not api.authenticate()
    assert not api.is_authenticated()


@responses.activate
def test_upload_cookies():
    api = APIClient("http://localhost:3000", "key")
    api.token = "jwt"
    responses.add(
        responses.POST, "http://localhost:3000/v1/cookies",
        json={"platform": "youtube", "content": "cookie"}, status=200,
    )
    assert api.upload_cookies("youtube", "cookie")


@responses.activate
def test_add_channel():
    api = APIClient("http://localhost:3000", "key")
    api.token = "jwt"
    responses.add(
        responses.POST, "http://localhost:3000/v1/channels",
        json={"id": "uuid", "platform": "youtube", "platform_id": "UCx", "name": "Test"},
        status=200,
    )
    assert api.add_channel("youtube", "UCx", "Test")


@responses.activate
def test_remove_channel():
    api = APIClient("http://localhost:3000", "key")
    api.token = "jwt"
    responses.add(
        responses.DELETE, "http://localhost:3000/v1/channels/youtube/UCx",
        json={"status": "removed"}, status=200,
    )
    assert api.remove_channel("youtube", "UCx")


@responses.activate
def test_remove_channel_404():
    api = APIClient("http://localhost:3000", "key")
    api.token = "jwt"
    responses.add(
        responses.DELETE, "http://localhost:3000/v1/channels/youtube/UCx",
        json={"error": "not found"}, status=404,
    )
    assert api.remove_channel("youtube", "UCx")  # 404 is OK (already removed)


@responses.activate
def test_sync_channels():
    api = APIClient("http://localhost:3000", "key")
    api.token = "jwt"
    responses.add(
        responses.POST, "http://localhost:3000/v1/channels",
        json={}, status=200,
    )
    channels = [{"platform": "youtube", "platform_id": "UCx", "name": "Test"}]
    assert api.sync_channels(channels)


@responses.activate
def test_upload_transcription():
    api = APIClient("http://localhost:3000", "key")
    api.token = "jwt"
    responses.add(
        responses.POST, "http://localhost:3000/v1/transcriptions/vid1",
        json={}, status=200,
    )
    assert api.upload_transcription("vid1", "transcription text")