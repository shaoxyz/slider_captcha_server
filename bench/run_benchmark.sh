#!/bin/bash

# 性能测试脚本
# 测试 slider_captcha_server 的性能表现

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 配置
HOST=${HOST:-"http://127.0.0.1:8080"}
PUZZLE_URL="${HOST}/puzzle"
VERIFY_URL="${HOST}/puzzle/solution"

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  Slider Captcha 性能测试套件${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# 检查服务是否运行
check_service() {
    echo -e "${YELLOW}检查服务状态...${NC}"
    if curl -s "${HOST}/health" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ 服务正在运行${NC}"
    else
        echo -e "${RED}✗ 服务未运行，请先启动服务：${NC}"
        echo "  cargo run --example actix_production --release"
        exit 1
    fi
    echo ""
}

# 单次测试
test_single_request() {
    echo -e "${YELLOW}[1/5] 单次请求测试${NC}"
    echo "发送单个请求并测量响应时间..."
    
    START=$(date +%s%3N)
    RESPONSE=$(curl -s -w "\n%{http_code}\n%{time_total}" "${PUZZLE_URL}")
    END=$(date +%s%3N)
    
    STATUS=$(echo "$RESPONSE" | tail -n 2 | head -n 1)
    TIME=$(echo "$RESPONSE" | tail -n 1)
    
    if [ "$STATUS" = "200" ]; then
        echo -e "${GREEN}✓ 请求成功${NC}"
        echo "  HTTP状态码: ${STATUS}"
        echo "  响应时间: ${TIME}s"
    else
        echo -e "${RED}✗ 请求失败，状态码: ${STATUS}${NC}"
    fi
    echo ""
}

# 并发测试 - 使用curl
test_concurrent_curl() {
    echo -e "${YELLOW}[2/5] 并发测试 (50并发 x 100请求)${NC}"
    
    SUCCESS=0
    FAILED=0
    TOTAL=100
    CONCURRENT=50
    
    echo "开始测试..."
    START=$(date +%s%3N)
    
    for i in $(seq 1 $TOTAL); do
        (
            STATUS=$(curl -s -o /dev/null -w "%{http_code}" "${PUZZLE_URL}")
            if [ "$STATUS" = "200" ]; then
                echo "SUCCESS" >> /tmp/bench_result_$$
            else
                echo "FAILED" >> /tmp/bench_result_$$
            fi
        ) &
        
        # 控制并发数
        if [ $((i % CONCURRENT)) -eq 0 ]; then
            wait
        fi
    done
    wait
    
    END=$(date +%s%3N)
    DURATION=$(echo "scale=3; ($END - $START) / 1000" | bc)
    
    if [ -f /tmp/bench_result_$$ ]; then
        SUCCESS=$(grep -c "SUCCESS" /tmp/bench_result_$$ || true)
        FAILED=$(grep -c "FAILED" /tmp/bench_result_$$ || true)
        rm /tmp/bench_result_$$
    fi
    
    QPS=$(echo "scale=2; $TOTAL / $DURATION" | bc)
    
    echo -e "${GREEN}测试完成${NC}"
    echo "  总请求数: ${TOTAL}"
    echo "  成功: ${SUCCESS}"
    echo "  失败: ${FAILED}"
    echo "  总耗时: ${DURATION}s"
    echo "  平均QPS: ${QPS}"
    echo ""
}

# wrk压测 (如果安装了wrk)
test_with_wrk() {
    if ! command -v wrk &> /dev/null; then
        echo -e "${YELLOW}[3/5] wrk压测 - 跳过 (未安装wrk)${NC}"
        echo "  安装wrk: brew install wrk (macOS) 或参考 https://github.com/wg/wrk"
        echo ""
        return
    fi
    
    echo -e "${YELLOW}[3/5] wrk压测 - 100连接 10秒${NC}"
    
    if [ -f "bench/wrk_test.lua" ]; then
        wrk -t4 -c100 -d10s --latency -s bench/wrk_test.lua "${PUZZLE_URL}"
    else
        wrk -t4 -c100 -d10s --latency "${PUZZLE_URL}"
    fi
    echo ""
}

# 高负载测试 (500 QPS目标)
test_high_load() {
    if ! command -v wrk &> /dev/null; then
        echo -e "${YELLOW}[4/5] 高负载测试 (500 QPS) - 跳过 (需要wrk)${NC}"
        echo ""
        return
    fi
    
    echo -e "${YELLOW}[4/5] 高负载测试 - 目标500 QPS${NC}"
    echo "  配置: 8线程, 200连接, 持续30秒"
    echo ""
    
    wrk -t8 -c200 -d30s --latency "${PUZZLE_URL}"
    echo ""
}

# 完整流程测试（生成+验证）
test_full_workflow() {
    echo -e "${YELLOW}[5/5] 完整流程测试 (生成+验证)${NC}"
    
    # 生成验证码
    echo "步骤1: 生成验证码..."
    RESPONSE=$(curl -s "${PUZZLE_URL}")
    ID=$(echo "$RESPONSE" | grep -o '"id":"[^"]*' | cut -d'"' -f4)
    
    if [ -z "$ID" ]; then
        echo -e "${RED}✗ 生成验证码失败${NC}"
        echo ""
        return
    fi
    
    echo -e "${GREEN}✓ 验证码生成成功${NC}"
    echo "  ID: ${ID}"
    
    # 验证（使用随机值测试）
    echo "步骤2: 提交验证..."
    VERIFY_DATA="{\"id\":\"${ID}\",\"x\":0.5}"
    VERIFY_RESPONSE=$(curl -s -X POST \
        -H "Content-Type: application/json" \
        -d "${VERIFY_DATA}" \
        "${VERIFY_URL}")
    
    echo "  响应: ${VERIFY_RESPONSE}"
    echo ""
}

# 内存使用监控
monitor_memory() {
    echo -e "${YELLOW}提示: 性能测试建议${NC}"
    echo "  1. 使用 --release 模式编译运行服务"
    echo "  2. 监控内存使用: watch -n 1 'ps aux | grep actix_production'"
    echo "  3. 监控缓存状态: curl ${HOST}/health"
    echo "  4. 对于高QPS测试，建议使用专业工具如 wrk, ab, 或 vegeta"
    echo ""
}

# 主函数
main() {
    check_service
    test_single_request
    test_concurrent_curl
    test_with_wrk
    test_high_load
    test_full_workflow
    monitor_memory
    
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}  性能测试完成！${NC}"
    echo -e "${GREEN}========================================${NC}"
}

main

