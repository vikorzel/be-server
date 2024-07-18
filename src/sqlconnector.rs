use async_std::task::block_on;
use async_trait::async_trait;
use sqlx::{postgres::PgPoolOptions, query};

use crate::external::abstract_external::ExternalDatabase;

#[derive(Default)]
struct PostgressDatabaseConfig {
    host: String,
    username: String,
    password: String,
    port: u16,
    basename: String,
}

#[derive(Default)]
pub struct PostgressDatabase {
    config: PostgressDatabaseConfig,
    pool: Option<sqlx::Pool<sqlx::Postgres>>,
}

#[async_trait]
impl ExternalDatabase for PostgressDatabase {
    async fn get_device_config(&mut self, key: &String) -> Result<String, std::io::Error> {
        const TABLE_NAME: &'static str = "DeviceConfig";
        let tab_name = TABLE_NAME;
        println!("Getting config for device: {}", key);
        let pool = self.pool.as_ref().expect("No connection to database");
        let id = key.parse::<i32>().expect("Error parsing key");
        let query = format!("SELECT config FROM {} WHERE id = $1", tab_name); 
        let config:(sqlx::types::JsonValue,) = sqlx::query_as(query.as_str())
            .bind(id)
            .fetch_one(pool)
            .await.expect("Error fetching data from database");
        Ok(config.0.to_string().to_owned())
    }
}

impl PostgressDatabaseConfig {
    pub fn make_connection_string(&self) -> String {
        let connection_string = format!(
            "postgres://{}:{}@{}:{}/{}", self.username, self.password, self.host, self.port, self.basename
        );
        return connection_string;
    }
}

impl PostgressDatabase {
    pub fn new(
        username: String,
        password: String,
        host: String,
        basename: String,
        port: u16,
    ) -> PostgressDatabase {
        let database_instance_config = PostgressDatabaseConfig {
            host: host,
            username: username,
            port: port,
            password: password,
            basename: basename,
        };
        let mut instance = PostgressDatabase {
            config: database_instance_config,
            ..Default::default()
        };
        instance.connect();

        instance
    }

    fn connect(&mut self) {
        let connection_string = self.config.make_connection_string();
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&connection_string);

        match block_on(pool) {
            Ok(pg_pool) => {
                println!("Connected to database");
                self.pool = Some(pg_pool);
            }
            Err(e) => {
                println!("Connection string: {}", connection_string);
                println!("Error connecting to database: {}", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::sqlconnector::PostgressDatabaseConfig;

    #[test]
    fn connection_string() {
        let config = PostgressDatabaseConfig {
            host: "hostname".to_owned(),
            username: "user".to_owned(),
            password: "password".to_owned(),
            port: 1234,
            basename: "base".to_owned(),
        };

        let connection_string = config.make_connection_string();
        assert_eq!(
            connection_string,
            "postgres://user:password@hostname:1234/base"
        );
    }
}
