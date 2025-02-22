pub mod clocks;

use aws_config::SdkConfig;
use aws_sdk_cognitoidentityprovider::operation::get_user::{GetUserError, GetUserOutput};
use aws_smithy_runtime_api::{client::result::SdkError, http::Response};

pub(crate) use aws_sdk_cognitoidentityprovider::Client as AwsCognitoClient;
pub(crate) use aws_sdk_dynamodb::Client as AwsDynamoDbClient;

use axum::{body::Body, http::StatusCode, response::IntoResponse};
use clocks::{ClockClientDependency, ClockError};
use tokio::sync::RwLock;

use std::borrow::Cow;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use thiserror::Error;

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
pub enum AuthError {
    #[error("error with cognito interface: {0}")]
    AwsGetUser(#[from] SdkError<GetUserError, Response>),
    #[error("missing authentication cookie")]
    MissingAuthenticationCookie,
    #[error("cannot access this resource")]
    Unauthorized,
}

#[derive(Error, Debug)]
pub enum ContextError {
    #[error("could not parse url: {0}")]
    UrlParse(#[from] url::ParseError),
    #[error("could not authenticate: {0}")]
    AuthError(#[from] AuthError),
    #[error("error in clock interface: {0}")]
    ClockError(#[from] ClockError),
    #[error("error parsing body: {0}")]
    HttpBody(Cow<'static, str>)
}

#[derive(Clone, Debug)]
pub struct Context {
    aws_sdk_config: Arc<RwLock<SdkConfig>>,
    aws_dynamodb: Arc<RwLock<AwsDynamoDbClient>>,
    aws_cognito: Arc<RwLock<AwsCognitoClient>>,
    clocks_client: Arc<dyn ClockClientDependency>,
}

impl Context {
    pub async fn new(sdk_config: SdkConfig) -> Result<Self, ContextError> {
        let cognito_client = AwsCognitoClient::new(&sdk_config);

        let aws_dynamodb = Arc::new(RwLock::new(AwsDynamoDbClient::new(&sdk_config)));

        let clocks_client = clocks::v1::ClockClient::new(Arc::downgrade(&aws_dynamodb));

        Ok(Self {
            aws_sdk_config: Arc::new(RwLock::new(sdk_config)),
            aws_cognito: Arc::new(RwLock::new(cognito_client)),
            aws_dynamodb,
            clocks_client: Arc::new(clocks_client),
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

    pub async fn load_cognito_user(
        &self,
        access_token: &str,
    ) -> Result<GetUserOutput, ContextError> {
        let client_lock = self.aws_cognito.read().await;

        let get_user_output = client_lock
            .get_user()
            .access_token(access_token)
            .send()
            .await
            .map_err(AuthError::from)?;

        Ok(get_user_output)
    }

    pub fn clock_client(&self) -> &dyn ClockClientDependency {
        self.clocks_client.as_ref()
    }
}
