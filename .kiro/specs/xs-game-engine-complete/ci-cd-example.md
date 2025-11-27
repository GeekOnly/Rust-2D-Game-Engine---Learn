# CI/CD Configuration Examples

## GitHub Actions Workflows

### 1. Continuous Integration (ci.yml)

```yaml
name: CI

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main, develop ]

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  test:
    name: Test Suite
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
        rust: [stable, nightly]
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@master
        with:
          toolchain: ${{ matrix.rust }}
          components: rustfmt, clippy
      
      - name: Cache cargo registry
        uses: actions/cache@v4
        with:
          path: ~/.cargo/registry
          key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}
      
      - name: Cache cargo index
        uses: actions/cache@v4
        with:
          path: ~/.cargo/git
          key: ${{ runner.os }}-cargo-index-${{ hashFiles('**/Cargo.lock') }}
      
      - name: Cache cargo build
        uses: actions/cache@v4
        with:
          path: target
          key: ${{ runner.os }}-cargo-build-target-${{ hashFiles('**/Cargo.lock') }}
      
      - name: Run tests
        run: cargo test --all-features --workspace
      
      - name: Run property-based tests
        run: cargo test --all-features --workspace -- --ignored
  
  lint:
    name: Linting
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      
      - name: Check formatting
        run: cargo fmt --all -- --check
      
      - name: Run clippy
        run: cargo clippy --all-features --workspace -- -D warnings
  
  coverage:
    name: Code Coverage
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
      
      - name: Install tarpaulin
        run: cargo install cargo-tarpaulin
      
      - name: Generate coverage
        run: cargo tarpaulin --all-features --workspace --out Xml
      
      - name: Upload coverage to Codecov
        uses: codecov/codecov-action@v4
        with:
          files: ./cobertura.xml
  
  security:
    name: Security Audit
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Run cargo audit
        uses: actions-rs/audit-check@v1
        with:
          token: ${{ secrets.GITHUB_TOKEN }}
```

### 2. Continuous Deployment (cd.yml)

```yaml
name: CD

on:
  push:
    tags:
      - 'v*'

jobs:
  build-linux:
    name: Build Linux
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
      
      - name: Build release
        run: cargo build --release --all-features
      
      - name: Strip binary
        run: strip target/release/xs_editor target/release/xs_runtime
      
      - name: Create archive
        run: |
          mkdir -p dist
          tar czf dist/xs-game-engine-linux-x86_64.tar.gz \
            -C target/release xs_editor xs_runtime
      
      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: linux-x86_64
          path: dist/*.tar.gz
  
  build-windows:
    name: Build Windows
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
      
      - name: Build release
        run: cargo build --release --all-features
      
      - name: Create archive
        run: |
          mkdir dist
          Compress-Archive -Path target/release/xs_editor.exe,target/release/xs_runtime.exe `
            -DestinationPath dist/xs-game-engine-windows-x86_64.zip
      
      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: windows-x86_64
          path: dist/*.zip
  
  build-macos:
    name: Build macOS
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: x86_64-apple-darwin,aarch64-apple-darwin
      
      - name: Build x86_64
        run: cargo build --release --all-features --target x86_64-apple-darwin
      
      - name: Build ARM64
        run: cargo build --release --all-features --target aarch64-apple-darwin
      
      - name: Create universal binary
        run: |
          lipo -create \
            target/x86_64-apple-darwin/release/xs_editor \
            target/aarch64-apple-darwin/release/xs_editor \
            -output xs_editor
          lipo -create \
            target/x86_64-apple-darwin/release/xs_runtime \
            target/aarch64-apple-darwin/release/xs_runtime \
            -output xs_runtime
      
      - name: Create archive
        run: |
          mkdir -p dist
          tar czf dist/xs-game-engine-macos-universal.tar.gz xs_editor xs_runtime
      
      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: macos-universal
          path: dist/*.tar.gz
  
  build-android:
    name: Build Android
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Android NDK
        uses: nttld/setup-ndk@v1
        with:
          ndk-version: r25c
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: aarch64-linux-android,armv7-linux-androideabi
      
      - name: Install cargo-apk
        run: cargo install cargo-apk
      
      - name: Build APK
        run: cargo apk build --release
      
      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: android
          path: target/release/apk/*.apk
  
  build-web:
    name: Build WebAssembly
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown
      
      - name: Install wasm-pack
        run: curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
      
      - name: Build WASM
        run: wasm-pack build --target web --release
      
      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: web
          path: pkg/
  
  docker:
    name: Build Docker Images
    runs-on: ubuntu-latest
    needs: [build-linux]
    steps:
      - uses: actions/checkout@v4
      
      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3
      
      - name: Login to Docker Hub
        uses: docker/login-action@v3
        with:
          username: ${{ secrets.DOCKER_USERNAME }}
          password: ${{ secrets.DOCKER_PASSWORD }}
      
      - name: Build and push editor
        uses: docker/build-push-action@v5
        with:
          context: .
          file: ./docker/Dockerfile.editor
          push: true
          tags: |
            xsgameengine/editor:latest
            xsgameengine/editor:${{ github.ref_name }}
      
      - name: Build and push server
        uses: docker/build-push-action@v5
        with:
          context: .
          file: ./docker/Dockerfile.server
          push: true
          tags: |
            xsgameengine/server:latest
            xsgameengine/server:${{ github.ref_name }}
  
  release:
    name: Create Release
    runs-on: ubuntu-latest
    needs: [build-linux, build-windows, build-macos, build-android, build-web]
    steps:
      - uses: actions/checkout@v4
      
      - name: Download all artifacts
        uses: actions/download-artifact@v4
        with:
          path: dist
      
      - name: Create Release
        uses: softprops/action-gh-release@v1
        with:
          files: dist/**/*
          generate_release_notes: true
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

### 3. Benchmarks (benchmarks.yml)

```yaml
name: Benchmarks

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  benchmark:
    name: Run Benchmarks
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
      
      - name: Run benchmarks
        run: cargo bench --all-features --workspace
      
      - name: Store benchmark result
        uses: benchmark-action/github-action-benchmark@v1
        with:
          tool: 'cargo'
          output-file-path: target/criterion/output.txt
          github-token: ${{ secrets.GITHUB_TOKEN }}
          auto-push: true
```

---

## Docker Configurations

### Dockerfile.editor

```dockerfile
# Build stage
FROM rust:1.75-slim as builder

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

# Build release
RUN cargo build --release --bin xs_editor

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy binary
COPY --from=builder /app/target/release/xs_editor /usr/local/bin/

# Create user
RUN useradd -m -u 1000 xsengine
USER xsengine

EXPOSE 8080

CMD ["xs_editor"]
```

### Dockerfile.server

```dockerfile
FROM rust:1.75-slim as builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

RUN cargo build --release --bin xs_server

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/xs_server /usr/local/bin/

RUN useradd -m -u 1000 xsengine
USER xsengine

EXPOSE 7777/udp
EXPOSE 8080/tcp

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:8080/health || exit 1

CMD ["xs_server"]
```

### docker-compose.yml

```yaml
version: '3.8'

services:
  editor:
    build:
      context: .
      dockerfile: docker/Dockerfile.editor
    ports:
      - "8080:8080"
    volumes:
      - ./projects:/app/projects
    environment:
      - RUST_LOG=info
  
  server:
    build:
      context: .
      dockerfile: docker/Dockerfile.server
    ports:
      - "7777:7777/udp"
      - "8081:8080"
    environment:
      - RUST_LOG=info
      - MAX_PLAYERS=100
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 3s
      retries: 3
  
  postgres:
    image: postgres:16
    environment:
      - POSTGRES_DB=xsgameengine
      - POSTGRES_USER=xsengine
      - POSTGRES_PASSWORD=password
    volumes:
      - postgres_data:/var/lib/postgresql/data
    ports:
      - "5432:5432"
  
  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data

volumes:
  postgres_data:
  redis_data:
```

---

## Kubernetes Manifests

### deployment.yaml

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: xs-game-server
  namespace: xs-engine
spec:
  replicas: 3
  selector:
    matchLabels:
      app: xs-game-server
  template:
    metadata:
      labels:
        app: xs-game-server
    spec:
      containers:
      - name: server
        image: xsgameengine/server:latest
        ports:
        - containerPort: 7777
          protocol: UDP
          name: game
        - containerPort: 8080
          protocol: TCP
          name: http
        env:
        - name: RUST_LOG
          value: "info"
        - name: MAX_PLAYERS
          value: "100"
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: xs-secrets
              key: database-url
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "2Gi"
            cpu: "2000m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
```

### service.yaml

```yaml
apiVersion: v1
kind: Service
metadata:
  name: xs-game-server
  namespace: xs-engine
spec:
  type: LoadBalancer
  selector:
    app: xs-game-server
  ports:
  - name: game
    port: 7777
    targetPort: 7777
    protocol: UDP
  - name: http
    port: 8080
    targetPort: 8080
    protocol: TCP
```

### hpa.yaml

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: xs-game-server-hpa
  namespace: xs-engine
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: xs-game-server
  minReplicas: 3
  maxReplicas: 20
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
```

---

This CI/CD configuration provides:
- ✅ Automated testing on multiple platforms
- ✅ Multi-platform builds (Linux, Windows, macOS, Android, Web)
- ✅ Docker image building and publishing
- ✅ Kubernetes deployment with auto-scaling
- ✅ Code coverage and security audits
- ✅ Automated releases with GitHub Releases
