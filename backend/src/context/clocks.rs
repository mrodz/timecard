pub mod v1;

use std::{collections::HashMap, fmt::Debug};

use async_trait::async_trait;
use aws_sdk_dynamodb::{error::SdkError, operation::query::QueryError, types::AttributeValue};
use aws_smithy_runtime_api::http::Response;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetClocksInput(pub Uuid);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClockSchema {
    /// Partition key
    pub identity_pool_user_id: String,
    /// Sort key
    pub uuid: String,
    pub name: String,
    pub last_edit: String,
    pub active: bool,
    pub clock_in_time: Option<String>,
}

#[derive(Error, Debug)]
pub enum ClockError {
    #[error("error with dynamodb interface: {0}")]
    AwsDynamodb(#[from] SdkError<QueryError, Response>),
    #[error("could not parse field `{0}`, `ClockSchema` from unstructured object: {1:?}")]
    ParseMalformedQuery(String, HashMap<String, AttributeValue>),
}

impl TryFrom<HashMap<String, AttributeValue>> for ClockSchema {
    type Error = ClockError;

    fn try_from(mut value: HashMap<String, AttributeValue>) -> Result<Self, Self::Error> {
        let Some(AttributeValue::S(identity_pool_user_id)) = value.remove("identity_pool_user_id")
        else {
            unreachable!("should have AWS managed table key: `identity_pool_user_id`");
        };

        let Some(AttributeValue::S(uuid)) = value.remove("uuid") else {
            unreachable!("should have AWS managed table key: `uuid`");
        };

        let Some(AttributeValue::S(name)) = value.remove("name") else {
            return Err(ClockError::ParseMalformedQuery("name".into(), value));
        };

        let Some(AttributeValue::S(last_edit)) = value.remove("last_edit") else {
            return Err(ClockError::ParseMalformedQuery("last_edit".into(), value));
        };

        let Some(AttributeValue::Bool(active)) = value.remove("active") else {
            return Err(ClockError::ParseMalformedQuery("active".into(), value));
        };

        let clock_in_time = match value.remove("clock_in_time") {
            Some(AttributeValue::S(x)) => Some(x),
            Some(AttributeValue::Null(_)) => None,
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
}
