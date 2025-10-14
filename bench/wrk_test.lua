-- wrk性能测试脚本
-- 用于测试 /puzzle 接口的生成性能

wrk.method = "GET"
wrk.headers["Content-Type"] = "application/json"

-- 统计数据
local counter = 0
local errors = 0
local success = 0

-- 初始化
function setup(thread)
   thread:set("id", counter)
   counter = counter + 1
end

-- 每个请求调用
function response(status, headers, body)
    if status ~= 200 then
        errors = errors + 1
    else
        success = success + 1
    end
end

-- 测试完成后调用
function done(summary, latency, requests)
    io.write("==========================================\n")
    io.write("性能测试报告\n")
    io.write("==========================================\n")
    io.write(string.format("总请求数: %d\n", summary.requests))
    io.write(string.format("总耗时: %.2f 秒\n", summary.duration / 1000000))
    io.write(string.format("平均 QPS: %.2f\n", summary.requests / (summary.duration / 1000000)))
    io.write(string.format("成功请求: %d\n", success))
    io.write(string.format("失败请求: %d\n", errors))
    io.write("------------------------------------------\n")
    io.write("延迟统计:\n")
    io.write(string.format("  最小值: %.2f ms\n", latency.min / 1000))
    io.write(string.format("  最大值: %.2f ms\n", latency.max / 1000))
    io.write(string.format("  平均值: %.2f ms\n", latency.mean / 1000))
    io.write(string.format("  50%%ile: %.2f ms\n", latency:percentile(50) / 1000))
    io.write(string.format("  90%%ile: %.2f ms\n", latency:percentile(90) / 1000))
    io.write(string.format("  99%%ile: %.2f ms\n", latency:percentile(99) / 1000))
    io.write("==========================================\n")
end

