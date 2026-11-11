// WiFi
pub const DEFAULT_WIFI_SSID: &str = "YOUR_WIFI_SSID";
pub const DEFAULT_WIFI_PASSWORD: &str = "YOUR_WIFI_PASSWORD";

// Partitioning
pub const ESP_LITTLEFS_PARTITION_OFFSET: usize = 0x310000;
pub const ESP_LITTLEFS_PARTITION_SIZE: usize = 0xF0000;

// Web Server
pub const WEB_SERVER_PORT: usize = 80;
pub const WEB_TASK_POOL_SIZE: usize = 4;
pub const HTTP_BUFFER_SIZE: usize = 4096;
pub const UPLOAD_CHUNK_SIZE: usize = 32 * 1024; //match with clients

// Net Service
pub const SOCKET_SIZE: usize = 4;
pub const NET_STACK_RESOURCES_SIZE: usize = WEB_TASK_POOL_SIZE + SOCKET_SIZE;

// Path
pub const PATH_MAX_LENGTH: usize = 256;

// Intervals
pub const WIFI_RETRY_INTERVAL: usize = 2000;
pub const DEBUG_SERVICE_INTERVAL: usize = 5000;
