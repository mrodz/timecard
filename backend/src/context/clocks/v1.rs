use std::sync::Weak;

use async_trait::async_trait;
use aws_sdk_dynamodb::types::{AttributeValue, ReturnValue};
use tokio::sync::RwLock;

use crate::context::ClockError;

use super::*;

#[derive(Debug)]
pub struct ClockClient {
    dynamodb_client: Weak<RwLock<crate::context::AwsDynamoDbClient>>,
}

impl ClockClient {
    pub fn new(dynamodb_client: Weak<RwLock<crate::context::AwsDynamoDbClient>>) -> Self {
        Self { dynamodb_client }
    }
}

#[async_trait]
impl ClockClientDependency for ClockClient {
    async fn get_clocks(&self, input: GetClocksInput) -> Result<Vec<ClockSchema>, ClockError> {
        let dynamodb_client_shared = self
            .dynamodb_client
            .upgrade()
            .expect("dynamo_db_client dropped");
        let dynamodb_client = dynamodb_client_shared.read().await;

        let clocks_belonging_to_user = dynamodb_client
            .query()
            .table_name("timeclock-clocks")
            .key_condition_expression("#id = :identity_pool_user_id")
            .expression_attribute_names("#id", "identity_pool_user_id")
            .expression_attribute_values(
                ":identity_pool_user_id",
                AttributeValue::S(input.0.to_string()),
            )
            .send()
            .await?;

        let mut result = Vec::with_capacity(clocks_belonging_to_user.items().len());

        if let Some(items) = clocks_belonging_to_user.items {
            for clock in items {
                result.push(clock.try_into()?)
            }
        }

        Ok(result)
    }

    async fn create_clock(&self, input: CreateClockInput) -> Result<ClockSchema, ClockError> {
        let dynamodb_client_shared = self
            .dynamodb_client
            .upgrade()
            .expect("dynamo_db_client dropped");
        let dynamodb_client = dynamodb_client_shared.read().await;

        let to_insert: ClockSchema = input.into();

        let created_item = dynamodb_client
            .put_item()
            .table_name("timeclock-clocks")
            .set_item(Some(to_insert.into()))
            .return_values(ReturnValue::AllOld)
            .send()
            .await?;

        Ok(created_item
            .attributes
            .expect("`ReturnValue::AllOld` should have been set")
            .try_into()?)
    }
}
