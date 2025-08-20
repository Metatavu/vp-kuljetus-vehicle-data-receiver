use log::debug;
use sqlx::{MySql, Pool, Row};

/// Module for handling failed events in the system.
pub struct FailedEventsHandler {
    database_pool: Pool<MySql>,
}

/// Struct representing a failed event.
pub struct FailedEvent {
    pub id: Option<u64>,
    pub timestamp: i64,
    pub event_data: String,
    pub handler_name: String,
    pub imei: String,
}

/// Errors that can occur when processing failed events.
#[derive(Debug)]
pub enum FailedEventError {
    FailedToResend,
    MissingId,
    HandlerNotFound(String),
}

/// Handler implementation for failed events.
impl FailedEventsHandler {
    /// Creates a new instance of the handler.
    ///
    /// # Arguments
    /// * `database_pool` - Database connection pool
    pub fn new(database_pool: Pool<MySql>) -> Self {
        FailedEventsHandler { database_pool }
    }

    /// Persists a failed event to the database.
    ///
    /// # Arguments
    /// * `imei` - The IMEI of the vehicle
    /// * `event` - The failed event to persist
    ///
    /// # Returns
    /// The ID of the persisted failed event
    pub async fn persist_event(&self, imei: String, event: FailedEvent) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            INSERT INTO failed_event (timestamp, attempted_at, imei, handler_name, event_data)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(event.timestamp)
        .bind(chrono::Utc::now().naive_utc().and_utc().timestamp())
        .bind(imei)
        .bind(event.handler_name)
        .bind(event.event_data)
        .execute(&self.database_pool)
        .await?;

        Ok(result.last_insert_id())
    }

    /// Retrieves the next failed IMEI from the database.
    ///
    /// # Returns
    /// The next failed IMEI, if it exists
    pub async fn next_failed_imei(&self) -> Result<Option<String>, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT imei FROM failed_event
            ORDER BY attempted_at DESC
            LIMIT 1
            "#,
        )
        .fetch_optional(&self.database_pool)
        .await?;

        if let Some(row) = row {
            let imei = row.try_get("imei")?;
            Ok(Some(imei))
        } else {
            Ok(None)
        }
    }

    /// Lists failed events for a specific IMEI.
    ///
    /// # Arguments
    /// * `imei` - The IMEI of the vehicle
    /// * `max_results` - The maximum number of results to return
    ///
    /// # Returns
    /// A list of failed events for the specified IMEI
    pub async fn list_failed_events(&self, imei: &str, max_results: u64) -> Result<Vec<FailedEvent>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT id, imei, timestamp, event_data, handler_name
            FROM failed_event
            WHERE imei = ?
            LIMIT ?
            "#,
        )
        .bind(imei)
        .bind(max_results)
        .fetch_all(&self.database_pool)
        .await?;

        let mut events = Vec::new();
        for row in rows {
            let id = row.try_get::<u64, _>("id").ok();
            let imei = row.try_get::<String, _>("imei")?;
            let timestamp = row.try_get::<i64, _>("timestamp")?;
            let event_data = row.try_get::<String, _>("event_data")?;
            let handler_name = row.try_get::<String, _>("handler_name")?;

            events.push(FailedEvent {
                id,
                timestamp,
                event_data,
                handler_name,
                imei,
            });
        }
        Ok(events)
    }

    /// Deletes a failed event by its ID.
    ///
    /// # Arguments
    /// * `event_id` - The ID of the failed event to delete
    ///
    /// # Returns
    /// A result indicating the success or failure of the operation
    pub async fn delete_failed_event(&self, event_id: u64) -> Result<(), sqlx::Error> {
        debug!("Deleting failed event: {}", event_id);

        sqlx::query("DELETE FROM failed_event WHERE id = ?")
            .bind(event_id)
            .execute(&self.database_pool)
            .await?;

        Ok(())
    }

    /// Updates the attempted_at timestamp for a failed event.
    ///
    /// # Arguments
    /// * `event_id` - The ID of the failed event to update
    /// * `attempted_at` - The new attempted_at timestamp
    ///
    /// # Returns
    /// A result indicating the success or failure of the operation
    pub async fn update_attempted_at(&self, event_id: u64, attempted_at: i64) -> Result<(), sqlx::Error> {
        debug!("Updating attempted status for failed event: {}", event_id);

        sqlx::query("UPDATE failed_event SET attempted_at = ? WHERE id = ?")
            .bind(attempted_at)
            .bind(event_id)
            .execute(&self.database_pool)
            .await?;

        Ok(())
    }
}
