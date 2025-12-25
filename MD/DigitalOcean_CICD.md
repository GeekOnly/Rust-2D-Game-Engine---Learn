# Digital Ocean CI/CD & Deployment Plan (Admin Level)

This document outlines a professional, automated DevOps pipeline to deploy your specialized Game Server to Digital Ocean. This setup allows for "Push-to-Deploy" functionality and provides a Web Admin Interface to manage servers.

## Architecture Overview

1.  **Source Code**: GitHub Repository.
2.  **CI (Build)**: GitHub Actions compiles the Rust binary.
3.  **Containerization**: Docker wraps the binary into a lightweight image.
4.  **Registry**: Digital Ocean Container Registry (DOCR) or GitHub Container Registry (GHCR).
5.  **CD (Deploy)**: 
    - Droplet pulls the new image.
    - `Watchtower` or SSH Script restarts the container.
6.  **Admin Panel**: **Portainer** for Web-based management + **Grafana** for monitoring.

---

## Phase 1: Dockerize the Game Server
*Objective: Create a portable server image that runs anywhere.*

Create a `Dockerfile` in the project root:
```dockerfile
# Build Stage
FROM rust:latest as builder
WORKDIR /app
COPY . .
# Build specifically for the server (headless)
RUN cargo build --release --bin game_server

# Runtime Stage (Tiny Image)
FROM debian:bullseye-slim
WORKDIR /app
# Install minimal dependencies (SSL, etc)
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/game_server /app/server
COPY --from=builder /app/assets /app/assets

ENV RUST_LOG=info
EXPOSE 7777/udp
CMD ["./server"]
```

---

## Phase 2: The "Admin" Server Setup (Digital Ocean)
*Objective: Prepare the Droplet to host multiple game worlds and admin tools.*

### 2.1 Infrastructure
- **Droplet Size**: Basic Regular (CPU optimized recommended for Game Servers). 
- **OS**: Ubuntu 22.04 LTS (Docker ready).

### 2.2 The Admin Stack (Docker Compose)
On the remote server, we run an "Admin Stack" using `docker-compose.yml`:

```yaml
version: '3'

services:
  # 1. The Game Server (Auto-updated)
  game-world-1:
    image: ghcr.io/yourusername/game-engine:latest
    ports:
      - "7777:7777/udp"
    restart: unless-stopped
    labels:
      - "com.centurylinklabs.watchtower.enable=true"

  # 2. Watchtower (The Auto-Updater)
  # Automatically checks for new updates every minute and updates the game server
  watchtower:
    image: containrrr/watchtower
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
    command: --interval 30 --cleanup

  # 3. Portainer (The Web Admin Panel)
  # allows you to Stop/Start/View Logs of your game servers from a browser
  portainer:
    image: portainer/portainer-ce:latest
    ports:
      - "9000:9000"
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
      - portainer_data:/data
    restart: always

volumes:
  portainer_data:
```

---

## Phase 3: GitHub Actions Pipeline (CI/CD)
*Objective: Automate everything. Code Change -> New Server Live.*

Create `.github/workflows/deploy.yml`:

```yaml
name: Deploy Game Server

on:
  push:
    branches: [ "main" ]

env:
  REGISTRY: ghcr.io
  IMAGE_NAME: ${{ github.repository }}

jobs:
  build-and-push:
    runs-on: ubuntu-latest
    permissions:
      contents: read
      packages: write

    steps:
      - name: Checkout repository
        uses: actions/checkout@v3

      - name: Log in to the Docker Registry
        uses: docker/login-action@v2
        with:
          registry: ${{ env.REGISTRY }}
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      - name: Build and push Docker image
        uses: docker/build-push-action@v4
        with:
          context: .
          push: true
          tags: ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:latest
```

---

## Phase 4: Monitoring (The "AAA" Touch)
To monitor server health like a pro admin:

1.  **Prometheus**: Scrapes metrics from your Rust server.
    - *Rust Tip*: Add `metrics-exporter-prometheus` crate to your engine.
2.  **Grafana**: Vizualizes CPU, RAM, and Connected Players.

Add these to your `docker-compose.yml` to get beautiful dashboards of your game's performance.

---

## Summary of Workflow
1. You push code to `main`.
2. GitHub Actions builds the Rust binary & Docker Image.
3. GitHub pushes image to Registry.
4. `Watchtower` on Digital Ocean detects the new image within 30 seconds.
5. `Watchtower` gracefully restarts the `game-world-1` container.
6. Server is updated. You check logs via `Portainer` (http://your-ip:9000).
