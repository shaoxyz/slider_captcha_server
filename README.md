# Slider Captcha Server

[中文文档](./README_CN.md) | English

A high-performance slider captcha puzzle creation and verification server designed for the [Flutter slider_captcha](https://pub.dev/packages/slider_captcha) component.

<div align="center">
  <img src="test/example_puzzle.png" alt="Puzzle Example" width="400"/>
  <p><i>Example of generated captcha puzzle</i></p>
</div>

## 🌟 Features

- **🚀 High Performance**: `/puzzle` handler serves pre-generated puzzles from memory
- **📦 Lightweight**: ~7KB per captcha (98% smaller than photo-based captchas)
- **🔒 Secure**: Auto-expiring cache (configurable TTL) with background cleanup
- **🎨 Randomized**: Unique gradient-based images every request
- **⚙️ Configurable**: Tune concurrency, cache size, prefill dimensions via env vars
- **🧪 Benchmarked**: Shell suite (`bench/run_benchmark.sh`) & wrk integration

## 📊 Performance Snapshot

Latest local run (macOS 4C/8G, PNG compression `CompressionType::Best`) via `./bench/run_benchmark.sh`:

| Scenario       | Requests/s | P50 Latency | P99 Latency | Timeouts | Notes                                          |
| -------------- | ---------: | ----------: | ----------: | -------: | ---------------------------------------------- |
| curl 50×100   |      128.70 |        N/A  |        N/A  |        0 | 100 curl 请求耗时 0.777s，总体受 CPU 影响 |
| wrk 4×100 10s |      692.76 |      162 ms |      511 ms |        0 | 4 线程 / 100 连接，缓存命中率稳定          |
| wrk 8×200 30s |      833.81 |      329 ms |      809 ms |        0 | 8 线程 / 200 连接，CPU 接近满载            |

> 若要维持更高 QPS，可降低 PNG 压缩等级（如 `CompressionType::Default`）、提升 `PUZZLE_GENERATOR_CONCURRENCY`、扩大 `PUZZLE_CACHE_PREFILL`，或部署多副本并使用负载均衡。

## 🚀 Quick Start

### Installation

```bash
git clone https://github.com/BrianTV98/slider_captcha_server
cd slider_captcha_server
```

### Run Development Server

```bash
cargo run --bin server --release
```

Server listens on `http://0.0.0.0:8080`.

Configurable via environment variables:

```bash
SERVER_HOST=0.0.0.0
SERVER_PORT=8080
SERVER_WORKERS=$(nproc)
PUZZLE_GENERATOR_CONCURRENCY=$(nproc)
PUZZLE_CACHE_PREFILL=8
PUZZLE_CACHE_MAX=32
PUZZLE_PREFILL_DIMENSIONS="500x300"
PUZZLE_SOLUTION_TTL_SECS=600
PUZZLE_CACHE_TTL_SECS=300
CLEANUP_INTERVAL_SECS=60
RUST_LOG=info
```

Example:

```bash
PUZZLE_PREFILL_DIMENSIONS="500x300,400x240" PUZZLE_GENERATOR_CONCURRENCY=6 \
  cargo run --bin server --release
```

### API Usage

#### 1. Generate Captcha

```bash
curl http://127.0.0.1:8080/puzzle
```

**Response:**

```json
{
  "puzzle_image": "iVBORw0KGgoAAAANSUhEUgAA...",  // base64 encoded
  "piece_image": "iVBORw0KGgoAAAANSUhEUgAA...",   // base64 encoded
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "y": 0.3367
}
```

**With custom dimensions:**

```bash
curl "http://127.0.0.1:8080/puzzle?w=800&h=400"
```

#### 2. Verify Solution

```bash
curl -X POST http://127.0.0.1:8080/puzzle/solution \
  -H "Content-Type: application/json" \
  -d '{"id":"550e8400-e29b-41d4-a716-446655440000","x":0.664}'
```

**Success Response:**

```json
{
  "success": true,
  "message": "Verification successful"
}
```

#### 3. Health Check

```bash
curl http://127.0.0.1:8080/health
```

**Response:**

```json
{
  "status": "ok",
  "prefill_sizes": [[500,300],[400,240]]
}
```

## 🎨 How It Works

### 1. Image Generation

Instead of using static images, we generate randomized gradient images:

```rust
// Generate gradient background
for y in 0..height {
    for x in 0..width {
        let ratio = x as f32 / width as f32;
        let r = (r1 * (1.0 - ratio) + r2 * ratio) as u8;
        let g = (g1 * (1.0 - ratio) + g2 * ratio) as u8;
        let b = (b1 * (1.0 - ratio) + b2 * ratio) as u8;
        image.put_pixel(x, y, Rgba([r, g, b, 255]));
    }
}
```

**Benefits:**

- Highly compressible (gradient patterns)
- Unique every time (random colors)
- No storage needed (generated on-demand)

### 2. Background Generator & Cache

- Dedicated worker tasks (`PUZZLE_GENERATOR_CONCURRENCY`) run `SliderPuzzle::from_dimensions` + PNG/base64 inside `spawn_blocking`.
- `/puzzle` handler just pops a cached item; misses queue a generation request and respond with 503 instead of blocking.
- `ExpiringCache<(w,h), PuzzleImages>` keeps per-dimension queues, enforcing TTL on pop and during cleanup.
- A periodic job (`CLEANUP_INTERVAL_SECS`) runs via `spawn_blocking` and logs removed/remaining entries.

### 3. Lock-Free Concurrency

- `DashMap` backs solution storage and the multi-queue cache; no coarse-grained mutexes.
- Cache operations are O(1) amortized, limited only by per-key queue length.

### 4. PNG Optimization

```rust
PngEncoder::new_with_quality(
    buffer,
    CompressionType::Best,   // Maximum compression
    FilterType::Sub,          // Best for gradients
)
```

Result: **98%+ size reduction** compared to photo-based captchas

## 🧪 Performance Testing

### Benchmark Tooling

| Tool         | Location                 | Status                           |
| ------------ | ------------------------ | -------------------------------- |
| Shell suite  | `bench/run_benchmark.sh` | ✅ Recommended                   |
| wrk script   | `bench/wrk_test.lua`     | ✅ Used by shell suite           |

Usage:

```bash
./bench/run_benchmark.sh
wrk -t4 -c100 -d10s --latency -s bench/wrk_test.lua http://127.0.0.1:8080/puzzle
```

## 📁 Project Structure

```
slider_captcha_server/
├── src/
│   ├── bin/server.rs       # Production entrypoint
│   ├── cache.rs
│   ├── config.rs
│   ├── generator/
│   ├── puzzle.rs
│   └── lib.rs
├── bench/
│   ├── run_benchmark.sh
│   └── wrk_test.lua
├── examples/
│   └── generate_random.rs
├── tests/
│   ├── cache_tests.rs
│   └── generator_tests.rs
├── docker-compose*.yml
└── README.md
```

## 🎯 Client Integration

### Flutter Example

```dart
import 'package:slider_captcha/slider_captcha.dart';

SliderCaptchaClient(
  provider: SliderCaptchaClientProvider(
    base64Image,   // From our API: puzzle_image
    base64Piece,   // From our API: piece_image
    coordinateY,   // From our API: y
  ),
  onConfirm: (value) async {
    // Verify with our server
    final response = await http.post(
      Uri.parse('http://your-server:8080/puzzle/solution'),
      body: json.encode({
        'id': captchaId,
        'x': value,
      }),
    );
    return response.statusCode == 200;
  },
)
```

For more details, see the [Flutter slider_captcha package](https://pub.dev/packages/slider_captcha).

## 🚢 Deployment

### Docker

Multi-stage Dockerfile targets `rust:1.90-slim` and the `server` binary:

```bash
docker compose build --no-cache
docker compose up -d
```

Tune via `docker-compose.prod.yml` by setting env vars (workers, cache sizes, prefill dimensions, etc.).

### systemd Service

Create `/etc/systemd/system/slider-captcha.service`:

```ini
[Unit]
Description=Slider Captcha Server
After=network.target

[Service]
Type=simple
User=www-data
WorkingDirectory=/opt/slider_captcha_server
ExecStart=/opt/slider_captcha_server/target/release/server
Restart=always

[Install]
WantedBy=multi-user.target
```

Enable and start:

```bash
sudo systemctl enable slider-captcha
sudo systemctl start slider-captcha
```

## 📝 API Reference

### GET /puzzle

Generate a new captcha puzzle.

**Query Parameters:**

- `w` (optional): Width in pixels (default: 500)
- `h` (optional): Height in pixels (default: 300)

**Response:**

```typescript
{
  puzzle_image: string,  // base64 PNG
  piece_image: string,   // base64 PNG
  id: string,            // UUID
  y: number              // Relative Y position (0.0-1.0)
}
```

### POST /puzzle/solution

Verify captcha solution.

**Request Body:**

```typescript
{
  id: string,    // From generation response
  x: number      // User's slider position (0.0-1.0)
}
```

**Response (Success):**

```typescript
{
  success: true,
  message: string
}
```

**Response (Error):**

```typescript
{
  success: false,
  error: string
}
```

### GET /health

Check server health.

**Response:**

```typescript
{
  status: "healthy",
  cache_size: number,
  uptime: string
}
```

## 🙏 Acknowledgments

This project is designed to work with the [Flutter slider_captcha](https://pub.dev/packages/slider_captcha) component and is based on the original [slider_captcha_server](https://github.com/BrianTV98/slider_captcha_server) by [@BrianTV98](https://github.com/BrianTV98).

**Enhancements made with Claude AI:**

- ✨ Random gradient image generation (replaced static images)
- ⚡ High-performance caching with DashMap
- 🔒 Auto-expiring cache mechanism
- 📦 98%+ image size optimization
- 🚀 Production-ready deployment
- 🧪 Complete performance testing suite

## 📄 License

GPL-3.0 License - see [LICENSE](LICENSE) for details.

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📧 Support

If you encounter any issues or have questions, please [open an issue](https://github.com/BrianTV98/slider_captcha_server/issues).

---

Made with ❤️ and optimized with Claude AI

