# wx-shop

wx-shop 是一个高性能、基于 Rust 编写的电商后端服务，旨在提供安全、可靠且易于扩展的 API 接口。项目采用典型的分层架构（Handlers-Services-Repository），结合 **Axum** Web 框架、**SQLx** 异步数据库操作以及 **Redis** 会话管理，适用于构建高并发的电商应用。

## 🚀 核心特性

- **用户系统**: 支持用户注册、登录（加盐哈希）、退出登录，基于 Redis 的分布式 Session 管理。
- **商品管理**:
  - 商品 CRUD 操作（列表分页、添加、编辑、下架）。
  - **图片存储**: 集成 AWS S3 / MinIO 对象存储，支持图片上传及访问链接生成（Presigned URL）。
- **库存管理**:
  - 实时库存查询。
  - **乐观锁机制**: 防止高并发场景下的超卖问题。
  - 低库存预警阈值支持。
- **订单系统**:
  - 订单创建、查询、更新、删除。
  - **取消订单**: 支持原子性操作，取消订单自动释放冻结库存。
  - 事务支持：确保订单状态流转与库存扣减的数据一致性。
- **架构设计**:
  - 清晰的分层架构：Handlers (接口层) -> Services (业务逻辑) -> Repository (数据访问) -> Models (领域模型)。
  - **整合设计**: 移除了冗余的 domain 层，将业务参数整合至 `models/dto`，将接口契约整合至 `repository/traits`。
  - 统一错误处理与结构化日志 (Tracing)。

## 🛠️ 技术栈

- **编程语言**: [Rust](https://www.rust-lang.org/) (2024 Edition)
- **Web 框架**: [Axum](https://github.com/tokio-rs/axum)
- **数据库 ORM**: [SQLx](https://github.com/launchbadge/sqlx) (MySQL)
- **缓存/会话**: [Redis](https://redis.io/) (via `tower-sessions`, `fred`)
- **对象存储**: [AWS SDK for Rust](https://github.com/awslabs/aws-sdk-rust) (S3 / MinIO)
- **配置管理**: [Config](https://github.com/mehcode/config-rs)
- **日志追踪**: [Tracing](https://github.com/tokio-rs/tracing) + Tracing Appender
- **序列化**: [Serde](https://serde.rs/) (JSON)

## 📂 项目结构

```
src/
├── handlers/    # 接口层 - 处理 HTTP 请求/响应，参数校验 (控制器)
├── services/    # 服务层 - 核心业务逻辑，事务控制
├── repository/  # 仓储层 - 数据库/中间件交互实现，含 traits 定义
├── models/      # 模型层 - 包含 entities (数据库映射) 与 dto (传输对象)
│   ├── dto/     # 输入输出模型 (Request/Response params)
│   └── mod.rs   # 数据库实体映射定义
├── routes/      # 路由层 - 路由注册与中间件配置
├── config.rs    # 配置入口 - 环境/配置文件读取
├── error.rs     # 错误定义 - 全局 AppError 类型
├── lib.rs       # 库入口 - 导出 Settings 与常用工具
└── main.rs      # 应用入口 - 初始化配置、数据库并启动服务
```

## ⚙️ 配置说明

项目根目录下需要 `Settings.toml` 文件。

**Settings.toml 示例**:

```toml
[server]
host = "0.0.0.0"
port = 3000

[database]
# MySQL 连接地址
database_url = "mysql://user:password@localhost:3306/wx_shop"
max_connections = 10

[redis]
# Redis 连接地址
url = "redis://localhost:6379"
pool_size = 10

[log]
dir = "./logs"
file = "app.log"
level = "info"
max_history = 3

[s3]
# 对象存储配置 (兼容 AWS S3 或 MinIO)
endpoint = "http://127.0.0.1:9000"
bucket = "wx-shop-assets"
access_key = "your-access-key"
secret_key = "your-secret-key"
region = "us-east-1"
```

## 🚀 快速开始

### 前置要求

- Rust (Latest Stable)
- MySQL 8.0+
- Redis 6.0+
- MinIO (或其他 S3 兼容存储)

### 数据库初始化

请执行提供的 SQL 脚本以初始化数据库表结构（`wx_users`, `wx_products`, `wx_orders`, `wx_inventory` 等）。

### 运行项目

1.  **克隆仓库**:
    ```bash
    git clone https://github.com/your-username/wx-shop.git
    cd wx-shop
    ```

2.  **配置文件**:
    复制并修改配置文件（如上所示）。

3.  **运行**:
    ```bash
    cargo run
    ```

服务启动后默认监听: `http://0.0.0.0:3000`

## 🔌 API 接口概览

### 用户 (User)
- `POST /login` - 用户登录
- `POST /logout` - 退出登录
- `GET /user/{id}` - 获取用户信息

### 商品 (Product)
- `GET /products` - 获取商品列表 (分页)
- `POST /products` - 创建商品
- `POST /products/upload` - 上传商品图片
- `PUT /products/{id}` - 更新商品信息
- `POST /products/{id}/off-shelf` - 商品下架

### 库存 (Inventory)
- `GET /inventory` - 获取库存列表 (分页)
- `PUT /inventory/{id}` - 更新库存 (需携带 version 版本号)

### 订单 (Order)
- `POST /orders` - 创建订单
- `GET /orders/{id}` - 获取订单详情
- `PUT /orders/{id}` - 更新订单
- `POST /orders/{id}/cancel` - 取消订单
- `DELETE /orders/{id}` - 删除订单
- `GET /orders/{id}/items` - 获取订单项

## 📝 开发规范

- **代码风格**: 遵循 Rust 标准格式 (`cargo fmt`)。
- **提交规范**: 使用 Conventional Commits (e.g., `feat:`, `fix:`, `docs:`).
- **分层原则**: Handlers 仅处理 HTTP 协议转换；Services 处理业务；Repository 仅处理数据存取。
