# bigshit

Distributed video/livestream archiving system. Client-server architecture with optional SaaS API.

## Architecture

```
bigshit/
├── api/          # External API (Rust/Axum, HTTP/2, SSE)
├── scraper/      # Metadata scraper (Rust, yt-dlp)
├── shared/       # Shared types (Rust, serde + sqlx)
├── meili-init/   # Meilisearch indexer (Rust)
├── client/       # Daemon + web UI (Python, Flask)
└── migrations/   # PostgreSQL schema
```

## Quick Start

### Prerequisites
- Docker + Docker Compose
- Rust 1.85+
- Python 3.12+
- yt-dlp + ffmpeg (for client)

### Infrastructure
```bash
docker compose up -d
```

This starts PostgreSQL, RabbitMQ, Meilisearch, API, and Scraper.

### Client (standalone)
```bash
cd client
pip install -r requirements.txt
python -m client.main
```

Web UI at `http://127.0.0.1:8080`

### Configuration
- Client config: `~/.config/bigshit/config.yaml`
- Server config: `.env` (copy from `.env.example`)

## Running Tests

### Rust
```bash
cargo test -p shared
```

### Python
```bash
cd client
pip install pytest responses pytest-mock
PYTHONPATH=. python -m pytest tests/
```

## License
GPL-3.0