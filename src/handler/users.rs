// use crate::service::users::UserService;
use crate::AppState;
use crate::service::ServiceError;
use axum::extract::Path;
use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;
use serde_json;
use sha2::{Digest, Sha256};

#[derive(Deserialize)]
pub struct HashReq {
    pub passwd: String,
    pub salt: String,
}

pub async fn hash_handler(Json(payload): Json<HashReq>) -> Json<serde_json::Value> {
    let mut hasher = Sha256::new();
    hasher.update(payload.passwd.as_bytes());
    hasher.update(payload.salt.as_bytes());
    let hash = hex::encode(hasher.finalize());

    Json(serde_json::json!({
        "hash": hash
    }))
}

#[derive(Deserialize)]
pub struct LoginReq {
    pub username: String,
    pub passwd: String,
}

pub async fn login_handler(
    session: tower_sessions::Session,
    State(app_state): State<AppState>,
    Json(payload): Json<LoginReq>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match app_state
        .user_service
        .login(&payload.username, &payload.passwd)
        .await
    {
        Ok(user) => {
            if let Err(e) = session.insert("user", user).await {
                Ok(Json(serde_json::json!({
                    "code": 5000,
                    "msg": format!("Session error: {}", e)
                })))
            } else {
                Ok(Json(serde_json::json!({
                    "code": 0,
                    "msg": "login success"
                })))
            }
        }

        Err(e) => Ok(Json(serde_json::json!({
            "code": 4001,
            "msg": e
        }))),
    }
}

pub async fn get_user_by_id_handler(
    State(app_state): State<AppState>,
    Path(id): Path<u32>,
) -> Result<Json<serde_json::Value>, ServiceError> {
    let user = app_state.user_service.find_user_by_id(id).await?;

    Ok(Json(serde_json::json!({
        "code": 0,
        "msg": "success",
        "data": user
    })))
}

pub async fn logout_handler(
    session: tower_sessions::Session,
) -> Result<Json<serde_json::Value>, StatusCode> {
    session.delete().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({
        "code": 0,
        "msg": "logout success"
    })))
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::models;
    use crate::service::orders::OrderService;
    use crate::service::users::UserService;
    use crate::service::products::ProductService;
    use crate::service::inventory::InventoryService;
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::Arc;

    struct MockService;

    impl UserService for MockService {
        fn login<'a>(
            &'a self,
            _username: &'a str,
            _password: &'a str,
        ) -> Pin<Box<dyn Future<Output = Result<models::User, String>> + Send + 'a>> {
            Box::pin(async { Err("not implemented".into()) })
        }

        fn find_user_by_id<'a>(
            &'a self,
            id: u32,
        ) -> Pin<Box<dyn Future<Output = Result<models::User, ServiceError>> + Send + 'a>> {
            Box::pin(async move {
                Ok(models::User {
                    id,
                    username: "u".into(),
                    passwd: "p".into(),
                    salt: "s".into(),
                    created_at: None,
                    updated_at: None,
                })
            })
        }
    }

    struct MockOrderService;
    impl OrderService for MockOrderService {
        fn create_order_with_items<'a>(
            &'a self,
            _args: crate::service::orders::CreateOrderWithItemsArgs,
        ) -> Pin<Box<dyn Future<Output = Result<String, ServiceError>> + Send + 'a>> {
            Box::pin(async { Ok("ORDERID".into()) })
        }
        fn delete_order<'a>(
            &'a self,
            _order_id: &'a str,
        ) -> Pin<Box<dyn Future<Output = Result<(), ServiceError>> + Send + 'a>> {
            Box::pin(async { Ok(()) })
        }
        fn update_order<'a>(
            &'a self,
            _order_id: &'a str,
            _pay_amount: Option<f64>,
            _order_status: Option<i8>,
            _consignee_info: Option<serde_json::Value>,
        ) -> Pin<Box<dyn Future<Output = Result<(), ServiceError>> + Send + 'a>> {
            Box::pin(async { Ok(()) })
        }
        fn get_order<'a>(
            &'a self,
            _order_id: &'a str,
        ) -> Pin<Box<dyn Future<Output = Result<models::Order, ServiceError>> + Send + 'a>>
        {
            Box::pin(async { Err(ServiceError::NotFound("not found".into())) })
        }
        fn get_order_items<'a>(
            &'a self,
            _order_id: &'a str,
        ) -> Pin<Box<dyn Future<Output = Result<Vec<models::OrderItem>, ServiceError>> + Send + 'a>>
        {
            Box::pin(async { Ok(vec![]) })
        }
        fn cancel_order<'a>(
            &'a self,
            _order_id: &'a str,
        ) -> Pin<Box<dyn Future<Output = Result<(), ServiceError>> + Send + 'a>> {
            Box::pin(async { Ok(()) })
        }
    }

    struct MockProductService;
    impl ProductService for MockProductService {
        fn create_product<'a>(
            &'a self,
            _params: crate::domain::products::CreateProductParams,
        ) -> Pin<Box<dyn Future<Output = Result<i32, ServiceError>> + Send + 'a>> {
            Box::pin(async { Ok(1) })
        }

        fn update_product<'a>(
            &'a self,
            _product_id: i32,
            _params: crate::domain::products::UpdateProductParams,
        ) -> Pin<Box<dyn Future<Output = Result<(), ServiceError>> + Send + 'a>> {
            Box::pin(async { Ok(()) })
        }

        fn get_product<'a>(
            &'a self,
            _product_id: i32,
        ) -> Pin<Box<dyn Future<Output = Result<models::Product, ServiceError>> + Send + 'a>> {
            Box::pin(async { Err(ServiceError::NotFound("not found".into())) })
        }

        fn list_products<'a>(
            &'a self,
            _page: u32,
            _page_size: u32,
        ) -> Pin<Box<dyn Future<Output = Result<(Vec<models::Product>, u64), ServiceError>> + Send + 'a>> {
            Box::pin(async { Ok((vec![], 0)) })
        }

        fn upload_image<'a>(
            &'a self,
            _file_name: String,
            _file_data: Vec<u8>,
            _content_type: String,
        ) -> Pin<Box<dyn Future<Output = Result<String, ServiceError>> + Send + 'a>> {
            Box::pin(async { Ok("img.jpg".into()) })
        }
    }

    struct MockInventoryService;
    impl InventoryService for MockInventoryService {
        fn list_inventory<'a>(
            &'a self,
            _page: u32,
            _page_size: u32,
        ) -> Pin<Box<dyn Future<Output = Result<(Vec<models::Inventory>, u64), ServiceError>> + Send + 'a>> {
            Box::pin(async { Ok((vec![], 0)) })
        }

        fn update_inventory<'a>(
            &'a self,
            _inv_id: i32,
            _params: crate::domain::inventory::UpdateInventoryParams,
        ) -> Pin<Box<dyn Future<Output = Result<(), ServiceError>> + Send + 'a>> {
            Box::pin(async { Ok(()) })
        }

        fn get_inventory<'a>(
            &'a self,
            _inv_id: i32,
        ) -> Pin<Box<dyn Future<Output = Result<models::Inventory, ServiceError>> + Send + 'a>> {
            Box::pin(async { Err(ServiceError::NotFound("not found".into())) })
        }
    }

    #[tokio::test]
    async fn test_get_user_by_id_handler() {
        let app_state = AppState {
            user_service: Arc::new(MockService),
            order_service: Arc::new(MockOrderService),
            product_service: Arc::new(MockProductService),
            inventory_service: Arc::new(MockInventoryService),
        };
        let resp = get_user_by_id_handler(State(app_state), Path(1))
            .await
            .unwrap();
        let v = resp.0;
        assert_eq!(v["code"], 0);
        assert_eq!(v["data"]["id"], 1);
    }

    #[tokio::test]
    async fn test_hash_handler() {
        let payload = HashReq {
            passwd: "a".into(),
            salt: "b".into(),
        };
        let resp = hash_handler(Json(payload)).await;
        let v = resp.0;
        assert!(v["hash"].is_string());
    }
}
