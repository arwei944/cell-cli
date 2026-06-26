# 交接包: cell-architecture

生成时间: 2026-06-25 12:06:22 UTC

生成者: Cell Agent

## 1. 项目概览

- **架构风格**: Cell Architecture (Hexagonal + DDD)
- **技术栈**: Rust

### 关键目录

- `src/domain`: 领域内核：核心业务模型、实体、值对象、领域服务 (12 files)
- `src/application`: 应用层：用例编排、Port接口、应用服务 (34 files)
- `src/adapters`: 适配器层：技术实现、外部系统对接 (9 files)
- `src/interfaces`: 接口层：CLI/API/UI等外部接入点 (11 files)

## 2. 当前任务

- **名称**: 仪表盘与决策面板开发
- **状态**: InProgress
- **描述**: 实现 Web 仪表盘、决策记录工具和自进化系统

## 3. 进度时间线

- [22:41:16] **Start**: Task started
- [22:41:16] **Update**: 完成决策记录领域模型和服务
- [22:41:16] **Update**: 完成观测层服务和 Web 仪表盘后端
- [22:41:16] **Update**: 完成仪表盘前端页面开发

## 4. 架构快照

- **违规数量**: 0
- **Domain 层外部依赖**: 0

- **domain**: 12 files, 0 internal deps, 0 external deps
- **application**: 34 files, 0 internal deps, 0 external deps
- **adapters**: 9 files, 0 internal deps, 0 external deps
- **interfaces**: 11 files, 0 internal deps, 0 external deps

## 5. 熵值快照

- **总分**: 36.24 (阈值: 60.00)
- **文件数**: 73
- **总行数**: 13853

## 6. 相关文件

- `src\adapters\ast_analyzer.rs` [适配器]:  (233 LOC)
- `src\adapters\dashboard.html` [适配器]:  (489 LOC)
- `src\adapters\file_decision_store.rs` [适配器]:  (149 LOC)
- `src\adapters\file_evolution_store.rs` [适配器]:  (146 LOC)
- `src\adapters\file_handoff_exporter.rs` [适配器]:  (95 LOC)
- `src\adapters\file_progress_store.rs` [适配器]:  (145 LOC)
- `src\adapters\mod.rs` [适配器]:  (7 LOC)
- `src\adapters\template_engine.rs` [适配器]:  (331 LOC)
- `src\adapters\web_dashboard.rs` [适配器]:  (239 LOC)
- `src\application\architecture_tests.rs` [测试]:  (128 LOC)
- `src\application\arch_service.rs` [应用服务]:  (437 LOC)
- `src\application\auto_progress_service.rs` [应用服务]:  (426 LOC)
- `src\application\config_service.rs` [应用服务]:  (334 LOC)
- `src\application\coverage_service.rs` [应用服务]:  (336 LOC)
- `src\application\decision_service.rs` [应用服务]:  (333 LOC)
- `src\application\dependency_analyzer.rs` [其他]:  (458 LOC)
- `src\application\entropy_service.rs` [应用服务]:  (420 LOC)
- `src\application\evolution_service.rs` [应用服务]:  (446 LOC)
- `src\application\fast_verify_service.rs` [应用服务]:  (331 LOC)
- `src\application\generate_service.rs` [应用服务]:  (287 LOC)
- `src\application\handoff_service.rs` [应用服务]:  (474 LOC)
- `src\application\impact_analysis_service.rs` [应用服务]:  (569 LOC)
- `src\application\incremental_entropy_service.rs` [应用服务]:  (445 LOC)
- `src\application\init_service.rs` [应用服务]:  (147 LOC)
- `src\application\mod.rs` [其他]:  (22 LOC)
- `src\application\observability\dashboard.rs` [其他]:  (128 LOC)
- `src\application\observability\metrics.rs` [其他]:  (121 LOC)
- `src\application\observability\mod.rs` [其他]:  (9 LOC)
- `src\application\observability\phases.rs` [其他]:  (152 LOC)
- `src\application\ports\code_generator.rs` [端口接口]:  (11 LOC)
- `src\application\ports\decision_store.rs` [端口接口]:  (10 LOC)
- `src\application\ports\evolution_store.rs` [端口接口]:  (10 LOC)
- `src\application\ports\handoff_exporter.rs` [端口接口]:  (8 LOC)
- `src\application\ports\mod.rs` [端口接口]:  (5 LOC)
- `src\application\ports\progress_store.rs` [端口接口]:  (9 LOC)
- `src\application\progress_bar.rs` [其他]:  (498 LOC)
- `src\application\progress_service.rs` [应用服务]:  (207 LOC)
- `src\application\simplicity_checker\analysis.rs` [其他]:  (185 LOC)
- `src\application\simplicity_checker\checks.rs` [其他]:  (133 LOC)
- `src\application\simplicity_checker\format.rs` [其他]:  (116 LOC)
- `src\application\simplicity_checker\issues.rs` [其他]:  (39 LOC)
- `src\application\simplicity_checker\mod.rs` [其他]:  (314 LOC)
- `src\application\simplicity_checker\types.rs` [其他]:  (123 LOC)
- `src\domain\adr.rs` [领域模型]:  (69 LOC)
- `src\domain\cell_spec.rs` [领域模型]:  (82 LOC)
- `src\domain\context.rs` [领域模型]:  (71 LOC)
- `src\domain\decision.rs` [领域模型]:  (331 LOC)
- `src\domain\entropy.rs` [领域模型]:  (892 LOC)
- `src\domain\errors.rs` [领域模型]:  (36 LOC)
- `src\domain\evolution.rs` [领域模型]:  (374 LOC)
- `src\domain\feature.rs` [领域模型]:  (78 LOC)
- `src\domain\handoff.rs` [领域模型]:  (422 LOC)
- `src\domain\mod.rs` [领域模型]:  (11 LOC)
- `src\domain\observability.rs` [领域模型]:  (164 LOC)
- `src\domain\progress.rs` [领域模型]:  (213 LOC)
- `src\interfaces\cli.rs` [接口层]:  (679 LOC)
- `src\interfaces\commands\arch_cmd.rs` [接口层]:  (44 LOC)
- `src\interfaces\commands\decision_cmd.rs` [接口层]:  (134 LOC)
- `src\interfaces\commands\dev_cmd.rs` [接口层]:  (86 LOC)
- `src\interfaces\commands\entropy_cmd.rs` [接口层]:  (60 LOC)
- `src\interfaces\commands\evolve_cmd.rs` [接口层]:  (319 LOC)
- `src\interfaces\commands\init_cmd.rs` [接口层]:  (93 LOC)
- `src\interfaces\commands\lifecycle_cmd.rs` [接口层]:  (427 LOC)
- `src\interfaces\commands\mod.rs` [接口层]:  (8 LOC)
- `src\interfaces\commands\quality_cmd.rs` [接口层]:  (156 LOC)
- `src\interfaces\mod.rs` [接口层]:  (2 LOC)
- `src\lib.rs` [其他]:  (8 LOC)
- `src\main.rs` [其他]:  (45 LOC)

## 7. 下一步行动


## 9. 决策记录 (ADR)

- **0b36c5001310402b9759157ea87d8c31** [已接受]: 选择 Rust 作为开发语言
  - 分类: 技术选型
  - 理由: Rust 提供零成本抽象、内存安全保证和优秀的工具链，适合构建架构级工具
- **13d28af2f56f4f2ea7b8fa3357b64e2b** [已接受]: 采用 Axum 作为 Web 框架
  - 分类: 技术选型
  - 理由: Axum 基于 Tokio 生态，性能优秀，设计优雅，与 Tower 生态兼容
- **19038a59f7964166a4c4e0e61a3b78be** [已接受]: 采用文件系统作为决策存储
  - 分类: 架构决策
  - 理由: 文件系统存储无需外部依赖，配置简单，便于版本控制

## 10. 环境信息

- **Rust 版本**: rustc 1.96.0 (ac68faa20 2026-05-25)
- **构建状态**: ❓ 未知
- **测试状态**: ❓ 未知

## 11. 开发规范与约束

1. 严格遵循四层架构（domain/application/adapters/interfaces），依赖只能向内
2. domain 层不能依赖任何外部框架或库（除了基础标准库）
3. 使用 Cell 架构的 Port/Adapter 模式进行外部依赖隔离
4. 所有公共 API 必须有单元测试覆盖
5. 代码提交前必须通过 cargo clippy 和 cargo fmt 检查
6. 架构约束通过编译期测试强制执行（architecture_tests）
7. 熵值阈值不超过 60，超过必须先优化再继续开发
8. 重要技术决策必须记录到决策日志（cell decision new）

## 12. 快速上手指南

- 第一步：阅读本交接包，了解项目现状和当前任务
- 第二步：运行 `cell tools status` 查看可用工具
- 第三步：运行 `cell dashboard` 打开可视化仪表盘了解全局状态
- 第四步：运行 `cargo test` 确认所有测试通过
- 第五步：查看决策记录（cell decision list）了解技术选型背景
- 第六步：从 next_actions 中选择最高优先级任务开始开发
- 开发过程中随时使用 `cell progress log` 记录进度
- 遇到阻塞使用 `cell progress block` 记录，解除后用 `cell progress unblock`

## 13. 最近修改文件

| 文件 | 修改时间 | 行数 | 类型 |
|------|----------|------|------|
| src\interfaces\commands\lifecycle_cmd.rs | 2026-06-25 12:02 | 427 | modified |
| src\application\fast_verify_service.rs | 2026-06-25 01:55 | 331 | modified |
| src\adapters\file_progress_store.rs | 2026-06-25 01:48 | 145 | modified |
| src\interfaces\cli.rs | 2026-06-25 01:43 | 679 | modified |
| src\application\simplicity_checker\mod.rs | 2026-06-25 01:30 | 314 | modified |
| Cargo.toml | 2026-06-25 01:26 | 84 | modified |
| src\application\observability\metrics.rs | 2026-06-25 01:15 | 121 | modified |
| src\application\observability\dashboard.rs | 2026-06-25 01:15 | 128 | modified |
| src\adapters\web_dashboard.rs | 2026-06-25 01:14 | 239 | modified |
| src\application\mod.rs | 2026-06-25 01:13 | 22 | modified |
| src\application\observability\mod.rs | 2026-06-25 01:11 | 9 | modified |
| src\application\observability\phases.rs | 2026-06-25 01:10 | 152 | modified |
| src\application\simplicity_checker\issues.rs | 2026-06-25 01:09 | 39 | modified |
| src\application\simplicity_checker\format.rs | 2026-06-25 01:05 | 116 | modified |
| src\application\simplicity_checker\analysis.rs | 2026-06-25 01:04 | 185 | modified |
| src\application\simplicity_checker\checks.rs | 2026-06-25 01:04 | 133 | modified |
| src\application\simplicity_checker\types.rs | 2026-06-25 01:04 | 123 | modified |
| src\adapters\dashboard.html | 2026-06-25 00:58 | 489 | modified |
| src\domain\observability.rs | 2026-06-25 00:54 | 164 | modified |
| src\interfaces\commands\dev_cmd.rs | 2026-06-25 00:49 | 86 | modified |

## 14. 交接包验证

✅ 交接包完整，可以开始接手

### 警告

- ⚠️  No next actions defined - handoff recipient may not know what to do next
