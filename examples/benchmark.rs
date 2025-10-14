// Rust performance testing tool
// For testing slider_captcha_server performance

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;

const TARGET_URL: &str = "http://127.0.0.1:8080/puzzle";

#[tokio::main]
async fn main() {
    println!("\n🚀 Slider Captcha Performance Test Tool\n");
    println!("========================================");

    // Check service status
    print!("Checking service connection... ");
    if !check_service().await {
        println!("❌ Failed");
        println!("\nPlease start the service first:");
        println!("  cargo run --example actix_production --release\n");
        return;
    }
    println!("✓ Success");
    println!("========================================\n");

    // Run various tests
    run_warmup_test().await;
    run_sustained_load_test(50, 10).await;
    run_sustained_load_test(100, 10).await;
    run_sustained_load_test(200, 10).await;
    run_peak_load_test(500, 10).await;
    run_stress_test(1000, 5).await;

    println!("\n========================================");
    println!("✅ All tests completed!");
    println!("========================================\n");
}

async fn check_service() -> bool {
    let client = reqwest::Client::new();
    client
        .get("http://127.0.0.1:8080/health")
        .timeout(Duration::from_secs(2))
        .send()
        .await
        .is_ok()
}

async fn run_warmup_test() {
    println!("[Warmup Test] 10 requests");
    println!("----------------------------------------");

    let client = reqwest::Client::new();
    for i in 1..=10 {
        let start = Instant::now();
        match client.get(TARGET_URL).send().await {
            Ok(resp) => {
                let duration = start.elapsed();
                println!(
                    "  Request {}: {} - {:.2}ms",
                    i,
                    resp.status(),
                    duration.as_secs_f64() * 1000.0
                );
            }
            Err(e) => println!("  Request {i}: ❌ Error: {e}"),
        }
    }
    println!();
}

async fn run_sustained_load_test(qps: u64, duration_secs: u64) {
    println!("[Sustained Load Test] {qps} QPS, duration {duration_secs} seconds");
    println!("----------------------------------------");

    let success_count = Arc::new(AtomicU64::new(0));
    let error_count = Arc::new(AtomicU64::new(0));
    let total_latency = Arc::new(AtomicU64::new(0));

    let start_time = Instant::now();
    let end_time = start_time + Duration::from_secs(duration_secs);

    let interval = Duration::from_micros(1_000_000 / qps);

    while Instant::now() < end_time {
        let sc = success_count.clone();
        let ec = error_count.clone();
        let tl = total_latency.clone();

        tokio::spawn(async move {
            let req_start = Instant::now();
            let client = reqwest::Client::new();

            match client
                .get(TARGET_URL)
                .timeout(Duration::from_secs(10))
                .send()
                .await
            {
                Ok(resp) => {
                    let latency = req_start.elapsed();
                    if resp.status().is_success() {
                        sc.fetch_add(1, Ordering::Relaxed);
                        tl.fetch_add(latency.as_micros() as u64, Ordering::Relaxed);
                    } else {
                        ec.fetch_add(1, Ordering::Relaxed);
                    }
                }
                Err(_) => {
                    ec.fetch_add(1, Ordering::Relaxed);
                }
            }
        });

        sleep(interval).await;
    }

    // Wait for all requests to complete
    sleep(Duration::from_secs(2)).await;

    let actual_duration = start_time.elapsed();
    let success = success_count.load(Ordering::Relaxed);
    let errors = error_count.load(Ordering::Relaxed);
    let total = success + errors;
    let avg_latency = if success > 0 {
        total_latency.load(Ordering::Relaxed) / success
    } else {
        0
    };

    let actual_qps = total as f64 / actual_duration.as_secs_f64();
    let success_rate = if total > 0 {
        (success as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    println!("  Total requests: {total}");
    println!("  Success: {success} ({success_rate:.1}%)");
    println!("  Failed: {errors}");
    println!("  Actual QPS: {actual_qps:.2}");
    println!("  Average latency: {:.2}ms", avg_latency as f64 / 1000.0);

    if success_rate >= 99.0 {
        println!("  Result: ✅ Passed");
    } else if success_rate >= 95.0 {
        println!("  Result: ⚠️  Warning");
    } else {
        println!("  Result: ❌ Failed");
    }
    println!();
}

async fn run_peak_load_test(qps: u64, duration_secs: u64) {
    println!("[Peak Load Test] Target {qps} QPS, duration {duration_secs} seconds");
    println!("----------------------------------------");

    let success_count = Arc::new(AtomicU64::new(0));
    let error_count = Arc::new(AtomicU64::new(0));
    let latencies = Arc::new(tokio::sync::Mutex::new(Vec::new()));

    let start_time = Instant::now();
    let end_time = start_time + Duration::from_secs(duration_secs);

    let interval = Duration::from_micros(1_000_000 / qps);

    while Instant::now() < end_time {
        let sc = success_count.clone();
        let ec = error_count.clone();
        let lat = latencies.clone();

        tokio::spawn(async move {
            let req_start = Instant::now();
            let client = reqwest::Client::new();

            match client
                .get(TARGET_URL)
                .timeout(Duration::from_secs(10))
                .send()
                .await
            {
                Ok(resp) => {
                    let latency = req_start.elapsed();
                    if resp.status().is_success() {
                        sc.fetch_add(1, Ordering::Relaxed);
                        let mut l = lat.lock().await;
                        l.push(latency.as_micros() as u64);
                    } else {
                        ec.fetch_add(1, Ordering::Relaxed);
                    }
                }
                Err(_) => {
                    ec.fetch_add(1, Ordering::Relaxed);
                }
            }
        });

        sleep(interval).await;
    }

    // Wait for all requests to complete
    sleep(Duration::from_secs(3)).await;

    let actual_duration = start_time.elapsed();
    let success = success_count.load(Ordering::Relaxed);
    let errors = error_count.load(Ordering::Relaxed);
    let total = success + errors;
    let actual_qps = total as f64 / actual_duration.as_secs_f64();

    let mut lats = latencies.lock().await;
    lats.sort_unstable();

    let p50 = if !lats.is_empty() {
        lats[lats.len() / 2] as f64 / 1000.0
    } else {
        0.0
    };

    let p95 = if !lats.is_empty() {
        lats[lats.len() * 95 / 100] as f64 / 1000.0
    } else {
        0.0
    };

    let p99 = if !lats.is_empty() {
        lats[lats.len() * 99 / 100] as f64 / 1000.0
    } else {
        0.0
    };

    let avg = if !lats.is_empty() {
        lats.iter().sum::<u64>() as f64 / lats.len() as f64 / 1000.0
    } else {
        0.0
    };

    println!("  Total requests: {total}");
    println!("  Success: {success}");
    println!("  Failed: {errors}");
    println!("  Actual QPS: {actual_qps:.2}");
    println!("  Latency statistics:");
    println!("    Average: {avg:.2}ms");
    println!("    P50: {p50:.2}ms");
    println!("    P95: {p95:.2}ms");
    println!("    P99: {p99:.2}ms");

    if actual_qps >= qps as f64 * 0.95 && errors == 0 {
        println!("  Result: ✅ Target achieved");
    } else if actual_qps >= qps as f64 * 0.80 {
        println!("  Result: ⚠️  Close to target");
    } else {
        println!("  Result: ❌ Target not reached");
    }
    println!();
}

async fn run_stress_test(qps: u64, duration_secs: u64) {
    println!("[Stress Test] {qps} QPS, duration {duration_secs} seconds");
    println!("----------------------------------------");

    let success_count = Arc::new(AtomicU64::new(0));
    let error_count = Arc::new(AtomicU64::new(0));

    let start_time = Instant::now();
    let end_time = start_time + Duration::from_secs(duration_secs);

    let interval = Duration::from_micros(1_000_000 / qps);

    while Instant::now() < end_time {
        let sc = success_count.clone();
        let ec = error_count.clone();

        tokio::spawn(async move {
            let client = reqwest::Client::new();
            match client
                .get(TARGET_URL)
                .timeout(Duration::from_secs(10))
                .send()
                .await
            {
                Ok(resp) => {
                    if resp.status().is_success() {
                        sc.fetch_add(1, Ordering::Relaxed);
                    } else {
                        ec.fetch_add(1, Ordering::Relaxed);
                    }
                }
                Err(_) => {
                    ec.fetch_add(1, Ordering::Relaxed);
                }
            }
        });

        sleep(interval).await;
    }

    // Wait for all requests to complete
    sleep(Duration::from_secs(3)).await;

    let actual_duration = start_time.elapsed();
    let success = success_count.load(Ordering::Relaxed);
    let errors = error_count.load(Ordering::Relaxed);
    let total = success + errors;
    let actual_qps = total as f64 / actual_duration.as_secs_f64();
    let error_rate = if total > 0 {
        (errors as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    println!("  Total requests: {total}");
    println!("  Success: {success}");
    println!("  Failed: {errors} ({error_rate:.1}%)");
    println!("  Actual QPS: {actual_qps:.2}");
    println!("  Result: Stress test completed, error rate {error_rate:.2}%");
    println!();
}
