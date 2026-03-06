# 实施任务清单（按层推进）

## 1. Controller 层（handlers）
- 为每个对外接口 handler 添加 `#[instrument(level = "debug", skip(state/payload/...))]`
- 在入口/成功/失败各输出一条 info 级业务日志（含 user_id/主要业务主键）
- 限制/脱敏可能的敏感字段（如手机号，仅尾号）

## 2. Service 层
- 为关键业务方法添加 `#[instrument(level = "debug", skip(self, args), fields(...))]`
- 字段：常用 `user_id`、对象主键、批量规模（如 items_len）
- 对跨协程的异步流程（若存在）使用 `in_current_span()` 进行上下文传播

## 3. Repository 层
- 为重 I/O/事务方法添加 `#[instrument(level = "debug", skip(self, params), fields(...))]`
- 保持 `sqlx` 在 debug 下输出每条 SQL 的 `summary` 与 `elapsed`
- 仅关键路径加注解，避免全量覆盖

## 4. 请求级 Span 与中间件
- 保持 TraceLayer 的请求级 Span 字段（method/uri/session_id）
- 如需 request_id：在 middleware 生成 `X-Request-Id` 并加入 Span 字段

## 5. 日志初始化与配置
- 保持滚动日志与软链接
- Settings.toml `[log].level` 默认 `info`；排障时设为 `debug`
- 使用 `tracing_subscriber::fmt().with_max_level(log_level)`，不强制输出 span 事件

## 6. 文档与示例
- 在 `docs/observability` 增加使用指南与示例日志片段
- 说明如何通过 RUST_LOG/Settings 快速切换级别

