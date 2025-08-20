CREATE TABLE failed_event (
  id              BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
  timestamp       BIGINT NOT NULL,
  attempted_at    BIGINT NOT NULL,
  event_data      LONGTEXT NOT NULL,
  handler_name    VARCHAR(191) NOT NULL,
  imei            VARCHAR(20) NOT NULL,
  INDEX idx_failed_event_timestamp (timestamp)
);