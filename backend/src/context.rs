use aws_config::SdkConfig;
use aws_sdk_secretsmanager::operation::get_secret_value::GetSecretValueError;
use aws_smithy_runtime_api::{client::result::SdkError, http::Response};
use sea_orm::sqlx::postgres::PgConnectOptions;

use std::collections::{BTreeSet, HashMap, HashSet};
use std::ops::Deref;
use std::str::FromStr;

use anyhow::{anyhow, bail, Result};

use migration::{Expr, IntoCondition, Migrator, MigratorTrait};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, FromQueryResult, IntoActiveModel,
    JoinType, Order, PaginatorTrait, QueryFilter, QuerySelect, RelationTrait, Select, Set,
    TransactionError, TransactionTrait, TryIntoModel, UpdateResult, Value,
};
use sea_orm::{Database, DatabaseConnection, EntityTrait};
pub use sea_orm::{DbErr, DeleteResult};
use sea_orm::{EntityOrSelect, ModelTrait};
use sea_orm::{IntoSimpleExpr, QueryOrder};

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
}

pub struct Context {
    aws_sdk_config: SdkConfig,
	database_connection: DatabaseConnection,
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

		let raw_string = response.secret_string().ok_or(RDSCredentialsError::SecretNotFound)?;

		let credentials = serde_json::from_str::<RDSCredentials>(raw_string).map_err(|_| RDSCredentialsError::ParseError)?;

        Ok(credentials)
    }
}

impl Context {
    pub async fn new<S: Into<String>>(sdk_config: SdkConfig, rds_secret_id: Option<S>, mut rds_location: Url) -> Result<Self, ContextError> {
		let scheme = rds_location.scheme();

		if scheme != "postgres" {
			return Err(ContextError::BadSchema(scheme.to_owned()));
		}

		if rds_location.password().is_none() || rds_location.username().is_empty() {
			let Some(rds_secret_id) = rds_secret_id else {
				return Err(ContextError::MissingPassword);
			};
			let database_credentials = RDSCredentials::new(&sdk_config, rds_secret_id).await?;

			rds_location.set_password(Some(&database_credentials.password)).expect("postgres schema accepts password");
			rds_location.set_username(&database_credentials.username).expect("postgres schema accepts username");
		}

		let db = Database::connect(rds_location).await?;

		migration::Migrator::up(&db, None).await?;

		Ok(Self {
			aws_sdk_config: sdk_config,
			database_connection: db,
		})
    }

	pub async fn migrate(&self) {
		Migrator::refresh(&self.database_connection).await
;
		// Migrator::up(&self.database_connection, None).await;
	}
}
