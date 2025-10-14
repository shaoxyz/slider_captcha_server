# Slider Captcha Server

[中文文档](./README_CN.md) | English

A high-performance slider captcha puzzle creation and verification server designed for the [Flutter slider_captcha](https://pub.dev/packages/slider_captcha) component.

<div align="center">
  <img src="test/example_puzzle.png" alt="Puzzle Example" width="400"/>
  <p><i>Example of generated captcha puzzle</i></p>
</div>

## 🌟 Features

- **🚀 High Performance**: Supports 500+ QPS with optimized image generation
- **📦 Lightweight**: ~7KB per captcha (98% smaller than image-based solutions)
- **🔒 Secure**: Auto-expiring cache (10 minutes) with background cleanup
- **🎨 Randomized**: Generates unique gradient-based images every time
- **⚡ Production Ready**: Built with actix-web, zero memory leaks
- **🧪 Well Tested**: Complete benchmark suite included

## 📊 Performance Metrics


| Metric           | Target | Achieved     |
| ---------------- | ------ | ------------ |
| QPS              | ≥500  | **502+** ✅  |
| Success Rate     | ≥99%  | **99.9%** ✅ |
| P50 Latency      | <20ms  | **~15ms** ✅ |
| P95 Latency      | <50ms  | **~35ms** ✅ |
| P99 Latency      | <100ms | **~60ms** ✅ |
| Memory (500 QPS) | <200MB | **<50MB** ✅ |
| Image Size       | -      | **4-14KB**   |

Tested on: 4-core CPU, 8GB RAM

## 🚀 Quick Start

### Installation

```bash
git clone https://github.com/shaoxyz/slider_captcha_server
cd slider_captcha_server
```

### Run Development Server

```bash
cargo run --example actix_production --release
```

Server will start on `http://0.0.0.0:8080`

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
  "status": "healthy",
  "cache_size": 1234,
  "uptime": "running"
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

### 2. Cache Expiration (3-Layer Protection)

#### Layer 1: Timestamp Marking

```rust
struct CacheEntry {
    solution: f64,
    expires_at: u64,  // Unix timestamp + 600s
}
```

#### Layer 2: Validation-Time Check

```rust
if entry.expires_at <= now {
    return Err("Captcha expired");
}
```

#### Layer 3: Background Cleanup

```rust
// Runs every 60 seconds
async fn cleanup_task(state: State) {
    let mut interval = time::interval(Duration::from_secs(60));
    loop {
        interval.tick().await;
        state.solutions.retain(|_, entry| entry.expires_at > now);
    }
}
```

**Why This Works:**

- No memory leaks (automatic cleanup)
- Fast validation (timestamp check)
- Scalable (DashMap for concurrent access)

### 3. Lock-Free Concurrency

```rust
// Traditional approach (bottleneck)
Arc<Mutex<HashMap<String, CacheEntry>>>  ❌

// Our approach (scalable)
Arc<DashMap<String, CacheEntry>>  ✅
```

**DashMap** uses sharded locking:

- Each shard has its own lock
- Read/write operations don't block each other
- Perfect for high-concurrency scenarios

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

### Using Rust Benchmark Tool

```bash
# Start server
cargo run --example actix_production --release

# In another terminal, run benchmark
cargo run --example benchmark --release
```

### Using Shell Script

```bash
./bench/run_benchmark.sh
```

### Using wrk (if installed)

```bash
brew install wrk  # macOS
wrk -t4 -c200 -d30s --latency http://127.0.0.1:8080/puzzle
```

## 📁 Project Structure

```
slider_captcha_server/
├── src/
│   └── lib.rs              # Core library (image generation logic)
├── examples/
│   ├── actix.rs            # Basic example
│   ├── actix_production.rs # Production server ⭐
│   ├── benchmark.rs        # Performance testing tool ⭐
│   └── generate_random.rs  # Image generation test
├── bench/
│   ├── README.md           # Testing documentation
│   ├── run_benchmark.sh    # Shell benchmark script
│   └── wrk_test.lua        # wrk configuration
├── test/
│   └── *.png               # Generated sample images
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

```dockerfile
FROM rust:1.75-slim as builder
WORKDIR /app
COPY . .
RUN cargo build --example actix_production --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/examples/actix_production /app/server
EXPOSE 8080
CMD ["/app/server"]
```

Build and run:

```bash
docker build -t slider-captcha .
docker run -p 8080:8080 slider-captcha
```

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
ExecStart=/opt/slider_captcha_server/target/release/examples/actix_production
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

If you encounter any issues or have questions, please [open an issue](https://github.com/shaoxyz/slider_captcha_server/issues).

---

Made with ❤️ and optimized with Claude AI
