use crate::AppState;
use crate::domain::models::dto::params::{CreateProductParams, UpdateProductParams};
use crate::domain::services::ServiceError;
use axum::{Json, extract::Path, extract::Query, extract::State};
use serde::Deserialize;
use serde_json::Value;
use sqlx::types::BigDecimal;

#[derive(Deserialize)]
pub struct CreateProductReq {
    pub product_name: String,
    pub category_id: Option<i32>,
    pub category_name: Option<String>,
    pub spec_template: Option<Value>,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub base_price: BigDecimal,
}

#[derive(Deserialize)]
pub struct UpdateProductReq {
    pub product_name: Option<String>,
    pub category_id: Option<i32>,
    pub category_name: Option<String>,
    pub spec_template: Option<Value>,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub base_price: Option<BigDecimal>,
    pub status: Option<i8>,
}

#[derive(Deserialize)]
pub struct ListProductsReq {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub product_name: Option<String>,
}

pub async fn list_products_handler(
    State(app_state): State<AppState>,
    Query(params): Query<ListProductsReq>,
) -> Result<Json<Value>, ServiceError> {
    let page = params.page.unwrap_or(1);
    let page_size = params.page_size.unwrap_or(10);
    let (products, total) = app_state.product_service.list_products(page, page_size, params.product_name).await?;
    Ok(Json(serde_json::json!({
        "code": 0,
        "msg": "success",
        "data": products,
        "pagination": {
            "page": page,
            "page_size": page_size,
            "total": total
        }
    })))
}

pub async fn create_product_handler(
    State(app_state): State<AppState>,
    Json(req): Json<CreateProductReq>,
) -> Result<Json<Value>, ServiceError> {
    let params = CreateProductParams {
        product_name: req.product_name,
        category_id: req.category_id,
        category_name: req.category_name,
        spec_template: req.spec_template,
        description: req.description,
        image_url: req.image_url,
        base_price: req.base_price,
    };
    let product_id = app_state.product_service.create_product(params).await?;
    Ok(Json(serde_json::json!({
        "code": 0,
        "msg": "success",
        "data": {
            "product_id": product_id
        }
    })))
}

pub async fn update_product_handler(
    State(app_state): State<AppState>,
    Path(product_id): Path<i32>,
    Json(req): Json<UpdateProductReq>,
) -> Result<Json<Value>, ServiceError> {
    let params = UpdateProductParams {
        product_name: req.product_name,
        category_id: req.category_id,
        category_name: req.category_name,
        spec_template: req.spec_template,
        description: req.description,
        image_url: req.image_url,
        base_price: req.base_price,
        status: req.status,
    };
    app_state.product_service.update_product(product_id, params).await?;
    Ok(Json(serde_json::json!({
        "code": 0,
        "msg": "success"
    })))
}

pub async fn off_shelf_product_handler(
    State(app_state): State<AppState>,
    Path(product_id): Path<i32>,
) -> Result<Json<Value>, ServiceError> {
    let params = UpdateProductParams {
        product_name: None,
        category_id: None,
        category_name: None,
        spec_template: None,
        description: None,
        image_url: None,
        base_price: None,
        status: Some(0), // 0: off-shelf
    };
    app_state.product_service.update_product(product_id, params).await?;
    Ok(Json(serde_json::json!({
        "code": 0,
        "msg": "success"
    })))
}

use axum::extract::Multipart;

pub async fn upload_image_handler(
    State(app_state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<Value>, ServiceError> {
    while let Some(field) = multipart.next_field().await.map_err(|e| ServiceError::Internal(e.to_string()))? {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" {
             let file_name = field.file_name().unwrap_or("unknown.jpg").to_string();
             
             // Generate unique filename using UUID
             let extension = std::path::Path::new(&file_name)
                 .extension()
                 .and_then(|ext| ext.to_str())
                 .unwrap_or("jpg");
             let new_file_name = format!("{}.{}", uuid::Uuid::new_v4(), extension);
             
             let content_type = field.content_type().unwrap_or("application/octet-stream").to_string();
             let data = field.bytes().await.map_err(|e| ServiceError::Internal(e.to_string()))?;
             
             let key = app_state.product_service.upload_image(new_file_name, data.to_vec(), content_type).await?;
             return Ok(Json(serde_json::json!({
                 "code": 0,
                 "msg": "success",
                 "data": { "key": key }
             })));
        }
    }
    Err(ServiceError::Internal("No file found".into()))
}
