# parcel-trackers

[@parcel_trackers_bot](https://t.me/parcel_trackers_bot)

## Supported companies
1. CJ대한통운
2. 우체국

## docker compose
```yaml
version: "3.8"

services:
  parcel-trackers:
    build: ghcr.io/broot5/parcel-trackers:latest
    container_name: parcel-trackers
    restart: unless-stopped
    volumes:
      - ./db:/app/db
    environment:
      - TELOXIDE_TOKEN=00000000
```