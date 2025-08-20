use sqlx::{MySql, Pool};
use testcontainers::{
    core::{logs::consumer::logging_consumer::LoggingConsumer, IntoContainerPort, WaitFor},
    runners::AsyncRunner,
    ContainerAsync, GenericImage, ImageExt,
};

/// A test container running MySQL.
pub struct MySqlTestContainer {
    mysql_container: Option<ContainerAsync<GenericImage>>,
}

/// Implementation of MySqlTestContainer.
impl MySqlTestContainer {
    /// Creates a new instance of MySqlTestContainer.
    /// # Returns
    /// A new instance of MySqlTestContainer.
    pub fn new() -> Self {
        Self { mysql_container: None }
    }

    /// Starts the MySQL container with the specified configuration.
    /// # Returns
    /// A reference to the started MySqlTestContainer instance.
    /// # Errors
    /// Returns an error if the MySQL container fails to start.
    pub async fn start(&mut self) -> &mut Self {
        let mysql_container = GenericImage::new("mysql", "8")
            .with_exposed_port(3306.tcp())
            .with_wait_for(WaitFor::message_on_either_std("ready for connections. Version: '8.4.6'  socket: '/var/run/mysqld/mysqld.sock'  port: 3306  MySQL Community Server - GPL."))
            .with_network("tests")
            .with_env_var("MYSQL_ROOT_PASSWORD", "root")
            .with_env_var("MYSQL_DATABASE", "db")
            .with_container_name("mysql")
            .with_log_consumer(LoggingConsumer::new().with_prefix("mysql"));

        self.mysql_container = Some(mysql_container.start().await.unwrap());

        return self;
    }

    /// Stops the MySQL container.
    pub async fn stop(&mut self) {
        if let Some(container) = self.mysql_container.take() {
            container.stop().await.unwrap();
            container.rm().await.unwrap();
        }
    }

    /// Counts the number of failed trackable events in the MySQL database.
    ///
    /// # Returns
    /// The count of failed trackable events.
    pub async fn count_failed_events(&self) -> Result<i64, Box<dyn std::error::Error>> {
        // Replace the following with your actual connection logic
        let pool = self.get_connection_pool().await?;
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM failed_event")
            .fetch_one(&pool)
            .await?;
        Ok(row.0)
    }

    /// Gets a connection pool to the MySQL database.
    ///
    /// # Returns
    /// A connection pool to the MySQL database.
    /// # Errors
    /// Returns an error if the connection pool cannot be created.
    async fn get_connection_pool(&self) -> Result<Pool<MySql>, Box<dyn std::error::Error>> {
        let (host, port) = self.get_host_and_port().await;
        let url = format!("mysql://root:root@{}:{}/db", host, port);
        let pool = Pool::<MySql>::connect(&url).await?;
        Ok(pool)
    }

    /// Gets the host and port of the MySQL container.
    /// # Returns
    /// A tuple containing the host and port of the MySQL container.
    /// # Errors
    /// Returns an error if the host and port cannot be retrieved.
    async fn get_host_and_port(&self) -> (String, u16) {
        let container = self.mysql_container.as_ref().expect("MySQL not started");
        let mysql_host = container.get_host().await.unwrap().to_string();
        let mysql_port = container.get_host_port_ipv4(3306).await.unwrap();
        (mysql_host, mysql_port)
    }
}
