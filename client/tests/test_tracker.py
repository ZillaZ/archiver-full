from client.tracker import DownloadTracker


def test_init_creates_tables(tmp_db):
    t = DownloadTracker(tmp_db)
    t.conn.execute("SELECT 1 FROM downloads")
    t.conn.execute("SELECT 1 FROM transcriptions")
    t.conn.execute("SELECT 1 FROM watched_channels")
    t.close()


def test_is_downloaded_returns_false_for_new(tmp_db):
    t = DownloadTracker(tmp_db)
    assert not t.is_downloaded("youtube", "abc123")
    t.close()


def test_mark_and_check_downloaded(tmp_db):
    t = DownloadTracker(tmp_db)
    t.mark_downloaded("youtube", "abc123", "Test Video", "/path/to/file.mp4")
    assert t.is_downloaded("youtube", "abc123")
    assert not t.is_downloaded("youtube", "other")
    t.close()


def test_add_and_list_channels(tmp_db):
    t = DownloadTracker(tmp_db)
    assert len(t.list_channels()) == 0
    t.add_channel("youtube", "UC123", "Test Channel")
    channels = t.list_channels()
    assert len(channels) == 1
    assert channels[0]["platform"] == "youtube"
    assert channels[0]["platform_id"] == "UC123"
    t.close()


def test_add_duplicate_channel(tmp_db):
    t = DownloadTracker(tmp_db)
    t.add_channel("youtube", "UC123", "Channel A")
    t.add_channel("youtube", "UC123", "Channel B")
    assert len(t.list_channels()) == 1
    t.close()


def test_remove_channel(tmp_db):
    t = DownloadTracker(tmp_db)
    t.add_channel("youtube", "UC123", "Test")
    t.remove_channel("youtube", "UC123")
    assert len(t.list_channels()) == 0
    t.close()


def test_download_history(tmp_db):
    t = DownloadTracker(tmp_db)
    t.mark_downloaded("youtube", "v1", "Vid 1", "/a.mp4")
    t.mark_downloaded("youtube", "v2", "Vid 2", "/b.mp4")
    downloads = t.list_downloads(limit=10)
    assert len(downloads) == 2
    t.close()


def test_transcription_tracking(tmp_db):
    t = DownloadTracker(tmp_db)
    assert not t.is_transcribed("video1")
    t.mark_transcribed("video1", "small", "en")
    assert t.is_transcribed("video1")
    t.close()


def test_update_last_scrape(tmp_db):
    t = DownloadTracker(tmp_db)
    t.add_channel("youtube", "UCx", "Test")
    t.update_last_scrape("youtube", "UCx")
    channels = t.list_channels()
    assert channels[0]["last_scrape"] is not None
    t.close()