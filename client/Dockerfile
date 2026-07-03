FROM python:3.12-slim

RUN apt-get update && apt-get install -y \
    yt-dlp ffmpeg curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt
COPY client/ client/
COPY config.yaml .

CMD ["python", "-m", "client.main"]