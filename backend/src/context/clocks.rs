pub mod v1;

use std::{collections::HashMap, fmt::Debug};

use async_trait::async_trait;
use aws_sdk_dynamodb::{error::SdkError, operation::{get_item::GetItemError, put_item::PutItemError, query::QueryError, update_item::UpdateItemError}, types::AttributeValue};
use aws_smithy_runtime_api::http::Response;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetClocksInput(pub Uuid);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateClockInput {
    pub identity_pool_user_id: Uuid,
    pub name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EditClockInputStrategy {
    Fields {
        identity_pool_user_id: Uuid,
        name: Option<String>,
        active: Option<bool>,
        clock_in_time: Option<Option<DateTime<Utc>>>,
    },
    Publish(ClockSchema),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EditClockInput {
    pub uuid: Uuid,
    pub update: EditClockInputStrategy,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidateUserClaimsToClockInput {
    pub identity_pool_user_id: Uuid,
    pub uuid: Uuid,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClockSchema {
    /// Partition key
    pub identity_pool_user_id: Uuid,
    /// Sort key
    pub uuid: Uuid,
    pub name: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub last_edit: DateTime<Utc>,
    pub active: bool,
    #[serde(with = "chrono::serde::ts_seconds_option")]
    pub clock_in_time: Option<DateTime<Utc>>,
}

#[derive(Error, Debug)]
pub enum ClockError {
    #[error("error with dynamodb QUERY interface: {0}")]
    AwsDynamodbQuery(#[from] SdkError<QueryError, Response>),
    #[error("error with dynamodb PUT interface: {0}")]
    AwsDynamodbPut(#[from] SdkError<PutItemError, Response>),
    #[error("error with dynamodb GET interface: {0}")]
    AwsDynamodbGet(#[from] SdkError<GetItemError, Response>),
    #[error("error with dynamodb UPDATE interface: {0}")]
    AwsDynamodbUpdate(#[from] SdkError<UpdateItemError, Response>),
    #[error("could not parse field `{0}`, `ClockSchema` from unstructured object: {1:?}")]
    ParseMalformedQuery(String, HashMap<String, AttributeValue>),
    #[error("could not parse clock date string: {0}")]
    ParseTimestamp(#[from] chrono::ParseError),
    #[error("could not parse clock uuid: {0}")]
    ParseUuid(#[from] uuid::Error),
    #[error("could not find user({0})->clock({1})")]
    ClockNotFound(Uuid, Uuid),
}

impl From<CreateClockInput> for ClockSchema {
    fn from(value: CreateClockInput) -> Self {
        Self {
            active: false,
            clock_in_time: None,
            last_edit: Utc::now(),
            uuid: Uuid::new_v4(),
            identity_pool_user_id: value.identity_pool_user_id,
            name: value.name,
        }
    }
}

impl From<ClockSchema> for HashMap<String, AttributeValue> {
    fn from(value: ClockSchema) -> Self {
        let attributes = [
            ("identity_pool_user_id".to_owned(), AttributeValue::S(value.identity_pool_user_id.to_string())),
            ("uuid".to_owned(), AttributeValue::S(value.uuid.to_string())),
            ("name".to_owned(), AttributeValue::S(value.name)),
            ("last_edit".to_owned(), AttributeValue::S(value.last_edit.to_rfc3339())),
            ("active".to_owned(), AttributeValue::Bool(value.active)),
            ("clock_in_time".to_owned(), match value.clock_in_time {
                None => AttributeValue::Null(true),
                Some(date) => AttributeValue::S(date.to_rfc3339()),
            }),
        ];

        let mut result = HashMap::with_capacity(attributes.len());
        result.extend(attributes);

        result
    }
}

impl TryFrom<HashMap<String, AttributeValue>> for ClockSchema {
    type Error = ClockError;

    fn try_from(mut value: HashMap<String, AttributeValue>) -> Result<Self, Self::Error> {
        let Some(AttributeValue::S(identity_pool_user_id)) = value.remove("identity_pool_user_id")
        else {
            unreachable!("should have AWS managed table key: `identity_pool_user_id`");
        };

        let identity_pool_user_id = Uuid::parse_str(&identity_pool_user_id)?;

        let Some(AttributeValue::S(uuid)) = value.remove("uuid") else {
            unreachable!("should have AWS managed table key: `uuid`");
        };

        let uuid = Uuid::parse_str(&uuid)?;

        let Some(AttributeValue::S(name)) = value.remove("name") else {
            return Err(ClockError::ParseMalformedQuery("name".into(), value));
        };

        let Some(AttributeValue::S(last_edit)) = value.remove("last_edit") else {
            return Err(ClockError::ParseMalformedQuery("last_edit".into(), value));
        };

        let last_edit = DateTime::parse_from_rfc3339(&last_edit)?.to_utc();

        let Some(AttributeValue::Bool(active)) = value.remove("active") else {
            return Err(ClockError::ParseMalformedQuery("active".into(), value));
        };

        let clock_in_time = match value.remove("clock_in_time") {
            Some(AttributeValue::Null(_)) => None,
            Some(AttributeValue::S(x)) if x.is_empty() => None,
            Some(AttributeValue::S(x)) => Some(DateTime::parse_from_rfc3339(&x)?.to_utc()),
            _ => {
                return Err(ClockError::ParseMalformedQuery(
                    "clock_in_time".into(),
                    value,
                ))
            }
        };

        Ok(Self {
            active: active,
            clock_in_time,
            identity_pool_user_id,
            last_edit,
            name,
            uuid,
        })
    }
}

#[async_trait]
pub trait ClockClientDependency
where
    Self: Debug + Send + Sync,
{
    async fn get_clocks(&self, input: GetClocksInput) -> Result<Vec<ClockSchema>, ClockError>;
    async fn create_clock(&self, input: CreateClockInput) -> Result<ClockSchema, ClockError>;
    async fn edit_clock(&self, input: EditClockInput) -> Result<Option<ClockSchema>, ClockError>;
    async fn validate_user_claims_to_clock(&self, input: ValidateUserClaimsToClockInput) -> Result<ClockSchema, ClockError>;
}
