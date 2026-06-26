# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.8.0] - 2026-06-26

### Added

#### Sprint1: 核心架构与 CLI 基础设施
- 全新四层架构：Domain / Application / Adapters / Interfaces
- 45+ CLI 命令，覆盖架构、熵值、测试、部署、审计等核心场景
- 问题指纹库 + RCA v2 自动根因分析
- 模式库 + 架构建议引擎
- 重构助手 + 规则引擎 + 契约测试
- 服务网格（Istio 配置生成/验证/diff）
- 插件系统 + 沙箱 + 验证器
- Canary 发布 + A/B 实验 + 回滚
- Feature Unit 生命周期管理
- 交接包（Handoff）+ 进度追踪
- 多 Agent 协作协议 + 自愈系统
- 多环境配置 + 数据库迁移
- 强制门禁 + 工具策略 + 审计日志
- 模板引擎 + 代码生成

#### Sprint2: PHASE1 收尾 + PHASE2 命令补全
- `entropy.yaml` 配置文件支持（五维权重 + 阈值 + 忽略规则）
- 自动埋点能力（Trace/Metrics/Log 自动注入）
- 原子性挂载/卸载（4 阶段 + 回滚）
- `cell.yaml` Schema 验证
- Feature Unit 完整生命周期（create/mount/unmount/impact/list）
- Saga CLI 命令（create/list）
- Contract CLI 命令（create/list）
- Entropy Bank CLI 命令（balance/deposit/withdraw）
- Complexity Quota CLI 命令（status/check）
- 15+ 新增 CLI 子命令

### Changed

- 项目版本从 v1.0 白皮书版调整为 v0.8.0 功能版
- 命令总数从 30+ 扩展到 45+
- 单元测试覆盖率提升至 750+

### Fixed

- 修复编译错误 33 个（Sprint1）
- 修复类型不匹配、缺失导入、格式符错误等
- 修复 `.gitignore` 遗漏导致的推送失败问题

### Security

- 插件沙箱安全隔离
- 零信任工具策略管理
- Git Hooks 强制门禁

---

## [0.7.0] - 2026-06-25

### Added

- 白皮书 15 章完整版
- 架构全景图（ASCII + Mermaid）
- 智能体开发工作流（6 角色 + 7 步循环）
- 低熵监控体系（五维熵值模型）
- 事件驱动 + Saga + 插件系统
- 可观测性体系（OpenTelemetry）
- 运维与治理（K8s + SRE + 混沌工程）
- 全维度版本管理（12 类对象 + 四层迭代）

---

## [0.1.0] - 2026-06-20

### Added

- 项目初始化
- 核心概念定义（Cell / Port / Adapter / Domain）
- 基础架构文档
