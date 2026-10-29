use crate::models::{
    Controller, ControllerWithReadings, CreateController, CreateHumidityReadingByUuid,
};
use crate::{AppError, AppResult};
use salvo::oapi::extract::{JsonBody, PathParam, QueryParam};
use salvo::prelude::*;
use serde::Serialize;
use uuid::Uuid;

static LAST_UUID: std::sync::Mutex<Option<Uuid>> = std::sync::Mutex::new(None);

pub fn router() -> Router {
    Router::with_path("controllers")
        .oapi_tags(["Controllers"])
        .push(
            Router::new()
                .post(create_controller)
                .get(get_all_controllers),
        )
        .push(
            Router::with_path("{id}")
                .get(get_controller)
                .put(update_controller)
                .delete(delete_controller)
                .push(Router::with_path("readings").get(get_controller_with_readings))
                .push(Router::with_path("readings/recent").get(get_controller_recent_readings))
                .push(Router::with_path("reclaim-uuid").post(reclaim_controller_uuid)),
        )
        .push(Router::with_path("uuid/claim-latest").get(claim_latest_uuid))
        .push(
            Router::with_path("uuid/{uuid}")
                .get(get_controller_by_uuid)
                .push(
                    Router::with_path("readings")
                        .get(get_controller_with_readings_by_uuid)
                        .post(create_reading_by_uuid),
                ),
        )
}

#[derive(Serialize, ToSchema)]
struct ControllerId {
    controller_id: i32,
}

#[derive(Serialize, ToSchema)]
struct LatestUuidResponse {
    uuid: Option<Uuid>,
}

#[endpoint]
async fn claim_latest_uuid() -> AppResult<Json<LatestUuidResponse>> {
    let uuid = LAST_UUID
        .lock()
        .map_err(|_| AppError::Internal("fail".into()))?
        .take();
    Ok(Json(LatestUuidResponse { uuid }))
}

#[endpoint]
async fn reclaim_controller_uuid(id: PathParam<i32>) -> AppResult<Json<LatestUuidResponse>> {
    let controller = Controller::get_by_id(*id)
        .await?
        .ok_or(crate::error::AppError::NotFound(
            "Controller not found".to_string(),
        ))?;

    LAST_UUID
        .lock()
        .map_err(|_| AppError::Internal("fail".into()))?
        .replace(controller.uuid);

    Ok(Json(LatestUuidResponse {
        uuid: Some(controller.uuid),
    }))
}

#[endpoint]
async fn create_controller(
    controller: JsonBody<CreateController>,
) -> AppResult<Json<ControllerId>> {
    let JsonBody(controller) = controller;

    let new_controller = Controller::create(&controller).await?;

    let _ = LAST_UUID
        .lock()
        .map_err(|_| AppError::Internal("fail".into()))?
        .replace(new_controller.uuid);

    Ok(Json(ControllerId {
        controller_id: new_controller.controller_id,
    }))
}

#[endpoint]
async fn get_controller(id: PathParam<i32>) -> AppResult<Json<Controller>> {
    let controller = Controller::get_by_id(*id)
        .await?
        .ok_or(crate::error::AppError::NotFound(
            "Controller not found".to_string(),
        ))?;

    Ok(Json(controller))
}

#[endpoint]
async fn get_all_controllers() -> AppResult<Json<Vec<Controller>>> {
    let controllers = Controller::get_all().await?;
    Ok(Json(controllers))
}

#[endpoint]
async fn update_controller(
    id: PathParam<i32>,
    controller: JsonBody<CreateController>,
) -> AppResult<Json<Controller>> {
    let JsonBody(controller) = controller;

    let updated_controller = Controller::update(*id, &controller).await?;

    Ok(Json(updated_controller))
}

#[endpoint]
async fn delete_controller(id: PathParam<i32>) -> AppResult<()> {
    let deleted = Controller::delete(*id).await?;
    if !deleted {
        return Err(crate::error::AppError::NotFound(
            "Controller not found".to_string(),
        ));
    }

    Ok(())
}

#[endpoint]
async fn get_controller_with_readings(
    id: PathParam<i32>,
    limit: QueryParam<Option<i64>>,
) -> AppResult<Json<ControllerWithReadings>> {
    let limit = limit.into_inner();

    let controller_with_readings = ControllerWithReadings::get_by_controller_id(*id, limit)
        .await?
        .ok_or(crate::error::AppError::NotFound(
            "Controller not found".to_string(),
        ))?;

    Ok(Json(controller_with_readings))
}

#[endpoint]
async fn get_controller_by_uuid(uuid: PathParam<Uuid>) -> AppResult<Json<Controller>> {
    let controller = Controller::get_by_uuid(uuid.into_inner()).await?.ok_or(
        crate::error::AppError::NotFound("Controller not found".to_string()),
    )?;

    Ok(Json(controller))
}

#[endpoint]
async fn get_controller_with_readings_by_uuid(
    uuid: PathParam<Uuid>,
    limit: QueryParam<Option<i64>>,
) -> AppResult<Json<ControllerWithReadings>> {
    let limit = limit.into_inner();
    let uuid = uuid.into_inner();

    let controller = match Controller::get_by_uuid(uuid).await? {
        Some(c) => c,
        None => {
            return Err(crate::error::AppError::NotFound(
                "Controller not found".to_string(),
            ));
        }
    };

    let readings =
        crate::models::HumidityReading::get_by_controller(controller.controller_id, limit, Some(0))
            .await?;

    Ok(Json(ControllerWithReadings {
        controller,
        readings,
    }))
}

#[endpoint]
async fn create_reading_by_uuid(
    uuid: PathParam<Uuid>,
    reading: JsonBody<CreateHumidityReadingByUuid>,
) -> AppResult<Json<crate::models::HumidityReading>> {
    let JsonBody(reading) = reading;
    let uuid = uuid.into_inner();

    let created = crate::models::HumidityReading::create_by_uuid(
        uuid,
        reading.humidity,
        reading.battery_voltage,
        reading.rssi,
        reading.is_alert,
    )
    .await?;

    Ok(Json(created))
}

#[endpoint]
async fn get_controller_recent_readings(
    id: PathParam<i32>,
    hours: QueryParam<Option<i32>>,
) -> AppResult<Json<ControllerWithReadings>> {
    let hours = hours.into_inner();
    let hours = hours.unwrap_or(24);

    let controller_with_readings = ControllerWithReadings::get_recent_by_controller_id(*id, hours)
        .await?
        .ok_or(crate::error::AppError::NotFound(
            "Controller not found".to_string(),
        ))?;

    Ok(Json(controller_with_readings))
}
