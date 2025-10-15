# Changelog

All notable changes to this project will be documented in this file.

## [3.0.0] - 2025-10-15

### Added

- 🚀 **New production binary**: `src/bin/server.rs` replaces `examples/actix_production.rs`, powered by `actix-web 4` & `tracing`
- ⚙️ **Config module**: Runtime behavior (workers, TTLs, cache sizes, prefill dimensions, PNG settings) controlled via environment variables
- 🔁 **Background puzzle generator**: Dedicated async channel + `spawn_blocking` pool, `ExpiringCache<(w,h), PuzzleImages>` with TTL enforcement
- 🧹 **Robust cleanup task**: Runs via `spawn_blocking`, logging removed vs remaining cache entries
- 🧪 **Integration tests**: `tests/cache_tests.rs`, `tests/generator_tests.rs`
- 🛠️ **Benchmark suite refresh**: `bench/run_benchmark.sh` handles Python-less timestamping, graceful curl failures, and wraps wrk runs

### Changed

- 🏗️ Overall crate layout: puzzle logic moved to `puzzle.rs`; generator split into `generator/mod.rs` + `generator/model.rs`
- 🧱 `README.md`: Updated quick-start to `cargo run --bin server --release`, document env vars, refreshed performance snapshot, clarified legacy examples
- 🧾 `CHANGELOG.md`: Reset major version timeline to reflect new architecture
- 🐋 Dockerfile now targets `rust:1.90-slim` and builds the new `server` binary; `docker-compose.prod.yml` aligned with new env settings
- 📊 Bench docs (`bench/README.md`) reference legacy Rust example as optional, highlight shell suite as recommended path

### Removed / Deprecated

- 🗑️ `examples/actix_production.rs` dropped in favor of `src/bin/server.rs`
- ⚠️ `examples/benchmark.rs` marked legacy (still shipped but superseded by `bench/` tooling)

### Notes

- wrk results on 4C/8G hardware average ~800 req/s with PNG `CompressionType::Best`; adjust compression & concurrency for higher throughput
- All tests (`cargo test`) pass, including new integration coverage


## [1.0.0] - Original Release

- Basic slider captcha generation from static images
- Simple verification with actix-web
- Coordinate-based puzzle piece extraction
