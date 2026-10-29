use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use salvo::prelude::*;
use serde::{Deserialize, Serialize};
use sqlx::prelude::*;
use uuid::Uuid;
use validator::Validate;

#[derive(FromRow, Serialize, ToSchema, Debug)]
pub struct Controller {
    pub controller_id: i32,
    pub uuid: Uuid,
    pub name: String,
    pub min_humidity: Option<rust_decimal::Decimal>,
    pub max_humidity: Option<rust_decimal::Decimal>,
    pub location: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize, ToSchema, Debug, Validate)]
pub struct CreateController {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    pub min_humidity: Option<rust_decimal::Decimal>,
    pub max_humidity: Option<rust_decimal::Decimal>,
    #[validate(length(min = 1, max = 100))]
    pub location: String,
}

#[derive(FromRow, Serialize, ToSchema, Debug)]
pub struct HumidityReading {
    pub time: DateTime<Utc>,
    pub controller_id: i32,
    pub humidity: rust_decimal::Decimal,
    pub battery_voltage: Option<rust_decimal::Decimal>,
    pub rssi: Option<i16>,
    pub is_alert: bool,
}

#[derive(Deserialize, ToSchema, Debug)]
pub struct CreateHumidityReading {
    pub controller_id: i32,
    pub humidity: rust_decimal::Decimal,
    pub battery_voltage: Option<rust_decimal::Decimal>,
    pub rssi: Option<i16>,
    pub is_alert: Option<bool>,
}

#[derive(Deserialize, ToSchema, Debug)]
pub struct CreateHumidityReadingByUuid {
    pub humidity: Decimal,
    pub battery_voltage: Option<Decimal>,
    pub rssi: Option<i16>,
    pub is_alert: Option<bool>,
}

#[derive(Serialize, ToSchema, Debug)]
pub struct ControllerWithReadings {
    #[serde(flatten)]
    pub controller: Controller,
    pub readings: Vec<HumidityReading>,
}
