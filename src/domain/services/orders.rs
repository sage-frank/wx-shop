use crate::domain::models::dto::params::CreateOrderWithItemsParams;
use crate::infra::repository::traits::OrderRepo;
use crate::domain::models;
use crate::domain::services::{ServiceError, ServiceResultWithLifetime};
use serde_json::Value;
use std::sync::Arc;
use crate::ids;
use tracing::instrument;

pub struct OrderServiceImpl<R: OrderRepo + 'static> {
    repo: Arc<R>,
}

pub struct CreateOrderWithItemsArgs {
    pub user_id: i32,
    pub pay_amount: Option<f64>,
    pub order_status: i8,
    pub consignee_info: Option<Value>,
    pub items: Vec<(i32, String, i32, f64, Option<String>)>,
}

impl<R: OrderRepo + 'static> OrderServiceImpl<R> {
    pub fn new(repo: Arc<R>) -> Self {
        Self { repo }
    }
}

pub trait OrderService: Send + Sync {
    fn create_order_with_items<'a>(
        &'a self,
        args: CreateOrderWithItemsArgs,
    ) -> ServiceResultWithLifetime<'a, String>;

    fn delete_order<'a>(
        &'a self,
        order_id: &'a str,
    ) -> ServiceResultWithLifetime<'a, ()>;

    fn update_order<'a>(
        &'a self,
        order_id: &'a str,
        pay_amount: Option<f64>,
        order_status: Option<i8>,
        consignee_info: Option<Value>,
    ) -> ServiceResultWithLifetime<'a, ()>;

    fn get_order<'a>(
        &'a self,
        order_id: &'a str,
    ) -> ServiceResultWithLifetime<'a, models::Order>;

    fn get_order_items<'a>(
        &'a self,
        order_id: &'a str,
    ) -> ServiceResultWithLifetime<'a, Vec<models::OrderItem>>;

    fn cancel_order<'a>(
        &'a self,
        order_id: &'a str,
    ) -> ServiceResultWithLifetime<'a, ()>;
}

impl<R: OrderRepo + 'static> OrderService for OrderServiceImpl<R> {
    #[instrument(level = "debug", skip(self, args), fields(user_id = args.user_id, items_len = args.items.len()))]
    fn create_order_with_items<'a>(
        &'a self,
        args: CreateOrderWithItemsArgs,
    ) -> ServiceResultWithLifetime<'a, String> {
        Box::pin(async move {
            let CreateOrderWithItemsArgs {
                user_id,
                pay_amount,
                order_status,
                consignee_info,
                items,
            } = args;

            let total_amount: f64 = items
                .iter()
                .map(|(_, _, qty, price, _)| (*qty as f64) * (*price))
                .sum();
            
            let order_id = ids::generate_prefixed_snowflake("SNACK");

            let params = CreateOrderWithItemsParams {
                order_id: order_id.clone(),
                user_id,
                total_amount,
                pay_amount,
                order_status,
                consignee_info,
                items,
            };

            self.repo
                .create_order_with_items_blocking(params)
                .await
                .map_err(ServiceError::from)?;
            Ok(order_id)
        })
    }

    #[instrument(level = "debug", skip(self, order_id), fields(order_id = %order_id))]
    fn delete_order<'a>(
        &'a self,
        order_id: &'a str,
    ) -> ServiceResultWithLifetime<'a, ()> {
        Box::pin(async move {
            self.repo
                .delete_order_blocking(order_id)
                .await
                .map_err(ServiceError::from)
        })
    }

    #[instrument(level = "debug", skip(self, order_id, pay_amount, order_status, consignee_info), fields(order_id = %order_id))]
    fn update_order<'a>(
        &'a self,
        order_id: &'a str,
        pay_amount: Option<f64>,
        order_status: Option<i8>,
        consignee_info: Option<Value>,
    ) -> ServiceResultWithLifetime<'a, ()> {
        Box::pin(async move {
            self.repo
                .update_order_blocking(order_id, pay_amount, order_status, consignee_info)
                .await
                .map_err(ServiceError::from)
        })
    }

    #[instrument(level = "debug", skip(self, order_id), fields(order_id = %order_id))]
    fn get_order<'a>(
        &'a self,
        order_id: &'a str,
    ) -> ServiceResultWithLifetime<'a, models::Order> {
        Box::pin(async move {
            let opt = self
                .repo
                .get_order_blocking(order_id)
                .await
                .map_err(ServiceError::from)?;
            let order = opt.ok_or_else(|| ServiceError::NotFound("order not found".into()))?;
            Ok(order)
        })
    }

    #[instrument(level = "debug", skip(self, order_id), fields(order_id = %order_id))]
    fn get_order_items<'a>(
        &'a self,
        order_id: &'a str,
    ) -> ServiceResultWithLifetime<'a, Vec<models::OrderItem>>
    {
        Box::pin(async move {
            let items = self
                .repo
                .get_order_items_blocking(order_id)
                .await
                .map_err(ServiceError::from)?;
            Ok(items)
        })
    }

    #[instrument(level = "debug", skip(self, order_id), fields(order_id = %order_id))]
    fn cancel_order<'a>(
        &'a self,
        order_id: &'a str,
    ) -> ServiceResultWithLifetime<'a, ()> {
        Box::pin(async move {
            self.repo
                .cancel_order_blocking(order_id)
                .await
                .map_err(ServiceError::from)
        })
    }
}

pub fn new_order_service(
    repo: Arc<crate::infra::repository::orders::OrderRepository>,
) -> Arc<dyn OrderService> {
    Arc::new(OrderServiceImpl::new(repo)) as Arc<dyn OrderService>
}
