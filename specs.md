# System Specification
## Components
### Client
The client is capable to download videos and livestreams using yt-dlp + ffmpeg.
#### Client Capabilities
- Download Videos/Lives using yt-dlp from any supported platform
- Use user defined cookies
- Allow for custom storage configuration (local directory, S3 compatible API, Drive API, etc.)
- Allow for custom proxy configuration (SOCKS5 server list)
- Keep track of what has been downloaded already (sqlite does the job)
- Interact with external API for video search, listing and other features
- Generate video transcriptions to feed into External API

### External API
This API will be hosted by myself and will work as a SaaS. Beware that because of possible DMCA takedown requests, we need to document the API so users can make their own version of it and self host. We will host our own version too and it will be a paid service.
#### API capabilities
- Fetch video metadata (thumbnail, duration, title, subtitles, video transcription, upload date and etc)
- Notify client of new video/live
- Notify scrapper of a new user to watch or ask it to scrape on command

### Scraper
The scraper is the program responsible of gathering video metadata and checking if any new uploads/lives are going on. This is the service that the External API will interact with.
#### Scraper capabilities
- Fetch user video/live metadata
- Store video thumbnail
- Notify External API of new data
- Receive notifications from External API for new users to watch/scrape on command
- Use proxy (SOCKS5) to avoid IP bans

### RabbitMQ
This will work as a bridge for External API and Scraper, the notifications have been described on their specifications already

### PostgreSQL
PostgreSQL will be responsible for persistent video storage. It will keep all of the videos/live metadata.
#### PostgreSQL schema
- Videos (title, uploader, duration, upload_date, subtitles_path, transcription_path, heatmap_path, thumbnail_path)
- Lives (title, uploader, duration, live_date, transcription_path, thumbnail_path)

### Meilisearch
This will be used for video/live search and will use the data from PostgreSQL paired with a initialization script
#### Meilisearch capabilities
- Search videos/lives by title, uploader, duration, upload date, thumbnail content and transcription
- Sort results by duration and upload date

## Implementation details
- You must use Rust for this task. The yt_dlp crate will do most of the heavylifting for us on the client and on the scraper.
- The client can be made on any tech, pick the easiest/quickiest one. Remember that it will need to be running on the background at all times and it will depend on yt-dlp + ffmpeg to download the videos/lives
- Livestream detection must be really aggressive
- The system needs to be up 100% of time
- The server does not download videos or livestreams, it only gathers metadata. Only the client can download actual data. If the server can't gather metadata by itself, it must gather it from the client
- For now, we can focus on gathering/downloading things from only Youtube, Tiktok and Instagram
