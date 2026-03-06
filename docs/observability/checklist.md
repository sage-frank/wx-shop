# 检查清单（提交前后逐项确认）

## 功能正确性
- [ ] 编译通过（`cargo check`）
- [ ] 所有对外接口在 INFO 级别无额外噪音，仅输出业务日志
- [ ] DEBUG 级别可看到 Controller→Service→Repository 的嵌套 span
- [ ] DEBUG 级别可看到 sqlx 的每条 SQL 耗时

## 性能与安全
- [ ] INFO 与 DEBUG 的性能差异在可接受范围（如 QPS 下降 <5%）
- [ ] 使用 `skip(...)` 避免大对象序列化
- [ ] 无敏感信息（PII/密钥）写入日志；需要时已脱敏
- [ ] 大请求体日志有长度限制或不记录

## 规范一致性
- [ ] span 字段符合约定（如 user_id、order_id、items_len）
- [ ] 命名与模块路径一致，便于检索与聚合
- [ ] 日志初始化与 Settings.toml 配置保持一致（默认 info，可切 debug）

## 回滚与开关
- [ ] Settings 切回 info 后不再输出链路细节
- [ ] 注解/日志为增量、可随时移除，业务逻辑未被改动

