use redis::{Client, ConnectionInfo};
use redis::aio::ConnectionManagerConfig;
use std::time::Duration;

fn main() {
    let mut config = ConnectionManagerConfig::new();
    config = config.set_connection_timeout(Duration::from_secs(10));
    
    // Check if set_tcp_keepalive exists?
    
}
