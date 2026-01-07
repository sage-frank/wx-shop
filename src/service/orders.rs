use crate::domain::orders::OrderRepo;
use crate::models;
use crate::service::ServiceError;
use serde_json::Value;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub struct OrderServiceImpl<R: OrderRepo + 'static> {
    repo: Arc<R>,
}

impl<R: OrderRepo + 'static> OrderServiceImpl<R> {
    pub fn new(repo: Arc<R>) -> Self {
        Self { repo }
    }
}

pub trait OrderService: Send + Sync {
    fn create_order_with_items<'a>(
        &'a self,
        user_id: i32,
        pay_amount: Option<f64>,
        order_status: i8,
        consignee_info: Option<Value>,
        items: Vec<(i32, String, i32, f64, Option<String>)>,
    ) -> Pin<Box<dyn Future<Output = Result<String, ServiceError>> + Send + 'a>>;

    fn delete_order<'a>(&'a self, order_id: &'a str) -> Pin<Box<dyn Future<Output = Result<(), ServiceError>> + Send + 'a>>;

    fn update_order<'a>(
        &'a self,
        order_id: &'a str,
        pay_amount: Option<f64>,
        order_status: Option<i8>,
        consignee_info: Option<Value>,
    ) -> Pin<Box<dyn Future<Output = Result<(), ServiceError>> + Send + 'a>>;

    fn get_order<'a>(&'a self, order_id: &'a str) -> Pin<Box<dyn Future<Output = Result<models::Order, ServiceError>> + Send + 'a>>;

    fn get_order_items<'a>(&'a self, order_id: &'a str) -> Pin<Box<dyn Future<Output = Result<Vec<models::OrderItem>, ServiceError>> + Send + 'a>>;
}


impl<R: OrderRepo + 'static> OrderService for OrderServiceImpl<R> {
    fn create_order_with_items<'a>(
        &'a self,
        user_id: i32,
        pay_amount: Option<f64>,
        order_status: i8,
        consignee_info: Option<Value>,
        items: Vec<(i32, String, i32, f64, Option<String>)>,
    ) -> Pin<Box<dyn Future<Output = Result<String, ServiceError>> + Send + 'a>> {
        Box::pin(async move {
            let total_amount: f64 = items.iter().map(|(_, _, qty, price, _)| (*qty as f64) * (*price)).sum();
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .subsec_nanos();

            let suffix: u32 = (nanos % 10000) as u32;
            
            let order_id = format!("{}{}", chrono::Local::now().format("%Y%m%d%H%M%S"), suffix);
            
            self.repo
                .create_order_with_items_blocking(&order_id, user_id, total_amount, pay_amount, order_status, consignee_info, items)
                .await
                .map_err(ServiceError::from)?;
            Ok(order_id)
        })
    }

    fn delete_order<'a>(&'a self, order_id: &'a str) -> Pin<Box<dyn Future<Output = Result<(), ServiceError>> + Send + 'a>> {
        Box::pin(async move {
            self.repo.delete_order_blocking(order_id).await.map_err(ServiceError::from)
        })
    }

    fn update_order<'a>(
        &'a self,
        order_id: &'a str,
        pay_amount: Option<f64>,
        order_status: Option<i8>,
        consignee_info: Option<Value>,
    ) -> Pin<Box<dyn Future<Output = Result<(), ServiceError>> + Send + 'a>> {
        Box::pin(async move {
            self.repo
                .update_order_blocking(order_id, pay_amount, order_status, consignee_info)
                .await
                .map_err(ServiceError::from)
        })
    }

    fn get_order<'a>(&'a self, order_id: &'a str) -> Pin<Box<dyn Future<Output = Result<models::Order, ServiceError>> + Send + 'a>> {
        Box::pin(async move {
            let opt = self.repo.get_order_blocking(order_id).await.map_err(ServiceError::from)?;
            let order = opt.ok_or_else(|| ServiceError::NotFound("order not found".into()))?;
            Ok(order)
        })
    }

    fn get_order_items<'a>(&'a self, order_id: &'a str) -> Pin<Box<dyn Future<Output = Result<Vec<models::OrderItem>, ServiceError>> + Send + 'a>> {
        Box::pin(async move {
            let items = self.repo.get_order_items_blocking(order_id).await.map_err(ServiceError::from)?;
            Ok(items)
        })
    }
}


pub fn new_order_service(repo: Arc<crate::repos::orders::OrderRepository>) -> Arc<dyn OrderService> {
    Arc::new(OrderServiceImpl::new(repo)) as Arc<dyn OrderService>
}
