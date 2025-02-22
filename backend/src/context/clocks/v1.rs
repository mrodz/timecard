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

        let mut to_insert: ClockSchema = input.into();

        to_insert.last_edit = Utc::now();

        dynamodb_client
            .put_item()
            .table_name("timeclock-clocks")
            .set_item(Some(to_insert.clone().into()))
            .send()
            .await?;

        Ok(to_insert)
    }

    async fn edit_clock(&self, input: EditClockInput) -> Result<Option<ClockSchema>, ClockError> {
        let dynamodb_client_shared = self
            .dynamodb_client
            .upgrade()
            .expect("dynamo_db_client dropped");

        let dynamodb_client = dynamodb_client_shared.read().await;

        match input.update {
            EditClockInputStrategy::Publish(clock) => {
                if input.uuid != clock.uuid {
                    return Err(ClockError::ClockNotFound(
                        clock.identity_pool_user_id,
                        input.uuid,
                    ));
                }

                let mut attributes: HashMap<_, _> = clock.into();
                let pk = attributes.remove_entry("identity_pool_user_id").unwrap();
                let sk = attributes.remove_entry("uuid").unwrap();

                let clock = dynamodb_client
                    .update_item()
                    .table_name("timeclock-clocks")
                    .set_key(Some([pk, sk].into()))
                    .update_expression("SET #name=:name, #active=:active, #clock_in_time=:clock_in_time, #last_edit=:last_edit")
                    .expression_attribute_values(":name", attributes.remove("name").unwrap())
                    .expression_attribute_values(":active", attributes.remove("active").unwrap())
                    .expression_attribute_values(":clock_in_time", attributes.remove("clock_in_time").unwrap())
                    .expression_attribute_values(":last_edit", AttributeValue::S(Utc::now().to_rfc3339()))
                    .expression_attribute_names("#name", "name")
                    .expression_attribute_names("#active", "active")
                    .expression_attribute_names("#clock_in_time", "clock_in_time")
                    .expression_attribute_names("#last_edit", "last_edit")
                    .return_values(ReturnValue::AllNew)
                    .send()
                    .await?;

                let attributes = clock
                    .attributes
                    .expect("`ReturnValue::AllNew` should have been set");

                Ok(Some(attributes.try_into()?))
            }
            EditClockInputStrategy::Fields {
                identity_pool_user_id,
                name,
                active,
                clock_in_time,
            } => {
                let pk = AttributeValue::S(identity_pool_user_id.to_string());
                let sk = AttributeValue::S(input.uuid.to_string());

                let mut update_expression = "SET #last_edit=:last_edit".to_owned();

                let mut query = dynamodb_client
                    .update_item()
                    .table_name("timeclock-clocks")
                    .set_key(Some(
                        [
                            ("identity_pool_user_id".to_owned(), pk),
                            ("uuid".to_owned(), sk),
                        ]
                        .into(),
                    ))
                    .expression_attribute_values(
                        ":last_edit",
                        AttributeValue::S(Utc::now().to_rfc3339()),
                    )
                    .expression_attribute_names("#last_edit", "last_edit")
                    .return_values(ReturnValue::AllNew);

                let mut edits = 0;

                if let Some(name) = name {
                    update_expression += ", #name=:name";
                    query = query
                        .expression_attribute_values(":name", AttributeValue::S(name))
                        .expression_attribute_names("#name", "name");
                    edits += 1;
                }

                if let Some(active) = active {
                    update_expression += ", #active=:active";
                    query = query
                        .expression_attribute_values(":active", AttributeValue::Bool(active))
                        .expression_attribute_names("#active", "active");
                    edits += 1;
                }

                if let Some(clock_in_time) = clock_in_time {
                    update_expression += ", #clock_in_time=:clock_in_time";
                    query = query
                        .expression_attribute_values(
                            ":clock_in_time",
                            AttributeValue::S(
                                clock_in_time
                                    .as_ref()
                                    .map(DateTime::to_rfc3339)
                                    .unwrap_or_else(String::new),
                            ),
                        )
                        .expression_attribute_names("#clock_in_time", "clock_in_time");
                    edits += 1;
                }

                if edits == 0 {
                    return Ok(None);
                }

                let updated_clock = query
                    .update_expression(update_expression)
                    .send()
                    .await?
                    .attributes
                    .expect("`ReturnValue::AllNew` should have been set");

                Ok(Some(updated_clock.try_into()?))
            }
        }
    }

    async fn validate_user_claims_to_clock(
        &self,
        input: ValidateUserClaimsToClockInput,
    ) -> Result<ClockSchema, ClockError> {
        let dynamodb_client_shared = self
            .dynamodb_client
            .upgrade()
            .expect("dynamo_db_client dropped");

        let dynamodb_client = dynamodb_client_shared.read().await;

        let maybe_clock = dynamodb_client
            .get_item()
            .table_name("timeclock-clocks")
            .set_key(Some(
                [
                    (
                        "identity_pool_user_id".to_owned(),
                        AttributeValue::S(input.identity_pool_user_id.to_string()),
                    ),
                    ("uuid".to_owned(), AttributeValue::S(input.uuid.to_string())),
                ]
                .into(),
            ))
            .send()
            .await?;

        let Some(clock_attributes) = maybe_clock.item else {
            return Err(ClockError::ClockNotFound(
                input.identity_pool_user_id,
                input.uuid,
            ));
        };

        Ok(clock_attributes.try_into()?)
    }
}
