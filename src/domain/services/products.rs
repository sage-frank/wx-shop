use crate::domain::models;
use crate::domain::models::dto::params::{CreateProductParams, UpdateProductParams};
use crate::domain::services::{Future, Pin, ServiceError, ServiceResultWithLifetime};
use crate::infra::repository::traits::ProductRepo;
use std::sync::Arc;

pub struct ProductServiceImpl<R: ProductRepo + 'static> {
    repo: Arc<R>,
    s3_client: aws_sdk_s3::Client,
    s3_bucket: String,
}

impl<R: ProductRepo + 'static> ProductServiceImpl<R> {
    pub fn new(
        repo: Arc<R>,
        s3_client: aws_sdk_s3::Client,
        s3_bucket: String,
    ) -> Self {
        Self {
            repo,
            s3_client,
            s3_bucket,
        }
    }
}

pub trait ProductService: Send + Sync {
    fn create_product<'a>(
        &'a self,
        params: CreateProductParams,
    ) -> Pin<Box<dyn Future<Output = Result<i32, ServiceError>> + Send + 'a>>;

    fn update_product<'a>(
        &'a self,
        product_id: i32,
        params: UpdateProductParams,
    ) -> Pin<Box<dyn Future<Output = Result<(), ServiceError>> + Send + 'a>>;

    fn get_product<'a>(
        &'a self,
        product_id: i32,
    ) -> Pin<Box<dyn Future<Output = Result<models::Product, ServiceError>> + Send + 'a>>;

    fn list_products<'a>(
        &'a self,
        page: u32,
        page_size: u32,
        product_name: Option<String>,
    ) -> Pin<Box<dyn Future<Output = Result<(Vec<models::Product>, u64), ServiceError>> + Send + 'a>>;

    fn upload_image<'a>(
        &'a self,
        file_name: String,
        file_data: Vec<u8>,
        content_type: String,
    ) -> Pin<Box<dyn Future<Output = Result<String, ServiceError>> + Send + 'a>>;
}

impl<R: ProductRepo> ProductService for ProductServiceImpl<R> {
    fn create_product<'a>(
        &'a self,
        params: CreateProductParams,
    ) -> Pin<Box<dyn Future<Output = Result<i32, ServiceError>> + Send + 'a>> {
        Box::pin(async move {
            self.repo
                .create_product_blocking(params)
                .await
                .map_err(ServiceError::Database)
        })
    }

    fn update_product<'a>(
        &'a self,
        product_id: i32,
        params: UpdateProductParams,
    ) -> Pin<Box<dyn Future<Output = Result<(), ServiceError>> + Send + 'a>> {
        Box::pin(async move {
            self.repo
                .update_product_blocking(product_id, params)
                .await
                .map_err(ServiceError::Database)
        })
    }

    fn get_product<'a>(
        &'a self,
        product_id: i32,
    ) -> ServiceResultWithLifetime<'a, models::Product> {
        let client = self.s3_client.clone();
        let bucket = self.s3_bucket.clone();
        Box::pin(async move {
            let opt = self
                .repo
                .get_product_blocking(product_id)
                .await
                .map_err(ServiceError::Database)?;
            let mut product = opt.ok_or_else(|| ServiceError::NotFound("product not found".into()))?;
            
            if let Some(key) = &product.image_url 
                && !key.starts_with("http")
                && let Some(url) = sign_url(&client, &bucket, key).await 
            {
                product.image_url = Some(url);
            }
            Ok(product)
        })
    }

    fn list_products<'a>(
        &'a self,
        page: u32,
        page_size: u32,
        product_name: Option<String>,
    ) -> ServiceResultWithLifetime<'a, (Vec<models::Product>, u64)> {
        let client = self.s3_client.clone();
        let bucket = self.s3_bucket.clone();
        Box::pin(async move {
            let (mut products, total) = self.repo
                .list_products_blocking(page, page_size, product_name)
                .await
                .map_err(ServiceError::Database)?;
            
            for product in &mut products {
                if let Some(key) = &product.image_url 
                    && !key.starts_with("http")
                    && let Some(url) = sign_url(&client, &bucket, key).await 
                {
                    product.image_url = Some(url);
                }
            }
            Ok((products, total))
        })
    }

    fn upload_image<'a>(
        &'a self,
        file_name: String,
        file_data: Vec<u8>,
        content_type: String,
    ) -> ServiceResultWithLifetime<'a, String> {
        let client = self.s3_client.clone();
        let bucket = self.s3_bucket.clone();
        
        Box::pin(async move {
            let body = aws_sdk_s3::primitives::ByteStream::from(file_data);

            client
                .put_object()
                .bucket(&bucket)
                .key(&file_name)
                .body(body)
                .content_type(content_type)
                .send()
                .await
                .map_err(|e| ServiceError::Internal(format!("S3 upload error: {}", e)))?;

            Ok(file_name)
        })
    }
}

pub fn new_product_service(
    repo: Arc<crate::infra::repository::products::ProductRepository>,
    s3_client: aws_sdk_s3::Client,
    s3_bucket: String,
) -> Arc<dyn ProductService> {
    Arc::new(ProductServiceImpl::new(repo, s3_client, s3_bucket)) as Arc<dyn ProductService>
}

async fn sign_url(client: &aws_sdk_s3::Client, bucket: &str, key: &str) -> Option<String> {
    let presigning_config = aws_sdk_s3::presigning::PresigningConfig::expires_in(std::time::Duration::from_secs(3600)).ok()?;
    let presigned_req = client
        .get_object()
        .bucket(bucket)
        .key(key)
        .presigned(presigning_config)
        .await
        .ok()?;
    Some(presigned_req.uri().to_string())
}
