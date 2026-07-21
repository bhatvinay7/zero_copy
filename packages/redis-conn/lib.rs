pub fn test_keepalive() {
    let _info = redis::ConnectionInfo {
        addr: redis::ConnectionAddr::Tcp("127.0.0.1".into(), 6379),
        redis: redis::RedisConnectionInfo::default(),
    };
}
pub fn test_ping() {
    let mut config = redis::aio::ConnectionManagerConfig::new();
    // Does it have ping?
    config = config.set_connection_timeout(std::time::Duration::from_secs(10));
}
