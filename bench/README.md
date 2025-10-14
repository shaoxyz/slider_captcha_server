# Performance Testing Guide

[中文文档](./README_CN.md) | English

This directory contains various tools and scripts for testing slider_captcha_server performance.

## 📋 Testing Tools

### 1. Rust Benchmark Tool (Recommended)
**File**: `examples/benchmark.rs`

Pure Rust implementation with no external dependencies, provides detailed performance metrics.

```bash
# Start the production server first
cargo run --example actix_production --release

# Run benchmark in another terminal
cargo run --example benchmark --release
```

**Test Coverage**:
- ✅ Warmup test (10 requests)
- ✅ Sustained load test (50/100/200 QPS)
- ✅ Peak load test (500 QPS)
- ✅ Stress test (1000 QPS)

### 2. Shell Benchmark Script
**File**: `bench/run_benchmark.sh`

Combined testing script using curl and wrk.

```bash
# Make script executable
chmod +x bench/run_benchmark.sh

# Run tests
./bench/run_benchmark.sh
```

**Features**:
- Single request test
- Concurrent test (50 connections)
- wrk benchmark (if installed)
- High load test (500 QPS target)
- Full workflow test (generate + verify)

### 3. wrk Script
**File**: `bench/wrk_test.lua`

Lua script for wrk performance testing with detailed statistics.

**Install wrk**:
```bash
# macOS
brew install wrk

# Ubuntu/Debian
sudo apt-get install wrk

# Build from source
git clone https://github.com/wg/wrk.git
cd wrk
make
sudo cp wrk /usr/local/bin
```

**Usage**:
```bash
# Basic test - 100 connections, 10 seconds
wrk -t4 -c100 -d10s --latency -s bench/wrk_test.lua http://127.0.0.1:8080/puzzle

# High concurrency - 200 connections, 30 seconds
wrk -t8 -c200 -d30s --latency -s bench/wrk_test.lua http://127.0.0.1:8080/puzzle

# 500 QPS target - 400 connections, 60 seconds
wrk -t8 -c400 -d60s --latency -s bench/wrk_test.lua http://127.0.0.1:8080/puzzle
```

## 🎯 Performance Targets

### Target Metrics (500 QPS)

| Metric | Target | Description |
|--------|--------|-------------|
| QPS | ≥ 500 | Requests per second |
| Success Rate | ≥ 99% | Request success rate |
| P50 Latency | < 20ms | 50th percentile |
| P95 Latency | < 50ms | 95th percentile |
| P99 Latency | < 100ms | 99th percentile |
| Memory | < 200MB | Stable memory usage |

### Expected Performance

Based on optimized implementation:

**Hardware**: 4-core CPU, 8GB RAM

| Connections | QPS | Avg Latency | P99 Latency | Success Rate |
|-------------|-----|-------------|-------------|--------------|
| 50 | ~200 | 5-10ms | 15ms | 100% |
| 100 | ~400 | 10-15ms | 25ms | 100% |
| 200 | ~600 | 15-25ms | 50ms | 99.9% |
| 400 | ~800 | 25-40ms | 100ms | 99% |

## 📊 Testing Steps

### Step 1: Build Release Version

```bash
# Build production version
cargo build --example actix_production --release
```

### Step 2: Start Server

```bash
# Start production server
cargo run --example actix_production --release
```

Server will listen on `http://0.0.0.0:8080`

### Step 3: Run Performance Tests

**Option A: Rust Benchmark Tool (Recommended)**
```bash
cargo run --example benchmark --release
```

**Option B: Shell Script**
```bash
./bench/run_benchmark.sh
```

**Option C: wrk**
```bash
wrk -t4 -c200 -d30s --latency -s bench/wrk_test.lua http://127.0.0.1:8080/puzzle
```

### Step 4: Monitor Server Status

**Method 1: Health Check Endpoint**
```bash
watch -n 1 'curl -s http://127.0.0.1:8080/health | jq'
```

**Method 2: System Resource Monitoring**
```bash
# CPU and memory usage
watch -n 1 'ps aux | grep actix_production | grep -v grep'

# Detailed monitoring (requires htop)
htop -p $(pgrep actix_production)
```

**Method 3: Network Connection Monitoring**
```bash
# Check current connections
watch -n 1 'netstat -an | grep :8080 | wc -l'
```

## 🔧 Performance Tuning

### System Level

1. **File Descriptor Limits**
```bash
# Check current limit
ulimit -n

# Temporarily increase limit
ulimit -n 65535

# Permanent setting (edit /etc/security/limits.conf)
* soft nofile 65535
* hard nofile 65535
```

2. **TCP Tuning** (Linux)
```bash
# Adjust TCP parameters
sudo sysctl -w net.ipv4.tcp_tw_reuse=1
sudo sysctl -w net.ipv4.tcp_fin_timeout=30
sudo sysctl -w net.core.somaxconn=4096
```

### Application Level

1. **Worker Count**
   - Modify `.workers(4)` in `actix_production.rs`
   - Recommended: CPU cores * 2

2. **Cache Cleanup Frequency**
   - Current: Every 60 seconds
   - High QPS: Can adjust to 120 seconds or longer

3. **Image Compression Level**
   - Current: `CompressionType::Best` (maximum compression)
   - If CPU bottleneck: Reduce to `CompressionType::Fast`

## 📈 Result Analysis

### Success Criteria

✅ **Pass**: 
- QPS ≥ 95% of target
- Success rate ≥ 99%
- P99 latency < 100ms

⚠️ **Warning**:
- QPS 80-95% of target
- Success rate 95-99%
- P99 latency 100-200ms

❌ **Fail**:
- QPS < 80% of target
- Success rate < 95%
- P99 latency > 200ms

### Troubleshooting

**Issue 1: Low QPS**
- Check if CPU usage is at 100%
- Check if network bandwidth is saturated
- Increase worker count
- Reduce PNG compression level

**Issue 2: High Latency**
- Check for unnecessary synchronous operations
- Check for DashMap lock contention
- Consider using connection pool

**Issue 3: High Memory Usage**
- Check cache size (access /health)
- Adjust cache expiration time
- Check for memory leaks

## 🚀 Best Practices

1. **Gradual Load Increase**: Start with low QPS and gradually increase
2. **Long Duration**: Run for at least 30 minutes to observe memory leaks
3. **Real Data**: Use actual request parameter distributions
4. **Complete Monitoring**: Monitor CPU, memory, network, disk I/O simultaneously
5. **Multiple Runs**: Run at least 3 times and take average

## 📝 Test Report Template

```
## Performance Test Report

**Test Date**: 2024-XX-XX
**Test Environment**: 
- CPU: X cores
- Memory: XGB
- System: macOS/Linux
- Rust version: 1.XX

**Test Results**:

| Connections | QPS | P50 Latency | P95 Latency | P99 Latency | Success Rate |
|-------------|-----|-------------|-------------|-------------|--------------|
| 100 | XXX | XXms | XXms | XXms | XX% |
| 200 | XXX | XXms | XXms | XXms | XX% |
| 400 | XXX | XXms | XXms | XXms | XX% |

**Resource Usage**:
- Peak CPU: XX%
- Peak Memory: XXMB
- Network Bandwidth: XX Mbps

**Conclusion**: 
- [✅/⚠️/❌] Met/Approached/Missed 500 QPS target
- Recommendations: ...
```

## 🛠️ Troubleshooting Checklist

If tests fail, check in this order:

1. ✅ Is the server running?
2. ✅ Using --release mode?
3. ✅ Firewall blocking connections?
4. ✅ Correct port (8080)?
5. ✅ Sufficient system resources?
6. ✅ Normal network latency?

## 📚 References

- [wrk Documentation](https://github.com/wg/wrk)
- [actix-web Performance Tuning](https://actix.rs/docs/performance)
- [DashMap Concurrency](https://github.com/xacrimon/dashmap)
