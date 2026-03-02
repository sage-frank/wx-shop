use crate::AppState;
use crate::services::ServiceError;
use axum::{Json, extract::Path, extract::State};
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
pub struct CreateOrderItemReq {
    pub product_id: i32,
    pub product_name: String,
    pub quantity: i32,
    pub unit_price: f64,
    pub spec_info: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateOrderReq {
    pub user_id: i32,
    pub pay_amount: Option<f64>,
    pub order_status: Option<i8>,
    pub consignee_info: Option<Value>,
    pub items: Vec<CreateOrderItemReq>,
}

#[derive(Deserialize)]
pub struct UpdateOrderReq {
    pub pay_amount: Option<f64>,
    pub order_status: Option<i8>,
    pub consignee_info: Option<Value>,
}

pub async fn create_order_handler(
    State(app_state): State<AppState>,
    Json(payload): Json<CreateOrderReq>,
) -> Result<Json<serde_json::Value>, ServiceError> {
    let items = payload
        .items
        .into_iter()
        .map(|i| {
            (
                i.product_id,
                i.product_name,
                i.quantity,
                i.unit_price,
                i.spec_info,
            )
        })
        .collect();
    let status = payload.order_status.unwrap_or(0);
    match app_state
        .order_service
        .create_order_with_items(crate::services::orders::CreateOrderWithItemsArgs {
            user_id: payload.user_id,
            pay_amount: payload.pay_amount,
            order_status: status,
            consignee_info: payload.consignee_info,
            items,
        })
        .await
    {
        Ok(order_id) => Ok(Json(
            serde_json::json!({"code":0,"msg":"success","data":{"order_id":order_id}}),
        )),
        Err(e) => Err(e),
    }
}

pub async fn delete_order_handler(
    State(app_state): State<AppState>,
    Path(order_id): Path<String>,
) -> Result<Json<serde_json::Value>, ServiceError> {
    match app_state.order_service.delete_order(&order_id).await {
        Ok(_) => Ok(Json(serde_json::json!({"code":0,"msg":"success"}))),
        Err(e) => Err(e),
    }
}

pub async fn update_order_handler(
    State(app_state): State<AppState>,
    Path(order_id): Path<String>,
    Json(payload): Json<UpdateOrderReq>,
) -> Result<Json<serde_json::Value>, ServiceError> {
    match app_state
        .order_service
        .update_order(
            &order_id,
            payload.pay_amount,
            payload.order_status,
            payload.consignee_info,
        )
        .await
    {
        Ok(_) => Ok(Json(serde_json::json!({"code":0,"msg":"success"}))),
        Err(e) => Err(e),
    }
}

pub async fn get_order_handler(
    State(app_state): State<AppState>,
    Path(order_id): Path<String>,
) -> Result<Json<serde_json::Value>, ServiceError> {
    let order = app_state.order_service.get_order(&order_id).await?;
    Ok(Json(
        serde_json::json!({"code":0,"msg":"success","data":order}),
    ))
}

pub async fn get_order_items_handler(
    State(app_state): State<AppState>,
    Path(order_id): Path<String>,
) -> Result<Json<serde_json::Value>, ServiceError> {
    let items = app_state.order_service.get_order_items(&order_id).await?;
    Ok(Json(
        serde_json::json!({"code":0,"msg":"success","data":items}),
    ))
}

pub async fn cancel_order_handler(
    State(app_state): State<AppState>,
    Path(order_id): Path<String>,
) -> Result<Json<serde_json::Value>, ServiceError> {
    match app_state.order_service.cancel_order(&order_id).await {
        Ok(_) => Ok(Json(serde_json::json!({"code":0,"msg":"success"}))),
        Err(e) => Err(e),
    }
}
