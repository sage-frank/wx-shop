# 可观测性与链路追踪规范（instrument 方案）

## 背景与目标
- 建立从 Repository → Service → Controller 的全链路可观测能力，便于精准定位慢点与异常。
- 在不影响生产性能的前提下，按需开启/关闭详细追踪。
- 在 Controller 层输出结构化业务日志（如 user_id、order_id），保证业务可审计、可排障。

## 范围
- API 层：`src/api/handlers/**` 与 `src/api/middleware.rs`
- Service 层：`src/domain/services/**`
- Repository 层：`src/infra/repository/**`
- 横切支持：日志初始化（`src/main.rs`）、配置（Settings.toml）
- 后续可扩展到定时任务/异步任务（若有）

## 设计原则
- 性能优先：默认仅输出必要的 info 级业务日志；debug 级链路仅在排障时开启。
- 最小侵入：采用 `#[instrument]` 宏，使用 `skip(...)` 避免大对象序列化。
- 统一规范：统一 span 命名与字段，便于检索聚合。
- 安全合规：不记录敏感信息；大请求体限流、脱敏。
- 可控开关：通过 Settings.toml 或 RUST_LOG 定位级别；无需改代码即可增减详情。

## 技术方案
### 1. 基于 tracing 的分层追踪
- Controller 层（handlers）：
  - 为每个对外接口函数添加 `#[instrument(level = "debug", skip(state/payload等), fields(...))]`
  - 入口/成功/失败分别输出一条 info 级业务日志（含关键字段），供生产常驻
- Service 层：
  - 为关键业务方法添加 `#[instrument(level = "debug", skip(self, args), fields(...))]`
  - 记录如 `user_id`、`items_len`、领域对象主键等
- Repository 层：
  - 为关键事务/重 I/O 方法添加 `#[instrument(level = "debug", skip(self, params), fields(...))]`
  - 依赖 `sqlx` 的 debug 日志展示每条 SQL 的耗时

### 2. 请求级 Span 与上下文传播
- 保持现有基于 `TraceLayer` 的请求级 Span（method、uri、session_id）
- 在 handlers/service/repository 的 instrument span 将自动挂载到请求级 Span 下，形成嵌套链路
- 异步任务（tokio::spawn）使用 `tracing::Instrument` 的 `in_current_span()` 进行上下文传播（如后续引入）

### 3. 日志级别与输出策略
- Settings.toml: `[log].level` 默认 `info`
  - info：仅业务日志（Controller 层）与基础请求日志
  - debug：开启 instrument span 与 sqlx 详细耗时
- 保持滚动日志与当日日志软链接；不更改现有日志目录与文件结构

### 4. 字段规范（建议）
- 通用：`request_id`（如引入）、`session_id`、`user_id`
- 订单相关：`order_id`、`items_len`、`pay_amount`
- 库存相关：`sku_id`、`quantity`
- 数据库相关：由 `sqlx` 输出 `summary`、`elapsed`，无需额外字段

### 5. 命名规范
- Span 名称即函数名；模块路径由编译器/日志器自动提供
- 业务 info 日志内容简洁、可读，字段放在结构化键值中

### 6. 性能影响与控制
- `#[instrument]` 在过滤级别高于 span 级别时几乎零开销（不会进入日志路径）
- 在宏上使用 `skip(...)` 避免大对象格式化
- 仅在关键路径添加 instrument，避免全量覆盖
- 通过 Settings/RUST_LOG 切换 info/debug 而无需重启（取决于部署）

## 验收标准
- 在 `info` 级别：可看到 Controller 输出的业务日志，无明显额外开销
- 在 `debug` 级别：可看到 Controller→Service→Repository 的嵌套 span 与每条 SQL 耗时
- 压测下 `info` 与 `debug` 的性能差异在可接受范围（例如 QPS 下降 <5%）
- 无敏感信息泄露，长请求体/日志体被限制或脱敏

## 回滚与禁用策略
- 紧急情况下将 Settings 的日志级别设回 `info` 即可关闭大部分追踪细节
- 所有改动仅限注解与少量日志，不改变业务逻辑，删除注解即可完全回滚

