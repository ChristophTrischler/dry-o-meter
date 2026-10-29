use crate::db::pool;
use crate::models::{
    Controller, ControllerWithReadings, CreateController, CreateHumidityReading, HumidityReading,
};
use anyhow::{Ok, Result};
use chrono::{Duration, Utc};
use uuid::Uuid;

// Controller queries
impl Controller {
    pub async fn create(create_controller: &CreateController) -> Result<Controller> {
        let row = sqlx::query!(
            r#"
            INSERT INTO controllers (name, min_humidity, max_humidity, location)
            VALUES ($1, $2, $3, $4)
            RETURNING controller_id, uuid, name, min_humidity, max_humidity, location, created_at
            "#,
            create_controller.name,
            create_controller.min_humidity,
            create_controller.max_humidity,
            create_controller.location
        )
        .fetch_one(pool())
        .await?;

        Ok(Controller {
            controller_id: row.controller_id,
            uuid: row.uuid,
            name: row.name,
            min_humidity: row.min_humidity,
            max_humidity: row.max_humidity,
            location: row.location,
            created_at: row.created_at,
        })
    }

    pub async fn get_by_id(controller_id: i32) -> Result<Option<Controller>> {
        let row = sqlx::query_as!(
            Controller,
            r#"
            SELECT controller_id, uuid, name, min_humidity, max_humidity, location, created_at
            FROM controllers
            WHERE controller_id = $1
            "#,
            controller_id
        )
        .fetch_optional(pool())
        .await?;
        Ok(row)
    }

    pub async fn get_all() -> Result<Vec<Controller>> {
        let rows = sqlx::query_as!(
            Controller,
            r#"
            SELECT controller_id, uuid, name, min_humidity, max_humidity, location, created_at
            FROM controllers
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(pool())
        .await?;

        Ok(rows)
    }

    pub async fn update(
        controller_id: i32,
        update_controller: &CreateController,
    ) -> Result<Controller> {
        let row = sqlx::query_as!(
            Controller,
            r#"
            UPDATE controllers
            SET name = $1, min_humidity = $2, max_humidity = $3, location = $4
            WHERE controller_id = $5
            RETURNING controller_id, uuid, name, min_humidity, max_humidity, location, created_at
            "#,
            update_controller.name,
            update_controller.min_humidity,
            update_controller.max_humidity,
            update_controller.location,
            controller_id
        )
        .fetch_one(pool())
        .await?;

        Ok(row)
    }

    pub async fn delete(controller_id: i32) -> Result<bool> {
        let result = sqlx::query!(
            "DELETE FROM controllers WHERE controller_id = $1",
            controller_id
        )
        .execute(pool())
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn get_by_uuid(uuid: Uuid) -> Result<Option<Controller>> {
        let row = sqlx::query_as!(
            Controller,
            r#"
            SELECT controller_id, uuid, name, min_humidity, max_humidity, location, created_at
            FROM controllers
            WHERE uuid = $1
            "#,
            uuid
        )
        .fetch_optional(pool())
        .await?;
        Ok(row)
    }
}

// HumidityReading queries
impl HumidityReading {
    pub async fn create(create_reading: &CreateHumidityReading) -> Result<HumidityReading> {
        let time = Utc::now();
        let is_alert = create_reading.is_alert.unwrap_or(false);

        let row = sqlx::query!(
            r#"
            INSERT INTO humidity_readings (time, controller_id, humidity, battery_voltage, rssi, is_alert)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING time, controller_id, humidity, battery_voltage, rssi, is_alert
            "#,
            time,
            create_reading.controller_id,
            create_reading.humidity,
            create_reading.battery_voltage,
            create_reading.rssi,
            is_alert
        )
        .fetch_one(pool())
        .await?;

        Ok(HumidityReading {
            time: row.time,
            controller_id: row.controller_id,
            humidity: row.humidity,
            battery_voltage: row.battery_voltage,
            rssi: row.rssi,
            is_alert: row.is_alert,
        })
    }

    pub async fn create_by_uuid(
        uuid: Uuid,
        humidity: rust_decimal::Decimal,
        battery_voltage: Option<rust_decimal::Decimal>,
        rssi: Option<i16>,
        is_alert: Option<bool>,
    ) -> Result<HumidityReading> {
        let time = Utc::now();
        let is_alert = is_alert.unwrap_or(false);

        let controller = Controller::get_by_uuid(uuid)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Controller not found"))?;

        let row = sqlx::query!(
            r#"
            INSERT INTO humidity_readings (time, controller_id, humidity, battery_voltage, rssi, is_alert)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING time, controller_id, humidity, battery_voltage, rssi, is_alert
            "#,
            time,
            controller.controller_id,
            humidity,
            battery_voltage,
            rssi,
            is_alert
        )
        .fetch_one(pool())
        .await?;

        Ok(HumidityReading {
            time: row.time,
            controller_id: row.controller_id,
            humidity: row.humidity,
            battery_voltage: row.battery_voltage,
            rssi: row.rssi,
            is_alert: row.is_alert,
        })
    }

    pub async fn get_by_controller(
        controller_id: i32,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<HumidityReading>> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);

        let rows = sqlx::query!(
            r#"
            SELECT time, controller_id, humidity, battery_voltage, rssi, is_alert
            FROM humidity_readings
            WHERE controller_id = $1
            ORDER BY time DESC
            LIMIT $2 OFFSET $3
            "#,
            controller_id,
            limit,
            offset
        )
        .fetch_all(pool())
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| HumidityReading {
                time: row.time,
                controller_id: row.controller_id,
                humidity: row.humidity,
                battery_voltage: row.battery_voltage,
                rssi: row.rssi,
                is_alert: row.is_alert,
            })
            .collect())
    }

    pub async fn get_recent_by_controller(
        controller_id: i32,
        hours: i32,
    ) -> Result<Vec<HumidityReading>> {
        let since = Utc::now() - Duration::hours(hours as i64);

        let rows = sqlx::query!(
            r#"
            SELECT time, controller_id, humidity, battery_voltage, rssi, is_alert
            FROM humidity_readings
            WHERE controller_id = $1 AND time >= $2
            ORDER BY time DESC
            "#,
            controller_id,
            since
        )
        .fetch_all(pool())
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| HumidityReading {
                time: row.time,
                controller_id: row.controller_id,
                humidity: row.humidity,
                battery_voltage: row.battery_voltage,
                rssi: row.rssi,
                is_alert: row.is_alert,
            })
            .collect())
    }

    pub async fn get_alerts(limit: Option<i64>) -> Result<Vec<HumidityReading>> {
        let limit = limit.unwrap_or(50);

        let rows = sqlx::query!(
            r#"
            SELECT time, controller_id, humidity, battery_voltage, rssi, is_alert
            FROM humidity_readings
            WHERE is_alert = TRUE
            ORDER BY time DESC
            LIMIT $1
            "#,
            limit
        )
        .fetch_all(pool())
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| HumidityReading {
                time: row.time,
                controller_id: row.controller_id,
                humidity: row.humidity,
                battery_voltage: row.battery_voltage,
                rssi: row.rssi,
                is_alert: row.is_alert,
            })
            .collect())
    }

    pub async fn get_latest_by_controller(controller_id: i32) -> Result<Option<HumidityReading>> {
        let row = sqlx::query!(
            r#"
            SELECT time, controller_id, humidity, battery_voltage, rssi, is_alert
            FROM humidity_readings
            WHERE controller_id = $1
            ORDER BY time DESC
            LIMIT 1
            "#,
            controller_id
        )
        .fetch_optional(pool())
        .await?;

        Ok(row.map(|row| HumidityReading {
            time: row.time,
            controller_id: row.controller_id,
            humidity: row.humidity,
            battery_voltage: row.battery_voltage,
            rssi: row.rssi,
            is_alert: row.is_alert,
        }))
    }
}

// ControllerWithReadings queries
impl ControllerWithReadings {
    pub async fn get_by_controller_id(
        controller_id: i32,
        reading_limit: Option<i64>,
    ) -> Result<Option<ControllerWithReadings>> {
        let controller = match Controller::get_by_id(controller_id).await? {
            Some(c) => c,
            None => return Ok(None),
        };

        let readings =
            HumidityReading::get_by_controller(controller_id, reading_limit, Some(0)).await?;

        Ok(Some(ControllerWithReadings {
            controller,
            readings,
        }))
    }

    pub async fn get_recent_by_controller_id(
        controller_id: i32,
        hours: i32,
    ) -> Result<Option<ControllerWithReadings>> {
        let controller = match Controller::get_by_id(controller_id).await? {
            Some(c) => c,
            None => return Ok(None),
        };

        let readings = HumidityReading::get_recent_by_controller(controller_id, hours).await?;

        Ok(Some(ControllerWithReadings {
            controller,
            readings,
        }))
    }
}
