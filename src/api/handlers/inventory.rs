use crate::domain::models::dto::params::UpdateInventoryParams;
use crate::domain::services::ServiceError;
use crate::AppState;



use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Deserialize)]
pub struct ListInventoryReq {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub product_name: Option<String>,
}

pub async fn list_inventory_handler(
    State(app_state): State<AppState>,
    Query(params): Query<ListInventoryReq>,
) -> Result<Json<Value>, ServiceError> {
    let page = params.page.unwrap_or(1);
    let page_size = params.page_size.unwrap_or(10);
    let (inventory, total) = app_state.inventory_service.list_inventory(page, page_size, params.product_name).await?;
    
    Ok(Json(json!({
        "code": 0,
        "msg": "success",
        "data": inventory,
        "pagination": {
            "page": page,
            "page_size": page_size,
            "total": total
        }
    })))
}

pub async fn update_inventory_handler(
    State(app_state): State<AppState>,
    Path(inv_id): Path<i32>,
    Json(params): Json<UpdateInventoryParams>,
) -> Result<Json<Value>, ServiceError> {
    app_state.inventory_service.update_inventory(inv_id, params).await?;
    
    Ok(Json(json!({
        "code": 0,
        "msg": "success"
    })))
}

pub async fn get_inventory_handler(
    State(app_state): State<AppState>,
    Path(inv_id): Path<i32>,
) -> Result<Json<Value>, ServiceError> {
    let inventory = app_state.inventory_service.get_inventory(inv_id).await?;
    
    Ok(Json(json!({
        "code": 0,
        "msg": "success",
        "data": inventory
    })))
}
