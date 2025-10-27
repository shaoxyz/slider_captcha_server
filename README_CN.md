# 滑块验证码服务器

English | [中文文档](./README_CN.md)

为 [Flutter slider_captcha](https://pub.dev/packages/slider_captcha) 组件设计的高性能滑块验证码生成与验证服务器。

<div align="center">
  <img src="test/example_puzzle.png" alt="验证码示例" width="400"/>
  <p><i>生成的验证码示例</i></p>
</div>

## 🌟 特性

- **🚀 高性能**：Captcha 生成工作在后台线程完成，接口响应极轻量
- **📦 轻量级**：单个验证码约 7KB（较传统方案缩小 98%）
- **🔒 安全**：自动过期缓存（TTL 可配置）+ 后台清理日志可追踪
- **🎨 随机化**：每次生成全新的渐变背景与拼块
- **⚙️ 可配置**：通过环境变量调节线程数、缓存大小、预生成规格
- **🧪 完整压测**： `bench/run_benchmark.sh` + `wrk` + 集成测试

## 📊 性能概览

最新一次在 macOS 4 核 / 8GB 环境下通过 `./bench/run_benchmark.sh`（PNG 压缩为 `CompressionType::Best`）获得的结果：

| 场景           | Requests/s | P50 延迟 | P99 延迟 | Timeout | 说明 |
|----------------|-----------:|---------:|---------:|--------:|------|
| curl 50×100   |      128.70 |   N/A    |   N/A    |       0 | 100 次 curl，总耗时 0.777s，受 CPU 影响明显 |
| wrk 4×100 10s |      692.76 | 162 ms   | 511 ms   |       0 | 4 线程 / 100 连接，缓存命中率稳定 |
| wrk 8×200 30s |      833.81 | 329 ms   | 809 ms   |       0 | 8 线程 / 200 连接，CPU 接近满载 |

> 如需进一步提高 QPS，可降低 PNG 压缩等级（例如 `CompressionType::Default`）、提升 `PUZZLE_GENERATOR_CONCURRENCY`、扩大 `PUZZLE_CACHE_PREFILL`，或采用多副本部署配合负载均衡。

## 🚀 快速开始

### 安装

```bash
git clone https://github.com/BrianTV98/slider_captcha_server
cd slider_captcha_server
```

### 运行开发服务器

```bash
cargo run --bin server --release
```

默认监听 `http://0.0.0.0:8080`

常用环境变量（默认值）：

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
IMMEDIATE_CACHE_CLEANUP=true
RUST_LOG=info
```

示例：

```bash
PUZZLE_PREFILL_DIMENSIONS="500x300,400x240" PUZZLE_GENERATOR_CONCURRENCY=6 \
  cargo run --bin server --release
```

#### 环境变量说明

| 变量名 | 默认值 | 说明 |
|--------|--------|------|
| `SERVER_HOST` | `0.0.0.0` | 服务器监听地址 |
| `SERVER_PORT` | `8080` | 服务器监听端口 |
| `SERVER_WORKERS` | `$(nproc)` | Actix worker 线程数 |
| `PUZZLE_GENERATOR_CONCURRENCY` | `$(nproc)` | 验证码生成并发数 |
| `PUZZLE_CACHE_PREFILL` | `8` | 每个尺寸预生成数量 |
| `PUZZLE_CACHE_MAX` | `32` | 每个尺寸最大缓存数量 |
| `PUZZLE_PREFILL_DIMENSIONS` | `"500x300"` | 预生成尺寸，逗号分隔 |
| `PUZZLE_SOLUTION_TTL_SECS` | `600` | 验证答案缓存时间（秒） |
| `PUZZLE_CACHE_TTL_SECS` | `300` | 验证码图片缓存时间（秒） |
| `CLEANUP_INTERVAL_SECS` | `60` | 缓存清理间隔（秒） |
| `IMMEDIATE_CACHE_CLEANUP` | `true` | 验证成功后是否立即删除缓存 |
| `RUST_LOG` | `info` | 日志级别 |

### API 使用

#### 1. 生成验证码

```bash
curl http://127.0.0.1:8080/puzzle
```

**响应:**

```json
{
  "puzzle_image": "iVBORw0KGgoAAAANSUhEUgAA...",  // base64 编码
  "piece_image": "iVBORw0KGgoAAAANSUhEUgAA...",   // base64 编码
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "y": 0.3367
}
```

**自定义尺寸:**

```bash
curl "http://127.0.0.1:8080/puzzle?w=800&h=400"
```

#### 2. 验证答案

```bash
curl -X POST http://127.0.0.1:8080/puzzle/solution \
  -H "Content-Type: application/json" \
  -d '{"id":"550e8400-e29b-41d4-a716-446655440000","x":0.664}'
```

**成功响应:**

```json
{
  "success": true,
  "message": "验证成功"
}
```

#### 3. 健康检查

```bash
curl http://127.0.0.1:8080/health
```

**响应:**

```json
{
  "status": "ok",
  "prefill_sizes": [[500,300],[400,240]]
}
```

## 🎨 实现原理

### 1. 图片生成

我们不使用静态图片，而是生成随机渐变图片：

```rust
// 生成渐变背景
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

**优势:**

- 高度可压缩（渐变模式）
- 每次唯一（随机颜色）
- 无需存储（按需生成）

### 2. 后台生成器与缓存

- `PUZZLE_GENERATOR_CONCURRENCY` 控制的工作线程使用 `spawn_blocking` 生成验证码并编码 PNG/base64。
- `/puzzle` 处理逻辑仅从 `ExpiringCache<(w,h), PuzzleImages>` 弹出已有数据；若为空则返回 503 并异步排队生成。
- 缓存采用 TTL + LRU 队列，`cleanup()` 会定期统计并清理过期条目。
- 通过环境变量可调整 TTL、预生成数量、缓存容量等。

### 3. 并发结构

- `DashMap` 用于存放验证码答案和各尺寸的缓存队列；无全局锁瓶颈。
- 所有生成操作都在后台线程执行，Actix worker 仅负责 JSON 序列化和响应。

### 4. PNG 优化

```rust
PngEncoder::new_with_quality(
    buffer,
    CompressionType::Best,   // 最高压缩
    FilterType::Sub,          // 最适合渐变
)
```

结果：相比基于照片的验证码，**体积减少 98%+**

## 🧪 性能测试

### 压测工具

| 工具       | 位置                     | 说明                            |
| ---------- | ------------------------ | ------------------------------- |
| Shell 脚本 | `bench/run_benchmark.sh` | 推荐使用，包含 curl + wrk 流程  |
| wrk 脚本   | `bench/wrk_test.lua`     | 被 Shell 脚本调用，也可单独使用 |

```bash
./bench/run_benchmark.sh
wrk -t4 -c100 -d10s --latency -s bench/wrk_test.lua http://127.0.0.1:8080/puzzle
```

## 📁 项目结构

```
slider_captcha_server/
├── src/
│   ├── bin/server.rs       # 生产入口
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
└── docker-compose*.yml
```

## 🎯 客户端集成

### Flutter 示例

```dart
import 'package:slider_captcha/slider_captcha.dart';

SliderCaptchaClient(
  provider: SliderCaptchaClientProvider(
    base64Image,   // 从我们的API获取: puzzle_image
    base64Piece,   // 从我们的API获取: piece_image
    coordinateY,   // 从我们的API获取: y
  ),
  onConfirm: (value) async {
    // 向服务器验证
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

更多详情请参考 [Flutter slider_captcha 组件](https://pub.dev/packages/slider_captcha)。

## 🚢 部署

仓库提供了基于 `rust:1.90-slim` 的多阶段 Dockerfile，默认构建 `server` 二进制。

```bash
docker compose build --no-cache
docker compose up -d
```

可在 `docker-compose.prod.yml` 中通过环境变量调整线程数、缓存容量等参数。

### systemd 服务

创建 `/etc/systemd/system/slider-captcha.service`:

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

启用并启动:

```bash
sudo systemctl enable slider-captcha
sudo systemctl start slider-captcha
```

## 📝 API 文档

### GET /puzzle

生成新的验证码。

**查询参数:**

- `w` (可选): 宽度像素 (默认: 500)
- `h` (可选): 高度像素 (默认: 300)

**响应:**

```typescript
{
  puzzle_image: string,  // base64 PNG
  piece_image: string,   // base64 PNG
  id: string,            // UUID
  y: number              // 相对Y位置 (0.0-1.0)
}
```

### POST /puzzle/solution

验证验证码答案。

**请求体:**

```typescript
{
  id: string,    // 从生成接口获取
  x: number      // 用户滑块位置 (0.0-1.0)
}
```

**成功响应:**

```typescript
{
  success: true,
  message: string
}
```

**错误响应:**

```typescript
{
  success: false,
  error: string
}
```

### GET /health

检查服务器健康状态。

**响应:**

```typescript
{
  status: "healthy",
  cache_size: number,
  uptime: string
}
```

## 🙏 致谢

本项目为 [Flutter slider_captcha](https://pub.dev/packages/slider_captcha) 组件设计，基于 [@BrianTV98](https://github.com/BrianTV98) 的原始项目 [slider_captcha_server](https://github.com/BrianTV98/slider_captcha_server) 开发。

**使用 Claude AI 进行的增强:**

- ✨ 随机渐变图片生成（替代静态图片）
- ⚡ 使用 DashMap 的高性能缓存
- 🔒 自动过期缓存机制
- 📦 98%+ 图片大小优化
- 🚀 生产级部署方案
- 🧪 完整的性能测试套件

## 📄 许可证

GPL-3.0 许可证 - 详见 [LICENSE](LICENSE)

## 🤝 贡献

欢迎贡献！请随时提交 Pull Request。

## 📧 支持

如果遇到任何问题或有疑问，请[提交 issue](https://github.com/BrianTV98/slider_captcha_server/issues)。

---

用 ❤️ 制作，由 Claude AI 优化

