use crate::domain::models::dto::params::{CreateProductParams, UpdateProductParams};
use crate::infra::repository::traits::ProductRepo;
use crate::domain::models;
use sqlx::{MySql, Pool};
use std::sync::Arc;

pub struct ProductRepository {
    pool: Pool<MySql>,
}

impl ProductRepository {
    pub fn new(pool: Pool<MySql>) -> Arc<Self> {
        Arc::new(Self { pool })
    }

    async fn create_product_internal(
        &self,
        params: CreateProductParams,
    ) -> Result<i32, sqlx::Error> {
        let result = sqlx::query(
            r#"INSERT INTO wx_products (product_name, category_id, category_name, spec_template, description, image_url, base_price)
               VALUES (?, ?, ?, ?, ?, ?, ?)"#,
        )
        .bind(params.product_name)
        .bind(params.category_id)
        .bind(params.category_name)
        .bind(params.spec_template)
        .bind(params.description)
        .bind(params.image_url)
        .bind(params.base_price)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_id() as i32)
    }

    async fn update_product_internal(
        &self,
        product_id: i32,
        params: UpdateProductParams,
    ) -> Result<(), sqlx::Error> {
        let mut qb = sqlx::QueryBuilder::new("UPDATE wx_products SET ");
        let mut separated = qb.separated(", ");
        let mut has_update = false;

        if let Some(name) = params.product_name {
            separated.push("product_name = ");
            separated.push_bind_unseparated(name);
            has_update = true;
        }
        if let Some(cat_id) = params.category_id {
            separated.push("category_id = ");
            separated.push_bind_unseparated(cat_id);
            has_update = true;
        }
        if let Some(cat_name) = params.category_name {
            separated.push("category_name = ");
            separated.push_bind_unseparated(cat_name);
            has_update = true;
        }
        if let Some(spec) = params.spec_template {
            separated.push("spec_template = ");
            separated.push_bind_unseparated(spec);
            has_update = true;
        }
        if let Some(desc) = params.description {
            separated.push("description = ");
            separated.push_bind_unseparated(desc);
            has_update = true;
        }
        if let Some(img) = params.image_url {
            separated.push("image_url = ");
            separated.push_bind_unseparated(img);
            has_update = true;
        }
        if let Some(price) = params.base_price {
            separated.push("base_price = ");
            separated.push_bind_unseparated(price);
            has_update = true;
        }
        if let Some(status) = params.status {
            separated.push("status = ");
            separated.push_bind_unseparated(status);
            has_update = true;
        }

        if !has_update {
            return Ok(());
        }

        qb.push(" WHERE product_id = ");
        qb.push_bind(product_id);

        qb.build().execute(&self.pool).await?;
        Ok(())
    }

    async fn get_product_internal(
        &self,
        product_id: i32,
    ) -> Result<Option<models::Product>, sqlx::Error> {
        sqlx::query_as::<_, models::Product>("SELECT * FROM wx_products WHERE product_id = ?")
            .bind(product_id)
            .fetch_optional(&self.pool)
            .await
    }

    async fn list_products_internal(
        &self,
        page: u32,
        page_size: u32,
        product_name: Option<String>,
    ) -> Result<(Vec<models::Product>, u64), sqlx::Error> {
        let offset = (if page > 0 { page - 1 } else { 0 }) * page_size;
        
        let mut count_qb = sqlx::QueryBuilder::new("SELECT COUNT(*) FROM wx_products WHERE 1=1");
        if let Some(ref name) = product_name {
            count_qb.push(" AND product_name LIKE ");
            count_qb.push_bind(format!("%{}%", name));
        }
        let total: (i64,) = count_qb.build_query_as().fetch_one(&self.pool).await?;

        let mut qb = sqlx::QueryBuilder::new("SELECT * FROM wx_products WHERE 1=1");
        if let Some(name) = product_name {
            qb.push(" AND product_name LIKE ");
            qb.push_bind(format!("%{}%", name));
        }
        qb.push(" ORDER BY created_at DESC LIMIT ");
        qb.push_bind(page_size);
        qb.push(" OFFSET ");
        qb.push_bind(offset);

        let products = qb.build_query_as::<models::Product>().fetch_all(&self.pool).await?;
        
        Ok((products, total.0 as u64))
    }
}

impl ProductRepo for ProductRepository {
    fn clone_box(&self) -> Box<dyn ProductRepo> {
        Box::new(ProductRepository {
            pool: self.pool.clone(),
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn create_product_blocking<'a>(
        &'a self,
        params: CreateProductParams,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<i32, sqlx::Error>> + Send + 'a>>
    {
        Box::pin(self.create_product_internal(params))
    }

    fn update_product_blocking<'a>(
        &'a self,
        product_id: i32,
        params: UpdateProductParams,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), sqlx::Error>> + Send + 'a>>
    {
        Box::pin(self.update_product_internal(product_id, params))
    }

    fn get_product_blocking<'a>(
        &'a self,
        product_id: i32,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<Option<models::Product>, sqlx::Error>>
                + Send
                + 'a,
        >,
    > {
        Box::pin(self.get_product_internal(product_id))
    }

    fn list_products_blocking<'a>(
        &'a self,
        page: u32,
        page_size: u32,
        product_name: Option<String>,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<(Vec<models::Product>, u64), sqlx::Error>>
                + Send
                + 'a,
        >,
    > {
        Box::pin(self.list_products_internal(page, page_size, product_name))
    }
}
