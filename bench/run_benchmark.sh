#!/bin/bash

# Performance testing script
# Test the performance of slider_captcha_server

set -e

# Color definitions
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
HOST=${HOST:-"http://101.126.148.100:8080"}
PUZZLE_URL="${HOST}/puzzle"
VERIFY_URL="${HOST}/puzzle/solution"

now_ms() {
    if command -v python3 >/dev/null 2>&1; then
        python3 - <<'PY'
import time
print(int(time.time() * 1000))
PY
    else
        echo $(( $(date +%s) * 1000 ))
    fi
}
TEST_DATA_DIR="bench/test_data"

# Create test data directory
mkdir -p "${TEST_DATA_DIR}"

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  Slider Captcha Performance Test Suite${NC}"
echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  Test Server: ${HOST}${NC}"
echo -e "${BLUE}  Image Save Directory: ${TEST_DATA_DIR}${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Check dependencies
if ! command -v jq &> /dev/null; then
    echo -e "${YELLOW}Note: jq not installed, some features will be limited (cannot save images)${NC}"
    echo -e "${YELLOW}     Install: brew install jq (macOS)${NC}"
    echo ""
fi

# Check if service is running
check_service() {
    echo -e "${YELLOW}Checking service status...${NC}"
    
    # Increase timeout to 15 seconds to avoid network fluctuations
    HEALTH_RESPONSE=$(curl -s --max-time 15 -w "\n%{http_code}" "${HOST}/health" 2>&1)
    CURL_EXIT=$?
    HTTP_CODE=$(echo "$HEALTH_RESPONSE" | tail -n 1)
    
    if [ $CURL_EXIT -eq 0 ] && [ "$HTTP_CODE" = "200" ]; then
        echo -e "${GREEN}✓ Service is running${NC}"
        # Display health status
        HEALTH_BODY=$(echo "$HEALTH_RESPONSE" | sed '$d')
        if command -v jq &> /dev/null; then
            PREFILL_SIZES=$(echo "$HEALTH_BODY" | jq -c '.prefill_sizes' 2>/dev/null)
            if [ "$PREFILL_SIZES" != "null" ]; then
                echo "  Prefill sizes: ${PREFILL_SIZES}"
            fi
        fi
    else
        echo -e "${RED}✗ Service is not running or inaccessible${NC}"
        echo "  Service address: ${HOST}/health"
        echo "  curl exit code: ${CURL_EXIT}"
        echo "  HTTP status code: ${HTTP_CODE}"
        echo ""
        
        # Provide specific error information based on curl exit code
        case $CURL_EXIT in
            28)
                echo -e "${RED}  Error: Connection timeout${NC}"
                echo "  Possible causes: Server overload, network latency, or server unresponsive"
                ;;
            7)
                echo -e "${RED}  Error: Cannot connect to server${NC}"
                echo "  Possible causes: Server not started, firewall blocking, or wrong address"
                ;;
            6)
                echo -e "${RED}  Error: Cannot resolve hostname${NC}"
                echo "  Possible causes: DNS resolution failed or wrong address format"
                ;;
            *)
                echo -e "${RED}  Error: curl exit code ${CURL_EXIT}${NC}"
                ;;
        esac
        
        echo ""
        echo "  Please check:"
        echo "  1. Is the service started: cargo run --bin server --release"
        echo "  2. Is the firewall allowing access"
        echo "  3. Is the HOST address correct: ${HOST}"
        echo "  4. Is the server overloaded, try again later"
        echo ""
        echo -e "${YELLOW}  Tip: You can try manual connection test:${NC}"
        echo "  curl -v ${HOST}/health"
        exit 1
    fi
    echo ""
}

# Save base64 image to file
save_base64_image() {
    local base64_data=$1
    local output_file=$2
    
    if command -v base64 &> /dev/null; then
        echo "$base64_data" | base64 -d > "$output_file" 2>/dev/null
        return $?
    fi
    return 1
}

# Single request test
test_single_request() {
    echo -e "${YELLOW}[1/5] Single Request Test${NC}"
    echo "Sending a single request and measuring response time..."
    
    START=$(now_ms)
    # Use temporary file to avoid complex pipe processing
    TEMP_FILE="/tmp/benchmark_response_$$"
    curl -s --max-time 15 -w "\n__STATUS__%{http_code}\n__TIME__%{time_total}" "${PUZZLE_URL}" > "$TEMP_FILE" 2>&1
    CURL_EXIT=$?
    END=$(now_ms)
    
    if [ $CURL_EXIT -ne 0 ]; then
        echo -e "${RED}✗ Request failed, curl exit code: ${CURL_EXIT}${NC}"
        rm -f "$TEMP_FILE"
        echo ""
        return
    fi
    
    # Extract status code and time
    STATUS=$(grep "__STATUS__" "$TEMP_FILE" | cut -d'_' -f5)
    TIME=$(grep "__TIME__" "$TEMP_FILE" | cut -d'_' -f5)
    # Get response body (excluding last two lines of status and time info)
    BODY=$(grep -v "^__STATUS__\|^__TIME__" "$TEMP_FILE")
    
    if [ "$STATUS" = "200" ]; then
        echo -e "${GREEN}✓ Request successful${NC}"
        echo "  HTTP status code: ${STATUS}"
        echo "  Response time: ${TIME}s"
        
        # Save images
        if command -v jq &> /dev/null; then
            TIMESTAMP=$(date +%Y%m%d_%H%M%S)
            PUZZLE_IMG=$(echo "$BODY" | jq -r '.puzzle_image' 2>/dev/null)
            PIECE_IMG=$(echo "$BODY" | jq -r '.piece_image' 2>/dev/null)
            
            if [ "$PUZZLE_IMG" != "null" ] && [ -n "$PUZZLE_IMG" ]; then
                save_base64_image "$PUZZLE_IMG" "${TEST_DATA_DIR}/puzzle_${TIMESTAMP}.png"
                echo -e "${GREEN}  ✓ Puzzle image saved: ${TEST_DATA_DIR}/puzzle_${TIMESTAMP}.png${NC}"
            fi
            
            if [ "$PIECE_IMG" != "null" ] && [ -n "$PIECE_IMG" ]; then
                save_base64_image "$PIECE_IMG" "${TEST_DATA_DIR}/piece_${TIMESTAMP}.png"
                echo -e "${GREEN}  ✓ Puzzle piece saved: ${TEST_DATA_DIR}/piece_${TIMESTAMP}.png${NC}"
            fi
        else
            echo -e "${YELLOW}  Tip: Install jq to save images (brew install jq)${NC}"
        fi
    else
        echo -e "${RED}✗ Request failed, status code: ${STATUS}${NC}"
    fi
    
    rm -f "$TEMP_FILE"
    echo ""
}

# Concurrent test using curl
test_concurrent_curl() {
    echo -e "${YELLOW}[2/5] Concurrent Test (50 concurrent x 100 requests)${NC}"
    
    SUCCESS=0
    FAILED=0
    TOTAL=100
    CONCURRENT=50
    
    echo "Starting test..."
    START=$(now_ms)
    
    for i in $(seq 1 $TOTAL); do
        (
            STATUS=$(curl -s --max-time 15 -o /dev/null -w "%{http_code}" "${PUZZLE_URL}")
            if [ "$STATUS" = "200" ]; then
                echo "SUCCESS" >> /tmp/bench_result_$$
            else
                echo "FAILED" >> /tmp/bench_result_$$
            fi
        ) &
        
        # Control concurrency
        if [ $((i % CONCURRENT)) -eq 0 ]; then
            wait
        fi
    done
    wait
    
    END=$(now_ms)
    DURATION_MS=$((END - START))
    if [ $DURATION_MS -le 0 ]; then
        DURATION_MS=1
    fi
    DURATION=$(echo "scale=3; $DURATION_MS / 1000" | bc)
    
    if [ -f /tmp/bench_result_$$ ]; then
        SUCCESS=$(grep -c "SUCCESS" /tmp/bench_result_$$ || true)
        FAILED=$(grep -c "FAILED" /tmp/bench_result_$$ || true)
        rm /tmp/bench_result_$$
    fi
    
    QPS=$(echo "scale=2; ($TOTAL * 1000) / $DURATION_MS" | bc)
    
    echo -e "${GREEN}Test completed${NC}"
    echo "  Total requests: ${TOTAL}"
    echo "  Successful: ${SUCCESS}"
    echo "  Failed: ${FAILED}"
    echo "  Total duration: ${DURATION}s"
    echo "  Average QPS: ${QPS}"
    echo ""
}

# wrk stress test (if wrk is installed)
test_with_wrk() {
    if ! command -v wrk &> /dev/null; then
        echo -e "${YELLOW}[3/5] wrk Stress Test - Skipped (wrk not installed)${NC}"
        echo "  Install wrk: brew install wrk (macOS) or refer to https://github.com/wg/wrk"
        echo ""
        return
    fi
    
    echo -e "${YELLOW}[3/5] wrk Stress Test - 100 connections 10 seconds${NC}"
    
    if [ -f "bench/wrk_test.lua" ]; then
        wrk -t4 -c100 -d10s --latency -s bench/wrk_test.lua "${PUZZLE_URL}"
    else
        wrk -t4 -c100 -d10s --latency "${PUZZLE_URL}"
    fi
    echo ""
}

# High load test (500 QPS target)
test_high_load() {
    if ! command -v wrk &> /dev/null; then
        echo -e "${YELLOW}[4/5] High Load Test (500 QPS) - Skipped (requires wrk)${NC}"
        echo ""
        return
    fi
    
    echo -e "${YELLOW}[4/5] High Load Test - Target 500 QPS${NC}"
    echo "  Configuration: 8 threads, 200 connections, 30 seconds duration"
    echo ""
    
    wrk -t8 -c200 -d30s --latency "${PUZZLE_URL}"
    echo ""
}

# Full workflow test (generate + verify)
test_full_workflow() {
    echo -e "${YELLOW}[5/5] Full Workflow Test (Generate + Verify)${NC}"
    
    # Generate captcha
    echo "Step 1: Generating captcha..."
    RESPONSE=$(curl -s --max-time 15 "${PUZZLE_URL}")
    CURL_EXIT=$?
    
    if [ $CURL_EXIT -ne 0 ] || [ -z "$RESPONSE" ]; then
        echo -e "${RED}✗ Captcha generation failed (curl exit code: ${CURL_EXIT})${NC}"
        echo ""
        return
    fi
    
    # Parse JSON using jq (if available)
    if command -v jq &> /dev/null; then
        ID=$(echo "$RESPONSE" | jq -r '.id' 2>/dev/null)
        Y=$(echo "$RESPONSE" | jq -r '.y' 2>/dev/null)
        PUZZLE_IMG=$(echo "$RESPONSE" | jq -r '.puzzle_image' 2>/dev/null)
        PIECE_IMG=$(echo "$RESPONSE" | jq -r '.piece_image' 2>/dev/null)
    else
        # Fallback: use grep
        ID=$(echo "$RESPONSE" | grep -o '"id":"[^"]*' | cut -d'"' -f4)
    fi
    
    if [ -z "$ID" ] || [ "$ID" = "null" ]; then
        echo -e "${RED}✗ Captcha generation failed (cannot parse ID)${NC}"
        echo "  Response: ${RESPONSE:0:200}..."
        echo ""
        return
    fi
    
    echo -e "${GREEN}✓ Captcha generated successfully${NC}"
    echo "  ID: ${ID}"
    if [ -n "$Y" ] && [ "$Y" != "null" ]; then
        echo "  Y coordinate: ${Y}"
    fi
    
    # Save images to test_data directory
    if command -v jq &> /dev/null; then
        TIMESTAMP=$(date +%Y%m%d_%H%M%S)
        
        if [ "$PUZZLE_IMG" != "null" ] && [ -n "$PUZZLE_IMG" ]; then
            save_base64_image "$PUZZLE_IMG" "${TEST_DATA_DIR}/workflow_puzzle_${TIMESTAMP}.png"
            if [ $? -eq 0 ]; then
                echo -e "${GREEN}  ✓ Puzzle image saved: ${TEST_DATA_DIR}/workflow_puzzle_${TIMESTAMP}.png${NC}"
            fi
        fi
        
        if [ "$PIECE_IMG" != "null" ] && [ -n "$PIECE_IMG" ]; then
            save_base64_image "$PIECE_IMG" "${TEST_DATA_DIR}/workflow_piece_${TIMESTAMP}.png"
            if [ $? -eq 0 ]; then
                echo -e "${GREEN}  ✓ Puzzle piece saved: ${TEST_DATA_DIR}/workflow_piece_${TIMESTAMP}.png${NC}"
            fi
        fi
    fi
    
    # Verification (using random value for testing)
    echo ""
    echo "Step 2: Submitting verification..."
    VERIFY_DATA="{\"id\":\"${ID}\",\"x\":0.5}"
    set +e
    VERIFY_RESPONSE=$(curl -s --max-time 15 -X POST \
        -H "Content-Type: application/json" \
        -d "${VERIFY_DATA}" \
        "${VERIFY_URL}")
    VERIFY_EXIT=$?
    set -e
    
    if [ $VERIFY_EXIT -ne 0 ] || [ -z "$VERIFY_RESPONSE" ]; then
        echo -e "${RED}✗ Verification request failed (curl exit code: ${VERIFY_EXIT})${NC}"
    else
        echo "  Response: ${VERIFY_RESPONSE}"
        
        # Check verification result
        if echo "$VERIFY_RESPONSE" | grep -q '"success":true'; then
            echo -e "${GREEN}✓ Verification successful${NC}"
        elif echo "$VERIFY_RESPONSE" | grep -q '"success":false'; then
            echo -e "${YELLOW}○ Verification failed (expected, using random x value)${NC}"
        elif echo "$VERIFY_RESPONSE" | grep -q 'VERIFIED'; then
            echo -e "${GREEN}✓ Verification successful${NC}"
        elif echo "$VERIFY_RESPONSE" | grep -q 'Incorrect'; then
            echo -e "${YELLOW}○ Verification failed (expected, using random x value)${NC}"
        fi
    fi
    echo ""
}

# Memory usage monitoring
monitor_memory() {
    echo -e "${YELLOW}Tips: Performance Testing Recommendations${NC}"
    echo "  1. Use --release mode to compile and run the service"
    echo "  2. Monitor memory usage: watch -n 1 'ps aux | grep actix_production'"
    echo "  3. Monitor cache status: curl ${HOST}/health"
    echo "  4. For high QPS testing, recommend using professional tools like wrk, ab, or vegeta"
    echo ""
}

# Main function
main() {
    check_service
    test_single_request
    test_concurrent_curl
    test_with_wrk
    test_high_load
    test_full_workflow
    monitor_memory
    
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}  Performance testing completed!${NC}"
    echo -e "${GREEN}========================================${NC}"
}

main