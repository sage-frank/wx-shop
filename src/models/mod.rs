use sqlx::FromRow; // 需要引入 FromRow Trait

// 允许在其他文件通过 `crate::models::User` 或 `use crate::models::User` 引用
#[derive(FromRow, Debug, Clone)] // 增加 Debug 和 Clone 可能会很有用
pub struct User {
    // 您可以根据需要决定是否包含 id
    // pub id: i32,
    pub name: String,
    pub email: Option<String>,
}