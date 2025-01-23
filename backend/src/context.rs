use aws_config::SdkConfig;
use aws_sdk_cognitoidentityprovider::{
    operation::get_user::{GetUserError, GetUserOutput}, Client as AwsCognitoClient,
};
use aws_sdk_secretsmanager::operation::get_secret_value::GetSecretValueError;
use aws_sdk_secretsmanager::Client as AwsSecretsClient;
use aws_sdk_dynamodb::Client as AwsDynamoDbClient;
use aws_smithy_runtime_api::{client::result::SdkError, http::Response};
use axum::{body::Body, http::StatusCode, response::IntoResponse};
use tokio::sync::RwLock;

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use thiserror::Error;
use url::Url;


// Tell axum how to convert `AppError` into a response.
impl IntoResponse for ContextError {
    fn into_response(self) -> axum::http::Response<Body> {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self),
        )
            .into_response()
    }
}

#[derive(Error, Debug)]
pub enum RDSCredentialsError {
    #[error("error with secrets manager: {0}")]
    Aws(#[from] SdkError<GetSecretValueError, Response>),
    #[error("this secret does not exist")]
    SecretNotFound,
    #[error("this secret does not conform to the JSON specification required")]
    ParseError,
}

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("error with cognito interface: {0}")]
    AwsGetUser(#[from] SdkError<GetUserError, Response>),
	#[error("missing authentication cookie")]
	MissingAuthenticationCookie,
}

#[derive(Error, Debug)]
pub enum ContextError {
    #[error("could not load credentials for database: {0}")]
    RDSCredentialsInitialization(#[from] RDSCredentialsError),
    #[error("database connection string is missing a password, and no secret key was found")]
    MissingPassword,
    #[error("database schema should be `postgres`, found `{0}`")]
    BadSchema(String),
    #[error("could not parse url: {0}")]
    UrlParse(#[from] url::ParseError),
    #[error("could not authenticate: {0}")]
    AuthError(#[from] AuthError),
}

#[derive(Clone, Debug)]
pub struct Context {
    aws_sdk_config: Arc<RwLock<SdkConfig>>,
    aws_dynamodb: Arc<RwLock<AwsDynamoDbClient>>,
    aws_secrets: Option<Arc<RwLock<AwsSecretsClient>>>,
    aws_cognito: Arc<RwLock<AwsCognitoClient>>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
struct RDSCredentials {
    username: String,
    password: String,
}

impl RDSCredentials {
    pub async fn new<S: Into<String>>(
        client: &AwsSecretsClient,
        secret_id: S,
    ) -> Result<Self, RDSCredentialsError> {
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

        let mut aws_secrets = None;

        if rds_location.password().is_none() || rds_location.username().is_empty() {
            let Some(rds_secret_id) = rds_secret_id else {
                return Err(ContextError::MissingPassword);
            };

            let secrets_client = AwsSecretsClient::new(&sdk_config);

            let database_credentials = RDSCredentials::new(&secrets_client, rds_secret_id).await?;

            rds_location
                .set_password(Some(&database_credentials.password))
                .expect("postgres schema accepts password");
            rds_location
                .set_username(&database_credentials.username)
                .expect("postgres schema accepts username");

            aws_secrets = Some(Arc::new(RwLock::new(secrets_client)));
        }

        let cognito_client = AwsCognitoClient::new(&sdk_config);

        let dynamodb_client = AwsDynamoDbClient::new(&sdk_config);

        Ok(Self {
            aws_sdk_config: Arc::new(RwLock::new(sdk_config)),
            aws_cognito: Arc::new(RwLock::new(cognito_client)),
            aws_dynamodb: Arc::new(RwLock::new(dynamodb_client)),
            aws_secrets,
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

    pub async fn aws_cognito_client<F, T, E>(&self, callback: F) -> Result<T, ContextError>
    where
        F: for<'c> FnOnce(
                &'c AwsCognitoClient,
            ) -> Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'c>>
            + Send,
        T: Send,
        E: Send,
        ContextError: From<E>,
    {
        let lock = self.aws_cognito.read().await;
        let output = callback(&lock).await?;
        Ok(output)
    }

    pub async fn aws_dynamodb_client<F, T, E>(&self, callback: F) -> Result<T, ContextError>
    where
        F: for<'c> FnOnce(
                &'c AwsDynamoDbClient,
            ) -> Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'c>>
            + Send,
        T: Send,
        E: Send,
        ContextError: From<E>,
    {
        let lock = self.aws_dynamodb.read().await;
        let output = callback(&lock).await?;
        Ok(output)
    }

    pub async fn aws_secrets_client<F, T, E>(&self, callback: F) -> Result<T, ContextError>
    where
        F: for<'c> FnOnce(
                Option<&'c AwsSecretsClient>,
            ) -> Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'c>>
            + Send,
        T: Send,
        E: Send,
        ContextError: From<E>,
    {
        let lock =
            futures::future::OptionFuture::from(self.aws_secrets.as_ref().map(|c| c.read())).await;
        let output = callback(lock.as_deref()).await?;
        Ok(output)
    }

    pub async fn load_cognito_user(&self, access_token: &str) -> Result<GetUserOutput, ContextError> {
        let client_lock = self.aws_cognito.read().await;

        let get_user_output = client_lock
            .get_user()
            .access_token(access_token)
            .send()
            .await
            .map_err(AuthError::from)?;

		Ok(get_user_output)
    }
}
