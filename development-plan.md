# Cell 架构开发计划 v2.0

> 文档版本：v2.0
> 更新日期：2026-06-25
> 状态：正式版
> 依据：[Cell 架构白皮书 v1.0](cell-architecture-whitepaper.md)
> 技术栈：Rust 2024 + Clap + Syn + Serde

---

## 一、文档说明

### 1.1 项目定位

**Cell CLI** 是 Cell 架构的官方工具链，面向 AI 智能体原生开发的低熵架构工具。

```
项目定位：架构即提示的执行载体
  ├── 代码生成器（从 Spec 生成代码骨架）
  ├── 静态分析引擎（架构规则 + 熵值计算）
  ├── 架构验证器（分层依赖 + 命名规范 + 不变量）
  └── 智能体工作流支持（ADR + 上下文 + 交接）
```

### 1.2 最小任务单元定义标准

一个最小任务单元满足：

1. **产出明确**：一个或多个具体文件/函数
2. **可独立测试**：有对应的单元测试
3. **不依赖未完成任务**（或依赖已标记完成的任务）
4. **1-3 天内可完成**
5. **验收标准可量化**：通过/失败有明确判定

### 1.3 任务编号规范

```
PHASE<N>-TASK-XXX
  │       │
  │       └── 三位流水号
  └────────── 阶段号 (0-4)

示例：PHASE0-TASK-001 = Phase 0 第 1 个任务
```

### 1.4 验收标准五级体系

| 等级 | 标识 | 内容 |
|------|------|------|
| L1 | ✅ 编译通过 | cargo build --release 无错误 |
| L2 | ✅ 测试通过 | cargo test 全部通过 |
| L3 | ✅ Lint 通过 | cargo clippy -- -D warnings |
| L4 | ✅ 架构合规 | cargo test -- architecture_tests + cell arch validate |
| L5 | ✅ 熵值达标 | cell entropy guard 低于阈值 |

---

## 二、当前状态与进度基线

### 2.1 已完成（Phase 0 部分完成）

| 任务 | 状态 | 产出 |
|------|------|------|
| CLI 框架搭建 | ✅ 完成 | clap + 子命令体系 |
| cell init 命令 | ✅ 完成 | init_service.rs |
| 基础 AST 解析 | ✅ 完成 | ast_analyzer.rs |
| 熵值计算原型 | ✅ 完成 | 结构熵 + 复杂度熵（二维） |
| 架构验证原型 | ✅ 完成 | arch_service.rs + 8 个架构测试 |
| 代码生成原型 | ✅ 完成 | template_engine + generate_service |
| Micro-ADR 数据结构 | ✅ 完成 | adr.rs |
| DevContext 数据结构 | ✅ 完成 | context.rs |
| Feature Unit 数据结构 | ✅ 完成 | feature.rs |
| CI/CD 架构门禁 | ✅ 完成 | ci.yml + architecture-gate |
| pre-commit hooks | ✅ 完成 | .githooks/ |
| Clippy 硬约束 | ✅ 完成 | Cargo.toml [lints] |
| 编译期架构测试 | ✅ 完成 | architecture_tests.rs |

### 2.2 待完成

- Phase 0 剩余任务
- Phase 1 全部任务
- Phase 2-4 远期规划

---

## 三、Phase 0：基础建设（共 15 个任务）

> **目标**：搭建完整的工具链基础，实现核心原型能力
> **预计周期**：2 周
> **里程碑**：工具链可用，能生成、能检查、能度量

---

### PHASE0-TASK-001：CLI 框架搭建

- **状态**：✅ 已完成
- **产出**：`src/interfaces/cli.rs`, `src/main.rs`
- **验收标准**：
  - [x] `cell --help` 正常输出
  - [x] 所有子命令定义完整
  - [x] 全局参数（--verbose, --format）可用
  - [x] shell completions 生成正常

---

### PHASE0-TASK-002：cell init 命令

- **状态**：✅ 已完成
- **产出**：`src/application/init_service.rs`
- **验收标准**：
  - [x] 生成标准四层目录结构
  - [x] 生成 Cargo.toml + README.md
  - [x] --force 参数可覆盖
  - [x] 单元测试覆盖率 ≥ 80%

---

### PHASE0-TASK-003：领域层基础类型

- **状态**：✅ 已完成
- **产出**：`src/domain/errors.rs`, `src/domain/cell_spec.rs`
- **验收标准**：
  - [x] CellResult 统一错误类型
  - [x] CellSpec / PortSpec / AdapterSpec 数据模型
  - [x] PortKind / AdapterKind 枚举完整
  - [x] 支持 JSON/YAML/TOML 序列化

---

### PHASE0-TASK-004：Rust AST 解析器

- **状态**：✅ 已完成
- **产出**：`src/adapters/ast_analyzer.rs`
- **验收标准**：
  - [x] 解析函数定义、结构体、枚举、Trait
  - [x] 解析 use 语句（依赖分析）
  - [x] 计算圈复杂度、嵌套深度
  - [x] 支持单文件和目录递归分析

---

### PHASE0-TASK-005：熵值计算原型（二维）

- **状态**：✅ 已完成
- **产出**：`src/domain/entropy.rs`, `src/application/entropy_service.rs`
- **验收标准**：
  - [x] 结构熵计算（文件大小分布、模块均匀度）
  - [x] 复杂度熵计算（圈复杂度、嵌套深度）
  - [x] EntropyReport 数据模型完整
  - [x] 支持多语言（rs/go/ts/js）

---

### PHASE0-TASK-006：架构分层验证

- **状态**：✅ 已完成
- **产出**：`src/application/arch_service.rs`, `src/application/architecture_tests.rs`
- **验收标准**：
  - [x] 四层分层识别（domain/application/adapters/interfaces）
  - [x] 依赖方向检查（只能向内依赖）
  - [x] 6 条核心铁律验证通过
  - [x] 8 个编译期架构单元测试

---

### PHASE0-TASK-007：代码生成模板引擎

- **状态**：✅ 已完成
- **产出**：`src/adapters/template_engine.rs`
- **验收标准**：
  - [x] 生成 Cell 完整目录结构
  - [x] 生成 Port 接口模板
  - [x] 生成 Adapter 骨架
  - [x] 单元测试覆盖率 ≥ 80%

---

### PHASE0-TASK-008：cell generate 系列命令

- **状态**：🔄 进行中
- **产出**：`src/application/generate_service.rs`
- **依赖**：PHASE0-TASK-007
- **验收标准**：
  - [ ] `cell generate cell <name>` 生成完整 Cell 骨架
  - [ ] `cell generate port <name> --kind` 生成 Port 接口
  - [ ] `cell generate adapter <name> --kind --port` 生成 Adapter
  - [ ] --spec 参数支持从 spec.yaml 生成
  - [ ] --force 参数可覆盖
  - [x] 单元测试覆盖率 ≥ 80%

---

### PHASE0-TASK-009：Micro-ADR 数据模型

- **状态**：✅ 已完成
- **产出**：`src/domain/adr.rs`
- **验收标准**：
  - [x] MicroAdr 结构体完整
  - [x] 状态流转（Proposed → Accepted → Deprecated → Superseded）
  - [x] 支持 JSON/YAML 序列化

---

### PHASE0-TASK-010：结构化开发上下文

- **状态**：✅ 已完成
- **产出**：`src/domain/context.rs`
- **验收标准**：
  - [x] DevContext 结构体完整
  - [x] FileRef / DecisionRef 类型完整
  - [x] 支持序列化持久化

---

### PHASE0-TASK-011：Feature Unit 元模型

- **状态**：✅ 已完成
- **产出**：`src/domain/feature.rs`
- **验收标准**：
  - [x] FeatureUnit 结构体完整
  - [x] FeatureStatus 生命周期枚举
  - [x] ExtensionPoint 类型定义
  - [x] EntropyBankAccount 熵值银行模型

---

### PHASE0-TASK-012：CI/CD 架构门禁

- **状态**：✅ 已完成
- **产出**：`.github/workflows/ci.yml`
- **验收标准**：
  - [x] lint + test + build 全流程
  - [x] architecture-gate 任务（架构验证必过）
  - [x] entropy-gate 任务（熵值门禁）
  - [x] Windows/Linux/macOS 三平台构建

---

### PHASE0-TASK-013：pre-commit Git Hooks

- **状态**：✅ 已完成
- **产出**：`.githooks/pre-commit`, `.githooks/pre-commit.ps1`
- **验收标准**：
  - [x] fmt check 强制
  - [x] clippy -D warnings 强制
  - [x] cargo test 强制
  - [x] arch validate 强制
  - [x] entropy guard 强制

---

### PHASE0-TASK-014：Clippy 硬约束配置

- **状态**：✅ 已完成
- **产出**：`Cargo.toml [lints]`
- **验收标准**：
  - [x] unwrap_used = deny
  - [x] unused_imports = deny
  - [x] unused_variables = deny
  - [x] unsafe_code = warn
  - [x] 所有现有代码无 lint 错误

---

### PHASE0-TASK-015：编译期架构测试

- **状态**：✅ 已完成
- **产出**：`src/application/architecture_tests.rs`
- **验收标准**：
  - [x] 8 个架构测试全部通过
  - [x] domain 不依赖 application
  - [x] domain 不依赖 adapters
  - [x] domain 不依赖 interfaces
  - [x] application 不依赖 interfaces
  - [x] application 不直接依赖 adapters
  - [x] adapters 不依赖 interfaces
  - [x] 循环依赖检测

---

## 四、Phase 1：核心能力（共 22 个任务）

> **目标**：完善核心功能，实现白皮书描述的全部基础能力
> **预计周期**：4-6 周
> **里程碑**：v0.1 — 单 Cell 开发可用

---

### PHASE1-TASK-001：完善五维熵值计算模型

- **优先级**：P0
- **产出**：`src/domain/entropy.rs`, `src/application/entropy_service.rs`
- **依赖**：PHASE0-TASK-005
- **验收标准**：
  - [ ] 结构熵（25% 权重）：文件大小分布、目录深度、模块均匀度
  - [ ] 复杂度熵（25% 权重）：圈复杂度、认知复杂度、嵌套深度
  - [ ] 耦合熵（20% 权重）：入度/出度、循环依赖、跨层依赖
  - [ ] 命名熵（15% 权重）：命名一致性、缩写使用率、标识符熵
  - [ ] 测试熵（15% 权重）：测试覆盖率、断言密度、测试完整性
  - [ ] 基于 Shannon 熵的数学计算正确
  - [ ] 熵值区间 0-100，分级准确（健康/注意/警告/危险/灾难）
  - [ ] 单元测试覆盖率 ≥ 90%
  - [ ] 性能：< 100ms / 千行代码

---

### PHASE1-TASK-002：熵值门禁命令

- **优先级**：P0
- **产出**：`src/application/entropy_service.rs`, CLI 集成
- **依赖**：PHASE1-TASK-001
- **验收标准**：
  - [ ] `cell entropy guard --threshold 40` 命令
  - [ ] 总熵值超标 → 非零退出码
  - [ ] 单维度熵增长超限 → 警告
  - [ ] 支持 --json 输出格式
  - [ ] CI 集成验证通过

---

### PHASE1-TASK-003：熵值趋势对比

- **优先级**：P1
- **产出**：`src/application/entropy_service.rs`
- **依赖**：PHASE1-TASK-001
- **验收标准**：
  - [ ] `cell entropy diff <commit-a> <commit-b>`
  - [ ] 计算两次提交的熵值变化量
  - [ ] 识别新增高熵文件
  - [ ] 输出增量报告

---

### PHASE1-TASK-004：完善静态检查规则（20+ 条）

- **优先级**：P0
- **产出**：`src/application/arch_service.rs`, `src/domain/` 新增规则模块
- **依赖**：PHASE0-TASK-006
- **验收标准**：

  **分层规则（L 系列）**：
  - [ ] L001: Domain 层不能 import Application/Adapters/Interfaces
  - [ ] L002: Application 层不能 import Interfaces
  - [ ] L003: 包之间不能有循环依赖
  - [ ] L004: Port 必须是 trait，不能有实现
  - [ ] L005: Adapter 只能向内依赖（Domain + Application 的 Port）
  - [ ] L006: UseCase 必须在 application 层

  **复杂度规则（C 系列）**：
  - [ ] C001: 函数圈复杂度 ≤ 15
  - [ ] C002: 函数长度 ≤ 80 行
  - [ ] C003: 嵌套深度 ≤ 4 层
  - [ ] C004: 函数参数 ≤ 6 个
  - [ ] C005: 结构体字段 ≤ 15 个

  **命名规则（N 系列）**：
  - [ ] N001: Port 接口使用名词或 -er 后缀
  - [ ] N002: UseCase 使用动词 + 名词命名
  - [ ] N003: Adapter 命名包含技术栈名 + Port 名
  - [ ] N004: 测试函数名清晰表达测试意图

  **测试规则（T 系列）**：
  - [ ] T001: 核心模块测试覆盖率 ≥ 85%
  - [ ] T002: Domain 层测试覆盖率 ≥ 95%
  - [ ] T003: 每个 Public 函数至少有一个测试

  - [ ] 所有规则可单独开关
  - [ ] 规则配置文件（cell.yaml）支持

---

### PHASE1-TASK-005：cell validate 命令完善

- **优先级**：P0
- **产出**：CLI 集成 + 验证报告
- **依赖**：PHASE1-TASK-004
- **验收标准**：
  - [ ] `cell validate [path]` 命令
  - [ ] 输出结构化报告（JSON/Table）
  - [ ] 违规按严重程度分级（error/warning/info）
  - [ ] 每个违规包含修复建议
  - [ ] --rules 参数指定规则集
  - [ ] 非零退出码表示有 error 级违规

---

### PHASE1-TASK-006：代码生成器 - 领域构件生成

- **优先级**：P0
- **产出**：`src/adapters/template_engine.rs`, `src/application/generate_service.rs`
- **依赖**：PHASE0-TASK-008
- **验收标准**：
  - [ ] `cell generate aggregate <name> --fields "id:ID,name:String"`
  - [ ] `cell generate entity <name> --fields "..."`
  - [ ] `cell generate valueobject <name> --fields "..."`
  - [ ] `cell generate domain-service <name>`
  - [ ] `cell generate event <name> --fields "..."`
  - [ ] 生成对应单元测试模板
  - [ ] 生成领域事件定义
  - [ ] 不变量检查桩（invariants）

---

### PHASE1-TASK-007：代码生成器 - 用例生成

- **优先级**：P0
- **产出**：模板扩展
- **依赖**：PHASE1-TASK-006
- **验收标准**：
  - [ ] `cell generate usecase <name>`
  - [ ] 生成 UseCase Port 接口
  - [ ] 生成 UseCase 实现骨架
  - [ ] 生成 Command / DTO 类型
  - [ ] 生成集成测试模板
  - [ ] 自动注入 Trace 埋点模板

---

### PHASE1-TASK-008：代码生成器 - 适配器生成

- **优先级**：P1
- **产出**：模板扩展
- **依赖**：PHASE1-TASK-007
- **验收标准**：
  - [ ] `cell generate adapter http --port <UseCasePort>`
  - [ ] `cell generate adapter repository --port <RepoPort> --kind postgres`
  - [ ] `cell generate adapter message --port <PublisherPort> --kind kafka`
  - [ ] HTTP 适配器：路由 + Handler + DTO 转换
  - [ ] Repository 适配器：CRUD 方法骨架
  - [ ] 自动注入 Metrics + Trace + Log 埋点

---

### PHASE1-TASK-009：cell.yaml 配置文件规范

- **优先级**：P0
- **产出**：`src/domain/` 配置模型 + 加载器
- **验收标准**：
  - [ ] Cell 元数据（name, version, description）
  - [ ] 熵值阈值配置
  - [ ] 规则集配置（启用/禁用规则）
  - [ ] 代码生成模板配置
  - [ ] Schema 验证
  - [ ] `cell config validate` 命令

---

### PHASE1-TASK-010：entropy.yaml 熵值配置

- **优先级**：P1
- **产出**：熵值配置加载
- **依赖**：PHASE1-TASK-009
- **验收标准**：
  - [ ] 五维权重配置
  - [ ] 阈值配置（总熵 + 单维度）
  - [ ] 忽略文件/目录配置
  - [ ] 复杂度阈值微调

---

### PHASE1-TASK-011：自动埋点能力

- **优先级**：P1
- **产出**：代码生成模板集成
- **依赖**：PHASE1-TASK-008
- **验收标准**：
  - [ ] UseCase 自动注入 Trace Span
  - [ ] Repository 操作自动埋点
  - [ ] HTTP Handler 自动埋点
  - [ ] 消息发布/消费自动埋点
  - [ ] 标准 Cell 属性（cell.name, cell.layer, cell.usecase）
  - [ ] 可配置开关（--no-telemetry）

---

### PHASE1-TASK-012：Cell 标准指标体系

- **优先级**：P1
- **产出**：`src/domain/metrics.rs`, 生成模板
- **验收标准**：
  - [ ] 请求类指标：cell_requests_total, duration, active, errors
  - [ ] 事件类指标：published, consumed, processing_duration, dlq
  - [ ] 领域类指标：aggregates_total, domain_errors, business_rules
  - [ ] Prometheus 格式导出
  - [ ] 代码生成时自动接入

---

### PHASE1-TASK-013：功能开关基础框架

- **优先级**：P1
- **产出**：`src/domain/feature.rs` 扩展
- **验收标准**：
  - [ ] Release Flag 类型（功能发布）
  - [ ] Ops Flag 类型（运维控制）
  - [ ] Experiment Flag 类型（A/B 测试）
  - [ ] 开关配置加载（YAML/JSON）
  - [ ] 运行时切换 API
  - [ ] `cell feature list` 命令

---

### PHASE1-TASK-014：cell handoff 交接命令

- **优先级**：P1
- **产出**：`src/application/handoff_service.rs`
- **依赖**：PHASE0-TASK-009, PHASE0-TASK-010
- **验收标准**：
  - [ ] `cell handoff generate` 生成交接清单
  - [ ] 包含：当前任务、相关文件、决策记录、未解决问题
  - [ ] `cell handoff show [path]` 读取并展示
  - [ ] JSON / Markdown 双格式输出
  - [ ] 交接清单包含 ADR 摘要

---

### PHASE1-TASK-015：问题指纹库原型

- **优先级**：P2
- **产出**：`src/domain/fingerprint.rs`
- **验收标准**：
  - [ ] 10+ 常见问题指纹定义
  - [ ] 指纹匹配算法（代码模式 + 错误模式）
  - [ ] `cell diagnose` 命令
  - [ ] 每个指纹包含：现象、根因、修复方案
  - [ ] 可扩展的指纹注册机制

---

### PHASE1-TASK-016：Feature Unit 生命周期管理

- **优先级**：P2
- **产出**：`src/application/feature_service.rs`
- **依赖**：PHASE0-TASK-011
- **验收标准**：
  - [ ] `cell feature create <name>`
  - [ ] `cell feature mount <name>`
  - [ ] `cell feature unmount <name>`
  - [ ] `cell feature retire <name>`
  - [ ] 生命周期状态流转验证
  - [ ] Feature 元数据持久化

---

### PHASE1-TASK-017：扩展点框架（5 种类型）

- **优先级**：P2
- **产出**：`src/domain/extension.rs`
- **验收标准**：
  - [ ] Validation 扩展点（验证规则）
  - [ ] Calculation 扩展点（计算逻辑）
  - [ ] Notification 扩展点（通知渠道）
  - [ ] Export 扩展点（导出格式）
  - [ ] Transformation 扩展点（数据转换）
  - [ ] Fallback 机制（扩展点失败降级）
  - [ ] 注册发现 API

---

### PHASE1-TASK-018：Feature Runtime

- **优先级**：P2
- **产出**：`src/application/feature_runtime.rs`
- **依赖**：PHASE1-TASK-017
- **验收标准**：
  - [ ] 扩展点注册中心
  - [ ] 扩展点调度器（优先级排序）
  - [ ] 降级管理器（Fallback 链）
  - [ ] 运行时监控（扩展点调用计数、延迟）

---

### PHASE1-TASK-019：原子性挂载/卸载

- **优先级**：P2
- **产出**：Feature Runtime 扩展
- **依赖**：PHASE1-TASK-018
- **验收标准**：
  - [ ] 4 阶段执行：Prepare → Validate → Activate → Commit
  - [ ] 失败自动回滚
  - [ ] 幂等性保证
  - [ ] 挂载/卸载日志完整

---

### PHASE1-TASK-020：六大不变量静态检查

- **优先级**：P1
- **产出**：架构规则扩展
- **依赖**：PHASE1-TASK-004
- **验收标准**：

  **积木式架构六大不变量**：
  - [ ] INV01: 依赖向内不变（外层只能依赖内层）
  - [ ] INV02: 端口契约不变（Port 接口语义稳定）
  - [ ] INV03: 隔离性不变（Cell 间不共享内部状态）
  - [ ] INV04: 事件契约不变（事件 Schema 向前兼容）
  - [ ] INV05: 可测试性不变（核心逻辑可独立测试）
  - [ ] INV06: 可删除性不变（功能可整体删除）

  - [ ] 每个不变量对应静态检查规则
  - [ ] 违规自动识别

---

### PHASE1-TASK-021：cell arch 架构命令集

- **优先级**：P1
- **产出**：CLI 架构命令完善
- **依赖**：PHASE1-TASK-005, PHASE1-TASK-020
- **验收标准**：
  - [ ] `cell arch validate` 架构验证
  - [ ] `cell arch visualize` 输出架构图（Mermaid/DOT）
  - [ ] `cell arch overview` 架构总览
  - [ ] `cell arch graph` 依赖图
  - [ ] `cell arch advise` 架构建议（基于规则违规）

---

### PHASE1-TASK-022：v0.1 发布准备

- **优先级**：P0
- **产出**：发布文档 + 示例
- **依赖**：Phase 1 全部任务
- **验收标准**：
  - [ ] README.md 完整重写
  - [ ] 使用文档（Quick Start / Tutorial）
  - [ ] 示例 Cell 项目
  - [ ] CHANGELOG.md
  - [ ] 版本号 0.1.0
  - [ ] 全量测试通过

---

## 五、Phase 2：多 Cell 协作（共 18 个任务）

> **目标**：支持多 Cell 系统开发，事件驱动 + 分布式追踪
> **预计周期**：4-6 周
> **里程碑**：v0.5 — 多 Cell 系统可用

| 任务 | 名称 | 优先级 | 核心产出 |
|------|------|--------|---------|
| PHASE2-TASK-001 | 事件 Schema 管理 | P0 | events.proto 生成 + 兼容性检查 |
| PHASE2-TASK-002 | Saga 模式代码生成 | P0 | Saga 编排模板 + 补偿逻辑 |
| PHASE2-TASK-003 | 跨 Cell 契约测试 | P0 | contract test 框架 + 生成器 |
| PHASE2-TASK-004 | 跨 Cell 熵值计算 | P0 | 系统级熵值聚合 + 跨 Cell 耦合熵 |
| PHASE2-TASK-005 | W3C Trace Context 支持 | P1 | 分布式追踪集成 |
| PHASE2-TASK-006 | 依赖图谱实时追踪 | P1 | 系统依赖图 + 影响分析 |
| PHASE2-TASK-007 | 自动 RCA 引擎 v1 | P1 | 日志+指标+Trace 关联分析 |
| PHASE2-TASK-008 | 熵值银行系统 | P1 | 跨功能熵值额度流转 |
| PHASE2-TASK-009 | 复杂度配额体系 | P2 | 系统/Cell/团队/功能四级配额 |
| PHASE2-TASK-010 | 功能组合框架 | P2 | 基于扩展点的功能拼装 |
| PHASE2-TASK-011 | 模式识别引擎 | P2 | 重复代码检测 + 相似模式识别 |
| PHASE2-TASK-012 | 影响范围分析器 | P2 | 删除前自动评估影响 |
| PHASE2-TASK-013 | K8s 部署模板生成 | P1 | Deployment + Service + HPA |
| PHASE2-TASK-014 | 三级健康探针生成 | P1 | Startup/Liveness/Readiness |
| PHASE2-TASK-015 | 数据库迁移工具 | P2 | 扩张-收缩模式迁移脚本 |
| PHASE2-TASK-016 | 配置中心集成 | P2 | Nacos/Consul + 配置版本化 |
| PHASE2-TASK-017 | 批量操作两阶段提交 | P2 | 2PC for Features |
| PHASE2-TASK-018 | 五级优雅降级 | P2 | Level 0-4 降级框架 |

---

## 六、Phase 3：插件与扩展（共 12 个任务）

> **目标**：插件生态 + 架构智能化
> **预计周期**：3-4 周
> **里程碑**：v0.8 — 插件生态可用

| 任务 | 名称 | 优先级 | 核心产出 |
|------|------|--------|---------|
| PHASE3-TASK-001 | 插件系统实现 | P0 | 插件加载器 + 生命周期管理 |
| PHASE3-TASK-002 | 插件安全沙箱 | P0 | 权限控制 + 资源限制 |
| PHASE3-TASK-003 | 插件验证工具 | P1 | 插件合规性检查 |
| PHASE3-TASK-004 | 服务网格集成 | P1 | Istio/Linkerd 配置生成 |
| PHASE3-TASK-005 | 金丝雀发布 | P1 | 灰度放量 + 自动回滚 |
| PHASE3-TASK-006 | 业务规则版本化 | P2 | 规则引擎 + 版本管理 |
| PHASE3-TASK-007 | A/B 实验开关 | P2 | Experiment Flag 增强 |
| PHASE3-TASK-008 | 问题指纹库扩展 | P2 | 50+ 指纹 |
| PHASE3-TASK-009 | 自动 RCA v2 | P2 | 依赖图 + 影响范围评估 |
| PHASE3-TASK-010 | 架构建议命令 | P1 | cell arch-advisor |
| PHASE3-TASK-011 | 模式库管理系统 | P2 | Pattern Library |
| PHASE3-TASK-012 | 重构辅助工具 | P2 | cell refactor |

---

## 七、Phase 4：生产就绪（共 14 个任务）

> **目标**：v1.0 生产级质量
> **预计周期**：4-6 周
> **里程碑**：v1.0 — 生产可用

| 任务 | 名称 | 优先级 | 核心产出 |
|------|------|--------|---------|
| PHASE4-TASK-001 | 完整文档体系 | P0 | API 文档 + 教程 + 最佳实践 |
| PHASE4-TASK-002 | 性能优化 | P0 | 基准测试 + 性能调优 |
| PHASE4-TASK-003 | 安全审计 | P0 | 零信任 + DevSecOps |
| PHASE4-TASK-004 | 示例项目 | P0 | 完整电商系统示例 |
| PHASE4-TASK-005 | SRE 运维自动化 | P1 | Runbook + 故障自愈 |
| PHASE4-TASK-006 | 混沌工程 | P1 | 故障注入 + 演练 |
| PHASE4-TASK-007 | FinOps 成本治理 | P2 | 资源画像 + 成本分摊 |
| PHASE4-TASK-008 | RTO/RPO 保障 | P1 | 多活 + 备份恢复 |
| PHASE4-TASK-009 | 完整版本治理 | P0 | 12 类版本对象 + SemVer |
| PHASE4-TASK-010 | 无感迭代全链路验证 | P0 | 部署+接口+数据+业务 |
| PHASE4-TASK-011 | 智能体无损交接 | P1 | 端到端验证 > 99.9% |
| PHASE4-TASK-012 | 一秒定位问题 | P1 | P1 问题 < 10 秒 |
| PHASE4-TASK-013 | 自适应复杂度验证 | P2 | 万行到亿行平滑演进 |
| PHASE4-TASK-014 | 熵值稳态验证 | P2 | 新增 100 功能，熵增长 < 5% |

---

## 八、开发铁律（强制执行）

### 8.1 架构分层铁律（6 条）

1. Domain 层 **绝对不能** import Application/Adapters/Interfaces
2. Application 层 **不能** import Interfaces
3. Application 层 **只能依赖 Port 接口**，不能依赖 Adapter 实现
4. Adapters 层 **只能向内依赖**（Domain + Application 的 Port）
5. 循环依赖 = 编译错误
6. 新增代码必须通过 `architecture_tests`

### 8.2 代码质量铁律（5 条）

1. **禁止** `unwrap()` / `expect()`（除非明确证明安全）
2. 公共 API **必须** 有文档注释
3. 核心逻辑 **必须** 有单元测试（覆盖率 ≥ 85%）
4. **禁止** 新增 Clippy warning
5. 每个函数 **必须** 有单一职责

### 8.3 开发流程铁律（5 条）

1. 提交前必须通过：fmt → clippy → test → arch → entropy
2. PR 必须通过 CI 全量检查
3. 新增功能必须对应：实现 + 测试 + 文档
4. 架构变更必须有 Micro-ADR
5. 熵值增长超限 = 不能合并

---

## 九、验收矩阵（每阶段必过）

| 检查项 | Phase 0 | Phase 1 | Phase 2 | Phase 3 | Phase 4 |
|--------|---------|---------|---------|---------|---------|
| 编译通过 | ✅ | ✅ | ✅ | ✅ | ✅ |
| 单元测试通过 | ✅ | ✅ | ✅ | ✅ | ✅ |
| Clippy 0 warnings | ✅ | ✅ | ✅ | ✅ | ✅ |
| 架构测试通过 | ✅ | ✅ | ✅ | ✅ | ✅ |
| 熵值 < 阈值 | ✅ | ✅ | ✅ | ✅ | ✅ |
| 测试覆盖率 ≥ 80% | ⬜ | ✅ | ✅ | ✅ | ✅ |
| 文档完整 | ⬜ | ⬜ | ✅ | ✅ | ✅ |
| 示例项目 | ⬜ | ⬜ | ⬜ | ✅ | ✅ |
| 性能基准 | ⬜ | ⬜ | ⬜ | ⬜ | ✅ |
| 安全审计 | ⬜ | ⬜ | ⬜ | ⬜ | ✅ |

---

## 十、下一步行动

### 立即开始（本周）

1. ✅ **PHASE0-TASK-008**：完成 cell generate 系列命令（进行中）
2. 🔜 **PHASE1-TASK-001**：完善五维熵值计算模型
3. 🔜 **PHASE1-TASK-004**：完善 20+ 静态检查规则
4. 🔜 **PHASE1-TASK-005**：完善 cell validate 命令

### 本阶段目标

完成 Phase 0 全部任务 + Phase 1 核心任务（P0 级），达到 v0.1 可发布状态。
