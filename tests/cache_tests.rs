use std::time::Duration;

use slider_captcha_server::cache::ExpiringCache;

#[tokio::test]
async fn expiring_cache_respects_ttl() {
    let cache = ExpiringCache::new(Duration::from_millis(50), 10);
    cache.insert("test", 42);
    assert!(cache.pop(&"test").is_some(), "Expected cache hit just after insert");

    tokio::time::sleep(Duration::from_millis(60)).await;

    assert!(cache.pop(&"test").is_none(), "Expected cache miss after TTL");
}

#[tokio::test]
async fn expiring_cache_truncates_to_max_len() {
    let cache = ExpiringCache::new(Duration::from_secs(1), 2);
    cache.insert("size", 1);
    cache.insert("size", 2);
    cache.insert("size", 3);

    assert_eq!(cache.len_for(&"size"), 2);
    let first = cache.pop(&"size").unwrap();
    let second = cache.pop(&"size").unwrap();
    assert!(cache.pop(&"size").is_none());

    assert!(*first == 3 || *second == 3, "Newest entry should be retained");
}

