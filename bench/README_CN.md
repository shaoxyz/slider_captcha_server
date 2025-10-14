# 性能测试指南

[English](./README.md) | 中文文档

本目录包含用于测试 slider_captcha_server 性能的各种工具和脚本。

## 📋 测试工具清单

### 1. Rust 压测工具 (推荐)
**文件**: `examples/benchmark.rs`

纯Rust实现，无需外部依赖，提供详细的性能指标。

```bash
# 首先启动服务（生产模式）
cargo run --example actix_production --release

# 在另一个终端运行压测
cargo run --example benchmark --release
```

**测试项目**:
- ✅ 预热测试 (10个请求)
- ✅ 持续负载测试 (50/100/200 QPS)
- ✅ 峰值负载测试 (500 QPS)
- ✅ 压力测试 (1000 QPS)

### 2. Shell 压测脚本
**文件**: `bench/run_benchmark.sh`

使用curl和wrk的组合测试脚本。

```bash
# 给脚本添加执行权限
chmod +x bench/run_benchmark.sh

# 运行测试
./bench/run_benchmark.sh
```

**功能**:
- 单次请求测试
- 并发测试 (50并发)
- wrk压测 (如果已安装)
- 高负载测试 (500 QPS目标)
- 完整流程测试 (生成+验证)

### 3. wrk 专用脚本
**文件**: `bench/wrk_test.lua`

wrk性能测试的Lua脚本，提供详细的统计报告。

**安装 wrk**:
```bash
# macOS
brew install wrk

# Ubuntu/Debian
sudo apt-get install wrk

# 从源码编译
git clone https://github.com/wg/wrk.git
cd wrk
make
sudo cp wrk /usr/local/bin
```

**使用方法**:
```bash
# 基础测试 - 100并发，持续10秒
wrk -t4 -c100 -d10s --latency -s bench/wrk_test.lua http://127.0.0.1:8080/puzzle

# 高并发测试 - 200并发，持续30秒
wrk -t8 -c200 -d30s --latency -s bench/wrk_test.lua http://127.0.0.1:8080/puzzle

# 500 QPS目标测试 - 400并发，持续60秒
wrk -t8 -c400 -d60s --latency -s bench/wrk_test.lua http://127.0.0.1:8080/puzzle
```

## 🎯 性能目标

### 目标指标 (500 QPS)

| 指标 | 目标值 | 说明 |
|------|--------|------|
| QPS | ≥ 500 | 每秒请求数 |
| 成功率 | ≥ 99% | 请求成功率 |
| P50延迟 | < 20ms | 50%请求延迟 |
| P95延迟 | < 50ms | 95%请求延迟 |
| P99延迟 | < 100ms | 99%请求延迟 |
| 内存占用 | < 200MB | 稳定运行内存 |

### 预期性能表现

基于优化后的实现：

**硬件配置**: 4核CPU, 8GB RAM

| 并发数 | QPS | 平均延迟 | P99延迟 | 成功率 |
|--------|-----|----------|---------|--------|
| 50 | ~200 | 5-10ms | 15ms | 100% |
| 100 | ~400 | 10-15ms | 25ms | 100% |
| 200 | ~600 | 15-25ms | 50ms | 99.9% |
| 400 | ~800 | 25-40ms | 100ms | 99% |

## 📊 性能测试步骤

### Step 1: 编译 Release 版本

```bash
# 编译生产版本
cargo build --example actix_production --release
```

### Step 2: 启动服务

```bash
# 启动生产服务器
cargo run --example actix_production --release
```

服务将在 `http://0.0.0.0:8080` 上监听。

### Step 3: 运行性能测试

**选项A: 使用Rust压测工具 (推荐)**
```bash
cargo run --example benchmark --release
```

**选项B: 使用Shell脚本**
```bash
./bench/run_benchmark.sh
```

**选项C: 使用wrk**
```bash
wrk -t4 -c200 -d30s --latency -s bench/wrk_test.lua http://127.0.0.1:8080/puzzle
```

### Step 4: 监控服务状态

**方法1: 健康检查接口**
```bash
watch -n 1 'curl -s http://127.0.0.1:8080/health | jq'
```

**方法2: 系统资源监控**
```bash
# CPU和内存使用
watch -n 1 'ps aux | grep actix_production | grep -v grep'

# 详细资源监控 (需要安装htop)
htop -p $(pgrep actix_production)
```

**方法3: 网络连接监控**
```bash
# 查看当前连接数
watch -n 1 'netstat -an | grep :8080 | wc -l'
```

## 🔧 性能调优建议

### 系统层面

1. **文件描述符限制**
```bash
# 查看当前限制
ulimit -n

# 临时提高限制
ulimit -n 65535

# 永久设置 (编辑 /etc/security/limits.conf)
* soft nofile 65535
* hard nofile 65535
```

2. **TCP调优** (Linux)
```bash
# 调整TCP参数
sudo sysctl -w net.ipv4.tcp_tw_reuse=1
sudo sysctl -w net.ipv4.tcp_fin_timeout=30
sudo sysctl -w net.core.somaxconn=4096
```

### 应用层面

1. **Worker数量调整**
   - 修改 `actix_production.rs` 中的 `.workers(4)`
   - 推荐值: CPU核心数 * 2

2. **缓存清理频率**
   - 当前: 每60秒清理一次
   - 高QPS场景: 可以调整到120秒或更长

3. **图片压缩级别**
   - 当前: `CompressionType::Best` (最高压缩)
   - 如果CPU成为瓶颈，可以降低到 `CompressionType::Fast`

## 📈 结果分析

### 成功标准

✅ **通过**: 
- QPS达到目标的95%以上
- 成功率 ≥ 99%
- P99延迟 < 100ms

⚠️ **警告**:
- QPS达到目标的80-95%
- 成功率 95-99%
- P99延迟 100-200ms

❌ **失败**:
- QPS低于目标的80%
- 成功率 < 95%
- P99延迟 > 200ms

### 常见问题排查

**问题1: QPS上不去**
- 检查CPU使用率是否达到100%
- 检查网络带宽是否饱和
- 增加worker数量
- 降低PNG压缩级别

**问题2: 延迟高**
- 检查是否在请求路径上有不必要的同步操作
- 检查DashMap是否有锁竞争
- 考虑使用连接池

**问题3: 内存占用过高**
- 检查缓存大小 (访问 /health)
- 调整缓存过期时间
- 检查是否有内存泄漏

## 🚀 压测最佳实践

1. **逐步增加负载**: 从低QPS开始，逐步提升
2. **长时间运行**: 至少运行30分钟以观察内存泄漏
3. **真实数据**: 使用实际的请求参数分布
4. **监控完整**: 同时监控CPU、内存、网络、磁盘IO
5. **多次测试**: 至少运行3次取平均值

## 📝 测试报告模板

```
## 性能测试报告

**测试时间**: 2024-XX-XX
**测试环境**: 
- CPU: X核
- 内存: XGB
- 系统: macOS/Linux
- Rust版本: 1.XX

**测试结果**:

| 并发数 | QPS | P50延迟 | P95延迟 | P99延迟 | 成功率 |
|--------|-----|---------|---------|---------|--------|
| 100    | XXX | XXms    | XXms    | XXms    | XX%    |
| 200    | XXX | XXms    | XXms    | XXms    | XX%    |
| 400    | XXX | XXms    | XXms    | XXms    | XX%    |

**资源使用**:
- CPU峰值: XX%
- 内存峰值: XXMB
- 网络带宽: XX Mbps

**结论**: 
- [✅/⚠️/❌] 达到/接近/未达到 500 QPS目标
- 建议: ...
```

## 🛠️ 故障排除

如果测试失败，按以下顺序检查：

1. ✅ 服务是否在运行
2. ✅ 是否使用了 --release 模式
3. ✅ 防火墙是否阻止连接
4. ✅ 端口是否正确 (8080)
5. ✅ 系统资源是否充足
6. ✅ 网络延迟是否正常

## 📚 参考资料

- [wrk文档](https://github.com/wg/wrk)
- [actix-web性能调优](https://actix.rs/docs/performance)
- [DashMap并发性能](https://github.com/xacrimon/dashmap)

