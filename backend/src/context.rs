use aws_config::SdkConfig;
use aws_sdk_secretsmanager::operation::get_secret_value::GetSecretValueError;
use aws_smithy_runtime_api::{client::result::SdkError, http::Response};
use tokio::sync::RwLock;

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use migration::{Migrator, MigratorTrait};
pub use sea_orm::DbErr;
use sea_orm::{Database, DatabaseConnection};

use serde::{Deserialize, Serialize};

use thiserror::Error;
use url::Url;

#[derive(Error, Debug)]
pub enum RDSCredentialsError {
    #[error("error with secrets manager")]
    Aws(#[from] SdkError<GetSecretValueError, Response>),
    #[error("this secret does not exist")]
    SecretNotFound,
    #[error("this secret does not conform to the JSON specification required")]
    ParseError,
}

#[derive(Error, Debug)]
pub enum ContextError {
    #[error("could not load credentials for database")]
    RDSCredentialsInitialization(#[from] RDSCredentialsError),
    #[error("error from database")]
    Database(#[from] DbErr),
    #[error("database connection string is missing a password, and no secret key was found")]
    MissingPassword,
    #[error("database schema should be `postgres`, found `{0}`")]
    BadSchema(String),
    #[error("could not parse url")]
    UrlParse(#[from] url::ParseError),
}

#[derive(Clone, Debug)]
pub struct Context {
    aws_sdk_config: Arc<RwLock<SdkConfig>>,
    database_connection: Arc<RwLock<DatabaseConnection>>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
struct RDSCredentials {
    username: String,
    password: String,
}

impl RDSCredentials {
    pub async fn new<S: Into<String>>(
        sdk_config: &SdkConfig,
        secret_id: S,
    ) -> Result<Self, RDSCredentialsError> {
        let client = aws_sdk_secretsmanager::Client::new(sdk_config);

        let response = client
            .get_secret_value()
            .secret_id(secret_id)
            .send()
            .await?;

        let raw_string = response
            .secret_string()
            .ok_or(RDSCredentialsError::SecretNotFound)?;

        let credentials = serde_json::from_str::<RDSCredentials>(raw_string)
            .map_err(|_| RDSCredentialsError::ParseError)?;

        Ok(credentials)
    }
}

impl Context {
    pub async fn new<S, U>(
        sdk_config: SdkConfig,
        rds_secret_id: Option<S>,
        rds_location: U,
    ) -> Result<Self, ContextError>
    where
        S: Into<String>,
        U: TryInto<Url>,
        ContextError: From<<U as TryInto<Url>>::Error>,
    {
        let mut rds_location = rds_location.try_into()?;

        let scheme = rds_location.scheme();

        if scheme != "postgres" {
            return Err(ContextError::BadSchema(scheme.to_owned()));
        }

        if rds_location.password().is_none() || rds_location.username().is_empty() {
            let Some(rds_secret_id) = rds_secret_id else {
                return Err(ContextError::MissingPassword);
            };
            let database_credentials = RDSCredentials::new(&sdk_config, rds_secret_id).await?;

            rds_location
                .set_password(Some(&database_credentials.password))
                .expect("postgres schema accepts password");
            rds_location
                .set_username(&database_credentials.username)
                .expect("postgres schema accepts username");
        }

        let db = Database::connect(rds_location).await?;

        migration::Migrator::up(&db, None).await?;

        Ok(Self {
            aws_sdk_config: Arc::new(RwLock::new(sdk_config)),
            database_connection: Arc::new(RwLock::new(db)),
        })
    }

    pub async fn aws_sdk_config<F, T, E>(&self, callback: F) -> Result<T, ContextError>
    where
        F: for<'c> FnOnce(&'c SdkConfig) -> Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'c>>
            + Send,
        T: Send,
        E: Send,
        ContextError: From<E>,
    {
        let lock = self.aws_sdk_config.read().await;
        let output = callback(&lock).await?;
        Ok(output)
    }

	/// # Example usage
	/// ```no_run
	///self.database_connection(|database_connection| {
	///  Box::pin(async move {
	///    Migrator::up(database_connection, None).await?;
	///    Ok::<_, ContextError>(())
	///   })
	/// })
	/// .await?;
	/// ```
    pub async fn database_connection<F, T, E>(&self, callback: F) -> Result<T, ContextError>
    where
        F: for<'c> FnOnce(
                &'c DatabaseConnection,
            ) -> Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'c>>
            + Send,
        T: Send,
        E: Send,
        ContextError: From<E>,
    {
        let lock = self.database_connection.read().await;
        let output = callback(&lock).await?;
        Ok(output)
    }

    pub async fn migrate(&self) -> Result<(), ContextError> {
		let lock = self.database_connection.read().await;
		Migrator::up(&*lock, None).await?;
		Ok(())
    }

	pub async fn test_database_connection(&self) -> Result<(), ContextError> {
		self.database_connection.read().await.ping().await?;
		Ok(())
	}
}
