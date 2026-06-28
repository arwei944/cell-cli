# Cell Architecture 最强约束体系开发计划

> 文档版本：v1.0
> 创建日期：2026-06-27
> 核心哲学：**约束越强，自由越大。能在编译期拦住的，绝对不留到运行期。**
> 依据：六层铁闸体系 + 全面架构审查 + [PROJECT_RULES.md](PROJECT_RULES.md)

---

## 一、总体架构：六层铁闸

### 1.1 防御体系

```
第六层：AI 实时拦截（IDE 期，毫秒级）
    ↓ 漏网之鱼
第五层：编译期铁闸（cargo build，秒级） ← 最强核心
    ↓ 漏网之鱼
第四层：测试铁闸（cargo test，分钟级）
    ↓ 漏网之鱼
第三层：pre-commit 铁闸（git commit，秒级）
    ↓ 漏网之鱼
第二层：CI 铁闸（GitHub Actions，10 分钟级）
    ↓ 漏网之鱼
第一层：发布门禁（release 流程，终极防线）
```

### 1.2 验收标准六级体系（零容忍版）

| 等级 | 标识 | 内容 | 验证方式 | 零容忍 |
|------|------|------|---------|--------|
| L0 | 🔒 物理隔离 | 多 crate 分层，依赖方向错误编译不通过 | Cargo workspace 结构 | 是 |
| L1 | ✅ 编译通过 | cargo build --release 零错误 | `cargo build --release` | 是 |
| L2 | 🧪 测试全绿 | 所有测试 100% 通过 + 变异测试 ≥ 80% | `cargo test` + 变异测试 | 是 |
| L3 | 🚨 Lint 零警告 | clippy 全 deny + 自定义架构 Lint | `cargo clippy -- -D warnings` | 是 |
| L4 | 🏛️ 架构合规 | 架构测试 + 架构验证双通过 | `cargo test architecture_tests` + `cell arch validate` | 是 |
| L5 | 📉 熵值只降不升 | 增量熵值 ≤ 0，绝对值低于阈值 | `cell entropy guard --incremental` | 是 |

### 1.3 最小任务单元定义

每个最小任务单元满足：
1. **单一目标**：只做一件事，一个验收点
2. **可独立验证**：有明确的验证命令
3. **不可绕过**：不完成就无法进入下一阶段
4. **2 小时内可完成**：代码量可控，风险可控

---

## 二、第一阶段：编译期铁闸（最强核心）

> **阶段目标**：违反架构规则的代码，从根上就编译不通过。
> **任务数**：12 个最小任务单元
> **预估周期**：5-7 天
> **优先级**：🔴 最高

### 1.1 多 crate 物理隔离

#### S0-T001：Cargo Workspace 骨架搭建

- **状态**：⏳ 待开始
- **产出**：
  - `Cargo.toml`（workspace 根）
  - `crates/cell-domain/Cargo.toml`
  - `crates/cell-application/Cargo.toml`
  - `crates/cell-adapters/Cargo.toml`
  - `crates/cell-interfaces/Cargo.toml`
  - `crates/cell-cli/Cargo.toml`（二进制入口）
- **修改内容**：
  1. 将根 `Cargo.toml` 改为 workspace 配置
  2. 创建 5 个新 crate，每个有独立的 Cargo.toml
  3. 配置依赖关系：
     - `cell-domain`：**零外部依赖**（只有 std）
     - `cell-application`：只依赖 `cell-domain`
     - `cell-adapters`：依赖 `cell-domain` + `cell-application`
     - `cell-interfaces`：依赖 `cell-application`
     - `cell-cli`：依赖 `cell-interfaces`
  4. 每个 crate 的 `[lib]` 配置正确的 path
- **验收标准**：
  - L0: ✅ Workspace 结构正确，`cargo build -p cell-domain` 能编译
  - L1: ✅ 5 个 crate 都能独立编译
  - L2: ✅ `cargo tree` 验证依赖方向正确
  - L3: ✅ `cell-domain` 的 Cargo.toml dependencies 为空（除了 std）
- **验证命令**：
  ```bash
  cargo build -p cell-domain
  cargo build -p cell-application
  cargo build -p cell-adapters
  cargo build -p cell-interfaces
  cargo tree -p cell-domain  # 应该只有 std
  cargo tree -p cell-application  # 只能看到 cell-domain
  ```
- **预估工作量**：2 小时
- **依赖**：无

#### S0-T002：Domain 层代码迁移到 cell-domain crate

- **状态**：⏳ 待开始
- **产出**：`crates/cell-domain/src/`
- **修改内容**：
  1. 将 `src/domain/` 下的所有文件移动到 `crates/cell-domain/src/`
  2. 创建 `crates/cell-domain/src/lib.rs`，re-export 所有公开类型
  3. 移除领域层代码中对外部 crate 的依赖（serde, chrono, uuid, thiserror）
  4. 用标准库替代外部库：
     - serde → 先移除 derive，序列化能力移到 application 层
     - chrono → 先用 `std::time::SystemTime` 替代，或定义抽象 trait
     - uuid → 先用 `String` 包装的 newtype 替代
     - thiserror → 手动实现 `Display` + `Error` trait
  5. 确保所有领域代码在 cell-domain crate 中能编译
- **验收标准**：
  - L0: ✅ `cargo build -p cell-domain` 编译通过
  - L1: ✅ `cell-domain` 的 Cargo.toml 中 `[dependencies]` 为空
  - L2: ✅ `grep -r "use serde\|use chrono\|use uuid\|use thiserror" crates/cell-domain/` 无结果
  - L3: ✅ 所有领域类型的公开 API 不变（通过 re-export 保证）
- **验证命令**：
  ```bash
  cargo build -p cell-domain
  cargo test -p cell-domain
  grep -r "extern crate\|use serde\|use chrono\|use uuid" crates/cell-domain/src/
  # 应该无输出
  ```
- **预估工作量**：8-12 小时
- **依赖**：S0-T001

#### S0-T003：Application 层代码迁移到 cell-application crate

- **状态**：⏳ 待开始
- **产出**：`crates/cell-application/src/`
- **修改内容**：
  1. 将 `src/application/` 下的所有文件移动到 `crates/cell-application/src/`
  2. 创建 `crates/cell-application/src/lib.rs`，re-export 所有公开服务
  3. 更新所有导入路径：`crate::domain::xxx` → `cell_domain::xxx`
  4. 移除对 adapters 和 interfaces 的直接依赖（如果有的话）
  5. Port trait 保留在 application 层（六边形架构的输出端口）
  6. 确保所有应用服务能编译
- **验收标准**：
  - L0: ✅ `cargo build -p cell-application` 编译通过
  - L1: ✅ `cargo tree -p cell-application` 只显示 cell-domain 依赖
  - L2: ✅ `grep -r "use crate::adapters\|use crate::interfaces" crates/cell-application/` 无结果
  - L3: ✅ 所有公开服务 API 不变
- **验证命令**：
  ```bash
  cargo build -p cell-application
  cargo test -p cell-application
  cargo tree -p cell-application | grep -E "cell-adapters|cell-interfaces"
  # 应该无输出
  ```
- **预估工作量**：4-6 小时
- **依赖**：S0-T002

#### S0-T004：Adapters 层代码迁移到 cell-adapters crate

- **状态**：⏳ 待开始
- **产出**：`crates/cell-adapters/src/`
- **修改内容**：
  1. 将 `src/adapters/` 下的所有文件移动到 `crates/cell-adapters/src/`
  2. 创建 `crates/cell-adapters/src/lib.rs`，只暴露构造函数
  3. 更新所有导入路径为 `cell_domain::` 和 `cell_application::`
  4. 所有适配器结构体设为私有，只通过 `dyn XxxPort` 暴露
  5. 确保所有适配器能编译并实现对应的 Port trait
- **验收标准**：
  - L0: ✅ `cargo build -p cell-adapters` 编译通过
  - L1: ✅ 每个适配器都实现了对应的 Port trait
  - L2: ✅ 适配器结构体都是私有的（不 pub）
  - L3: ✅ 只能通过 `create_xxx_adapter() -> dyn XxxPort` 获取实例
- **验证命令**：
  ```bash
  cargo build -p cell-adapters
  cargo test -p cell-adapters
  grep -r "pub struct .*Adapter" crates/cell-adapters/src/
  # 应该无输出（都是私有）
  ```
- **预估工作量**：2-3 小时
- **依赖**：S0-T003

#### S0-T005：Interfaces 层代码迁移到 cell-interfaces crate

- **状态**：⏳ 待开始
- **产出**：`crates/cell-interfaces/src/`
- **修改内容**：
  1. 将 `src/interfaces/` 下的所有文件移动到 `crates/cell-interfaces/src/`
  2. 创建 `crates/cell-interfaces/src/lib.rs`，暴露 CLI 类型
  3. 更新所有导入路径为 `cell_application::`
  4. 移除对 domain 层的直接依赖（如果有的话，通过 application 层间接使用）
  5. 确保 CLI 命令能编译
- **验收标准**：
  - L0: ✅ `cargo build -p cell-interfaces` 编译通过
  - L1: ✅ `cargo tree -p cell-interfaces` 只能看到 cell-application → cell-domain
  - L2: ✅ `grep -r "cell_domain\|crate::domain" crates/cell-interfaces/src/` 无结果
  - L3: ✅ 所有 CLI 命令结构不变
- **验证命令**：
  ```bash
  cargo build -p cell-interfaces
  cargo tree -p cell-interfaces
  grep -r "use cell_domain\|use crate::domain" crates/cell-interfaces/src/
  # 应该无输出
  ```
- **预估工作量**：3-4 小时
- **依赖**：S0-T004

#### S0-T006：CLI 二进制整合与全量编译验证

- **状态**：⏳ 待开始
- **产出**：`crates/cell-cli/src/main.rs` + 根 `Cargo.toml`
- **修改内容**：
  1. 创建 `cell-cli` crate 作为二进制入口
  2. 在 `main.rs` 中组装所有依赖（依赖注入）
  3. 调用 `cell-interfaces` 的 CLI 逻辑
  4. 更新根 `Cargo.toml` 的 workspace 成员
  5. 确保 `cargo build --release` 全量编译通过
  6. 确保 `cargo run -- --help` 能正常运行
- **验收标准**：
  - L0: ✅ `cargo build --release` 全量编译通过
  - L1: ✅ `cargo run -- --help` 正常输出
  - L2: ✅ `cargo test --workspace` 全部通过
  - L3: ✅ 最终二进制大小与之前相当（不超过 110%）
- **验证命令**：
  ```bash
  cargo build --release
  cargo run -- --help
  cargo test --workspace
  cargo tree --workspace
  ```
- **预估工作量**：2-3 小时
- **依赖**：S0-T005

### 1.2 模块可见性铁闸

#### S0-T007：Domain 层可见性收敛

- **状态**：⏳ 待开始
- **产出**：`crates/cell-domain/src/lib.rs` + 各领域模块
- **修改内容**：
  1. 审计所有 `pub struct`、`pub enum`、`pub fn`、`pub trait`
  2. 分类：
     - ✅ 确实需要对外暴露的（领域实体、错误类型、核心 trait）
     - ⚠️ 只在 crate 内部使用的（改为 `pub(crate)`）
     - ❌ 只在模块内使用的（改为私有）
  3. 逐项修改可见性
  4. 在 `lib.rs` 中统一 re-export 公开项
  5. 确保外部使用方（application 层）不受影响
- **验收标准**：
  - L1: ✅ 编译通过
  - L2: ✅ 测试全绿
  - L3: ✅ `pub` 项数量减少至少 50%
  - L4: ✅ 所有公开项都在 `lib.rs` 中显式 re-export
  - L5: ✅ 外部只能通过 `cell_domain::xxx` 访问，不能访问内部子模块
- **验证命令**：
  ```bash
  cargo build -p cell-domain
  cargo test -p cell-domain
  grep -r "^pub " crates/cell-domain/src/ | wc -l
  # 对比前后数量，减少 ≥ 50%
  ```
- **预估工作量**：3-4 小时
- **依赖**：S0-T006

#### S0-T008：Application 层可见性收敛

- **状态**：⏳ 待开始
- **产出**：`crates/cell-application/src/lib.rs` + 各服务模块
- **修改内容**：
  1. 审计每个应用服务的公开项
  2. 只保留"用例入口函数"为 `pub`（每个服务 3-5 个）
  3. 所有辅助函数、内部结构体改为私有或 `pub(crate)`
  4. Port trait 保持 `pub`（因为 adapters 层需要）
  5. 在 `lib.rs` 中统一 re-export 公开服务和 Port
- **验收标准**：
  - L1: ✅ 编译通过
  - L2: ✅ 测试全绿
  - L3: ✅ 每个服务的 pub 函数 ≤ 5 个
  - L4: ✅ 公开项数量减少至少 60%
  - L5: ✅ CLI 层功能完全不受影响
- **验证命令**：
  ```bash
  cargo build -p cell-application
  cargo test -p cell-application
  # 抽查几个服务
  grep -c "^pub fn" crates/cell-application/src/entropy_service.rs
  # 应该 ≤ 5
  ```
- **预估工作量**：3-4 小时
- **依赖**：S0-T007

#### S0-T009：Adapters 层完全隐藏实现

- **状态**：⏳ 待开始
- **产出**：`crates/cell-adapters/src/lib.rs`
- **修改内容**：
  1. 所有适配器结构体改为私有（去掉 `pub`）
  2. 只暴露构造函数：`pub fn create_file_decision_store() -> Arc<dyn DecisionStorePort>`
  3. 所有内部辅助函数、内部类型全私有
  4. 在 `lib.rs` 中只导出构造函数和 Port trait 重新导出
  5. 确保 application 层只通过 Port trait 与 adapters 交互
- **验收标准**：
  - L1: ✅ 编译通过
  - L2: ✅ 测试全绿
  - L3: ✅ `grep -r "pub struct" crates/cell-adapters/src/` 无结果
  - L4: ✅ 外部只能看到构造函数，看不到具体类型
  - L5: ✅ 替换适配器实现不影响上层（可独立测试）
- **验证命令**：
  ```bash
  cargo build -p cell-adapters
  grep -r "^pub struct" crates/cell-adapters/src/
  # 应该无输出
  ```
- **预估工作量**：1-2 小时
- **依赖**：S0-T008

### 1.3 类型系统加固

#### S0-T010：核心 ID 类型 newtype 化

- **状态**：⏳ 待开始
- **产出**：领域层 ID 类型
- **修改内容**：
  1. 识别所有用 String 表示的 ID：
     - `FeatureId`
     - `CellId`
     - `PluginId`
     - `DecisionId`
     - `EvolutionId`
     - 等等
  2. 每个 ID 定义为 newtype：`pub struct FeatureId(String)`
  3. 实现构造函数 `new() -> CellResult<Self>`，验证合法性
  4. 实现 `Display`、`Clone`、`PartialEq`、`Eq`、`Hash` 等必要 trait
  5. 替换所有直接用 String 的地方
- **验收标准**：
  - L1: ✅ 编译通过
  - L2: ✅ 测试全绿
  - L3: ✅ 至少 10 种 ID 改为 newtype
  - L4: ✅ 非法 ID 无法构造（构造函数返回 Err）
  - L5: ✅ 不同类型的 ID 不能混用（编译期保证）
- **验证命令**：
  ```bash
  cargo test -p cell-domain
  # 验证：把 FeatureId 传给期望 CellId 的函数，应该编译不通过
  ```
- **预估工作量**：4-6 小时
- **依赖**：S0-T007

#### S0-T011：状态机类型级实现（核心样本）

- **状态**：⏳ 待开始
- **产出**：`crates/cell-domain/src/feature.rs` 或 saga.rs
- **修改内容**：
  1. 选一个核心状态机（Feature 或 Saga）
  2. 将运行时状态检查改为编译期类型保证
  3. 每个状态是独立的类型：`DesignFeature`, `DevelopmentFeature`, `TestingFeature`...
  4. 合法的状态转移实现 `From` trait，非法转移不实现
  5. 提供 `into_xxx()` 方法进行状态转移
  6. 保持对外 API 兼容（或提供适配层）
- **验收标准**：
  - L1: ✅ 编译通过
  - L2: ✅ 测试全绿
  - L3: ✅ 非法状态转移写不出（编译报错）
  - L4: ✅ 不需要运行时 `can_transition()` 检查
  - L5: ✅ 原有功能完全兼容（通过外观模式包装）
- **验证命令**：
  ```bash
  cargo test -p cell-domain
  # 验证：尝试写 Design → Production 的转移代码，应该编译不通过
  ```
- **预估工作量**：3-4 小时
- **依赖**：S0-T010

#### S0-T012：值对象合法性编译期保证

- **状态**：⏳ 待开始
- **产出**：领域层值对象
- **修改内容**：
  1. 识别所有关键值对象：
     - 版本号（SemVer）
     - 熵值（0-100 之间）
     - 百分比（0-100 之间）
     - 路径（非空，合法格式）
  2. 每个值对象用 newtype 包装
  3. 构造函数验证合法性，非法值无法构造
  4. 实现必要的 trait（Display, From, TryFrom 等）
  5. 替换所有裸类型
- **验收标准**：
  - L1: ✅ 编译通过
  - L2: ✅ 测试全绿
  - L3: ✅ 至少 5 种值对象 newtype 化
  - L4: ✅ 非法值无法构造（编译期或运行时构造失败）
  - L5: ✅ 不影响性能（零成本抽象）
- **验证命令**：
  ```bash
  cargo test -p cell-domain
  ```
- **预估工作量**：2-3 小时
- **依赖**：S0-T010

### 第一阶段整体验收

| 等级 | 标准 | 验证命令 | 必须通过 |
|------|------|----------|---------|
| L0 | 物理隔离有效 | 依赖方向错误编译不通过 | ✅ 是 |
| L1 | 全量编译通过 | `cargo build --release` | ✅ 是 |
| L2 | 全量测试通过 | `cargo test --workspace` | ✅ 是 |
| L3 | 可见性收敛 | pub 项减少 ≥ 50% | ✅ 是 |
| L4 | 领域层零外部依赖 | cell-domain 的 dependencies 为空 | ✅ 是 |
| L5 | 类型系统加固 | 至少 15 个 newtype + 1 个类型级状态机 | ✅ 是 |

---

## 三、第二阶段：测试铁闸

> **阶段目标**：测试不仅验证功能，更要锁死架构。测试不过，代码不算写完。
> **任务数**：8 个最小任务单元
> **预估周期**：4-6 天
> **优先级**：🔴 高

### 2.1 架构测试扩充

#### S1-T001：分层依赖测试全覆盖

- **状态**：⏳ 待开始
- **产出**：`crates/cell-application/src/architecture_tests/`（独立模块）
- **修改内容**：
  1. 将架构测试从单个文件拆分为独立模块目录
  2. 增加测试用例，从 8 个扩充到 30+ 个
  3. 覆盖所有分层依赖方向（正反向都测）：
     - domain → application ✅ 应该失败
     - domain → adapters ✅ 应该失败
     - domain → interfaces ✅ 应该失败
     - application → domain ✅ 应该通过
     - application → adapters ✅ 应该失败
     - application → interfaces ✅ 应该失败
     - adapters → domain ✅ 应该通过
     - adapters → application ✅ 应该通过（只通过 Port）
     - adapters → interfaces ✅ 应该失败
     - interfaces → application ✅ 应该通过
     - interfaces → domain ❓ 现在是怎样？应该禁止还是允许？
  4. 每个测试都有明确的断言和错误消息
  5. 测试失败时输出具体的违规文件和行号
- **验收标准**：
  - L1: ✅ 编译通过
  - L2: ✅ `cargo test --workspace architecture_tests` 全部通过
  - L3: ✅ 测试用例 ≥ 30 个
  - L4: ✅ 每个方向都有正反向测试
  - L5: ✅ 失败消息清晰，包含文件路径和行号
- **验证命令**：
  ```bash
  cargo test --workspace architecture_tests
  ```
- **预估工作量**：3-4 小时
- **依赖**：第一阶段完成

#### S1-T002：模块可见性测试

- **状态**：⏳ 待开始
- **产出**：架构测试模块
- **修改内容**：
  1. 增加领域层公开项数量测试：
     - domain crate 的公开 struct ≤ 30 个
     - domain crate 的公开 enum ≤ 10 个
     - domain crate 的公开 trait ≤ 10 个
  2. 增加应用层公开项数量测试：
     - 每个服务的 pub 函数 ≤ 5 个
     - 应用层 pub 结构体总数 ≤ 20 个
  3. 增加适配器层可见性测试：
     - adapters crate 没有 pub struct（除了构造函数返回的 trait 对象）
     - 适配器内部类型全是私有的
  4. 测试通过反射/文档分析来统计公开项数量
- **验收标准**：
  - L1: ✅ 编译通过
  - L2: ✅ 架构测试全部通过
  - L3: ✅ 可见性测试 ≥ 10 个
  - L4: ✅ 超限会导致测试失败
- **验证命令**：
  ```bash
  cargo test --workspace architecture_tests::visibility
  ```
- **预估工作量**：2-3 小时
- **依赖**：S1-T001

#### S1-T003：代码质量量化测试

- **状态**：⏳ 待开始
- **产出**：架构测试模块
- **修改内容**：
  1. 增加文件大小测试：
     - 每个 .rs 文件 ≤ 500 行（测试文件除外）
     - 超过则测试失败
  2. 增加函数长度测试：
     - 每个函数 ≤ 50 行（测试函数除外）
     - 超过则测试失败
  3. 增加参数数量测试：
     - 每个函数参数 ≤ 6 个
     - 超过则测试失败
  4. 增加结构体字段数测试：
     - 每个结构体字段 ≤ 15 个
     - 超过则测试失败
  5. 实现一个简单的代码统计工具（或用 syn 解析）
- **验收标准**：
  - L1: ✅ 编译通过
  - L2: ✅ 架构测试全部通过
  - L3: ✅ 代码质量测试 ≥ 8 个
  - L4: ✅ 超限文件会被列出来，测试失败
- **验证命令**：
  ```bash
  cargo test --workspace architecture_tests::code_quality
  ```
- **预估工作量**：3-4 小时
- **依赖**：S1-T002

#### S1-T004：最佳实践禁令测试

- **状态**：⏳ 待开始
- **产出**：架构测试模块
- **修改内容**：
  1. 增加 `unwrap()` 禁令测试：
     - 生产代码（非 test 模块）零 `unwrap()`
     - 零 `expect("...")`
     - 违反则测试失败
  2. 增加 `unsafe` 禁令测试：
     - 生产代码零 `unsafe` 块
     - 违反则测试失败
  3. 增加 `todo!()` 禁令测试：
     - 生产代码零 `todo!()`
     - 零 `unimplemented!()`
     - 违反则测试失败
  4. 增加循环依赖检测测试：
     - 模块之间无循环依赖
     - 违反则测试失败
  5. 用文件扫描 + 简单语法分析实现
- **验收标准**：
  - L1: ✅ 编译通过
  - L2: ✅ 架构测试全部通过
  - L3: ✅ 最佳实践测试 ≥ 6 个
  - L4: ✅ 违规文件和行号清晰列出
- **验证命令**：
  ```bash
  cargo test --workspace architecture_tests::best_practices
  ```
- **预估工作量**：2-3 小时
- **依赖**：S1-T003

### 2.2 变异测试接入

#### S1-T005：变异测试框架接入

- **状态**：⏳ 待开始
- **产出**：Cargo.toml + 变异测试配置
- **修改内容**：
  1. 选择变异测试工具（`cargo-mutants` 推荐）
  2. 在 CI 中加入变异测试步骤
  3. 配置变异测试范围（先从核心模块开始）
  4. 设置突变得分门槛：核心模块 ≥ 80%
  5. 编写变异测试运行说明
- **验收标准**：
  - L1: ✅ `cargo mutants` 能正常运行
  - L2: ✅ 核心模块（domain 层）变异得分 ≥ 70%
  - L3: ✅ CI 中有变异测试步骤（可选：先 warn，后面再 deny）
  - L4: ✅ 有配置文件和运行文档
- **验证命令**：
  ```bash
  cargo mutants --list
  cargo mutants -p cell-domain
  ```
- **预估工作量**：2-3 小时
- **依赖**：第一阶段完成

#### S1-T006：核心领域模块测试增强

- **状态**：⏳ 待开始
- **产出**：领域层测试代码
- **修改内容**：
  1. 运行变异测试，找出"漏测"的代码
  2. 针对变异测试发现的漏洞，补充测试用例
  3. 重点模块：
     - entropy.rs（熵值计算，核心中的核心）
     - saga.rs（状态机，逻辑复杂）
     - feature.rs（生命周期管理）
     - errors.rs（错误处理）
  4. 将变异得分提升到 80% 以上
- **验收标准**：
  - L1: ✅ 编译通过
  - L2: ✅ `cargo test -p cell-domain` 全绿
  - L3: ✅ 核心模块变异得分 ≥ 80%
  - L4: ✅ 新增测试用例 ≥ 30 个
- **验证命令**：
  ```bash
  cargo mutants -p cell-domain -- -j 4
  # 查看突变检测率
  ```
- **预估工作量**：4-6 小时
- **依赖**：S1-T005

#### S1-T007：应用服务测试增强

- **状态**：⏳ 待开始
- **产出**：应用层测试代码
- **修改内容**：
  1. 对核心应用服务补充集成测试
  2. 使用 mock 对象测试 Port 交互
  3. 重点服务：
     - arch_service / arch_linter
     - entropy_service
     - feature_service
     - enforcement_service
  4. 每个核心服务至少 10 个测试用例
  5. 覆盖正常路径、异常路径、边界条件
- **验收标准**：
  - L1: ✅ 编译通过
  - L2: ✅ `cargo test -p cell-application` 全绿
  - L3: ✅ 核心服务测试用例 ≥ 10 个/每个
  - L4: ✅ 异常路径覆盖率 ≥ 50%
- **验证命令**：
  ```bash
  cargo test -p cell-application
  ```
- **预估工作量**：4-6 小时
- **依赖**：S1-T006

#### S1-T008：测试架构自身的测试（元测试）

- **状态**：⏳ 待开始
- **产出**：测试验证脚本
- **修改内容**：
  1. 验证架构测试真的能检测出问题：
     - 故意在 domain 里加一个 `use cell_application::xxx`，看测试会不会挂
     - 故意加一个 `unwrap()`，看测试会不会挂
     - 故意写一个 600 行的文件，看测试会不会挂
  2. 编写"测试有效性验证"脚本
  3. 每个架构测试都有对应的"故意违规"验证
  4. 确保没有"假阳性"（没问题误报）也没有"假阴性"（有问题没查到）
- **验收标准**：
  - L1: ✅ 验证脚本可运行
  - L2: ✅ 每个架构测试都能正确检测出违规
  - L3: ✅ 误报率 = 0
  - L4: ✅ 漏报率 = 0（对于已知的违规模式）
- **验证命令**：
  ```bash
  # 运行验证脚本
  ./scripts/verify-architecture-tests.sh
  ```
- **预估工作量**：2-3 小时
- **依赖**：S1-T004, S1-T007

### 第二阶段整体验收

| 等级 | 标准 | 验证命令 | 必须通过 |
|------|------|----------|---------|
| L1 | 编译通过 | `cargo build --release` | ✅ 是 |
| L2 | 测试全绿 | `cargo test --workspace` | ✅ 是 |
| L3 | 架构测试 ≥ 50 个 | `cargo test --workspace architecture_tests -- --list` | ✅ 是 |
| L4 | 变异得分 ≥ 80% | `cargo mutants -p cell-domain` | ✅ 是 |
| L5 | 测试有效性验证通过 | 元测试脚本通过 | ✅ 是 |

---

## 四、第三阶段：Lint 铁闸

> **阶段目标**：所有 Warning 都是 Error，没有例外。代码刚写出来就被标红。
> **任务数**：6 个最小任务单元
> **预估周期**：3-4 天
> **优先级**：🔴 高

### 3.1 Clippy 零容忍拉满

#### S2-T001：Clippy 全规则启用

- **状态**：⏳ 待开始
- **产出**：根 `Cargo.toml` [lints] 配置
- **修改内容**：
  1. 将 clippy lint 级别拉满：
     - `all = "deny"`
     - `pedantic = "deny"`
     - `nursery = "warn"`（观察期，稳定后转 deny）
     - `cargo = "deny"`
  2. 逐个修复现有的 clippy 警告
  3. 对确实需要保留的，添加 `#[allow(clippy::xxx)]` 并注释理由
  4. allow 列表必须控制在 5 个以内
- **验收标准**：
  - L1: ✅ 编译通过
  - L2: ✅ `cargo clippy --workspace -- -D warnings` 零警告
  - L3: ✅ allow 列表 ≤ 5 项，每项都有注释理由
  - L4: ✅ pedantic 级别的 lint 全部 deny
- **验证命令**：
  ```bash
  cargo clippy --workspace --all-targets --all-features -- -D warnings
  echo $?  # 应该是 0
  ```
- **预估工作量**：4-6 小时
- **依赖**：第一阶段完成

#### S2-T002：Rust 内置 lint 全 deny

- **状态**：⏳ 待开始
- **产出**：根 `Cargo.toml` [lints.rust] 配置
- **修改内容**：
  1. 启用并 deny 所有内置 lint：
     - `unsafe_code = "deny"`
     - `unused_imports = "deny"`
     - `unused_variables = "deny"`
     - `dead_code = "deny"`
     - `unused_must_use = "deny"`
     - 等等，所有能开的都开
  2. 修复所有现有问题
  3. allow 列表控制在 3 个以内
- **验收标准**：
  - L1: ✅ 编译通过
  - L2: ✅ clippy 零警告
  - L3: ✅ 内置 lint 至少 deny 15 项
  - L4: ✅ unsafe_code = deny（生产代码零 unsafe）
- **验证命令**：
  ```bash
  cargo build --workspace --all-targets
  # 应该零警告
  ```
- **预估工作量**：2-3 小时
- **依赖**：S2-T001

### 3.2 自定义架构 Lint

#### S2-T003：自定义架构 Lint 框架

- **状态**：⏳ 待开始
- **产出**：`crates/cell-arch-lint/`（新增 proc-macro crate）
- **修改内容**：
  1. 创建 proc-macro crate 或使用 dylint 框架
  2. 实现自定义 lint 的基础设施
  3. 实现 lint 注册机制
  4. 实现 lint 报告输出格式
  5. 集成到 cargo clippy 或单独的 `cargo cell-lint` 命令
- **验收标准**：
  - L1: ✅ Lint 框架能编译运行
  - L2: ✅ 至少有 1 个示例 lint 能工作
  - L3: ✅ 输出格式与 clippy 兼容（便于 IDE 集成）
  - L4: ✅ 有文档说明如何添加新 lint
- **预估工作量**：4-6 小时
- **依赖**：S2-T002

#### S2-T004：分层依赖 Lint 规则集

- **状态**：⏳ 待开始
- **产出**：自定义 lint 规则
- **修改内容**：
  1. 实现 6 条分层依赖 lint（全部 deny 级别）：
     - `domain_no_application_dep` → domain 用了 application → deny
     - `domain_no_adapters_dep` → domain 用了 adapters → deny
     - `domain_no_interfaces_dep` → domain 用了 interfaces → deny
     - `application_no_interfaces_dep` → application 用了 interfaces → deny
     - `application_no_adapters_dep` → application 直接用 adapters（不用 Port）→ deny
     - `adapters_no_interfaces_dep` → adapters 用了 interfaces → deny
  2. 每条 lint 都有清晰的错误消息和修复建议
  3. 测试每条 lint 都能正确触发
- **验收标准**：
  - L1: ✅ 编译通过
  - L2: ✅ 6 条分层 lint 全部实现
  - L3: ✅ 每条 lint 都有正反向测试
  - L4: ✅ 误报率 = 0
- **验证命令**：
  ```bash
  # 故意写违规代码，检查是否能检测到
  cargo cell-lint
  ```
- **预估工作量**：4-6 小时
- **依赖**：S2-T003

#### S2-T005：代码质量 Lint 规则集

- **状态**：⏳ 待开始
- **产出**：自定义 lint 规则
- **修改内容**：
  1. 实现代码质量相关 lint：
     - `file_too_long` → 文件超过 500 行 → deny
     - `fn_too_long` → 函数超过 50 行 → warn（逐步转 deny）
     - `too_many_params` → 函数参数超过 6 个 → warn
     - `struct_too_many_fields` → 结构体超过 15 字段 → warn
     - `no_unwrap_in_prod` → 非 test 代码用 unwrap → deny
     - `no_todo_in_prod` → 非 test 代码用 todo! → deny
  2. 每条 lint 都有明确的严重级别
  3. 每条 lint 都有修复建议
- **验收标准**：
  - L1: ✅ 编译通过
  - L2: ✅ 至少 6 条代码质量 lint
  - L3: ✅ 每条都有测试
  - L4: ✅ 能正确检测，不误报
- **预估工作量**：3-4 小时
- **依赖**：S2-T004

#### S2-T006：命名规范 Lint 规则集

- **状态**：⏳ 待开始
- **产出**：自定义 lint 规则
- **修改内容**：
  1. 实现命名规范 lint：
     - `port_naming` → Port trait 用 -er 后缀或名词 → warn
     - `adapter_naming` → Adapter 包含技术栈名 → warn
     - `usecase_naming` → UseCase 用动词+名词 → info
     - `test_naming` → 测试函数清晰表达意图 → info
  2. 集成到项目中
  3. 修复所有现有命名违规
- **验收标准**：
  - L1: ✅ 编译通过
  - L2: ✅ 至少 4 条命名 lint
  - L3: ✅ 现有代码零违规（已经修复）
  - L4: ✅ 有文档说明命名规范
- **预估工作量**：2-3 小时
- **依赖**：S2-T005

### 第三阶段整体验收

| 等级 | 标准 | 验证命令 | 必须通过 |
|------|------|----------|---------|
| L1 | 编译通过 | `cargo build --release` | ✅ 是 |
| L2 | Clippy 零警告 | `cargo clippy --workspace -- -D warnings` | ✅ 是 |
| L3 | 自定义 lint ≥ 16 条 | lint 列表统计 | ✅ 是 |
| L4 | 所有 lint 零违规 | `cargo cell-lint` | ✅ 是 |
| L5 | allow 列表 ≤ 8 项 | 人工检查 | ✅ 是 |

---

## 五、第四阶段：提交铁闸

> **阶段目标**：违规代码连提交都提交不了。本地就拦住，别等到 CI。
> **任务数**：6 个最小任务单元
> **预估周期**：2-3 天
> **优先级**：🟠 高

### 4.1 pre-commit 强制化

#### S3-T001：build.rs 自动检查钩子安装

- **状态**：⏳ 待开始
- **产出**：`build.rs`（根目录）
- **修改内容**：
  1. 创建 `build.rs` 构建脚本
  2. 每次 `cargo build` 时检查：
     - `.git/hooks/pre-commit` 是否存在
     - 是否指向项目的 `.githooks/pre-commit`
  3. 如果没装，输出 **编译错误**（不是警告！）
     - 错误消息："Git hooks not installed. Run `make setup-hooks` to install."
  4. 提供环境变量绕过（仅 CI 使用）：`CELL_SKIP_HOOK_CHECK=1`
- **验收标准**：
  - L1: ✅ 没装钩子时，`cargo build` 编译失败
  - L2: ✅ 装了钩子时，编译正常
  - L3: ✅ 设置环境变量时可绕过（给 CI 用）
  - L4: ✅ 错误消息清晰，告诉用户怎么修
- **验证命令**：
  ```bash
  # 没装钩子时
  cargo build 2>&1 | grep "Git hooks not installed"
  # 应该能看到错误

  # 装了钩子时
  make setup-hooks
  cargo build
  # 应该正常
  ```
- **预估工作量**：1-2 小时
- **依赖**：第三阶段完成

#### S3-T002：pre-commit 脚本增强

- **状态**：⏳ 待开始
- **产出**：`.githooks/pre-commit` + `.githooks/pre-commit.ps1`
- **修改内容**：
  1. 增强 pre-commit 脚本，从 5 项增加到 8 项：
     1. `cargo fmt --check`（格式）
     2. `cargo clippy -- -D warnings`（Lint）
     3. 自定义架构 lint
     4. `cargo test --lib`（单元测试，只跑变更相关）
     5. `cargo test architecture_tests`（架构测试）
     6. `cell arch validate -p .`（架构验证）
     7. 增量熵值检查
     8. 提交消息格式检查
  2. 优化输出格式，清晰好看
  3. 失败时给出具体的修复命令
  4. 保持 bash 和 PowerShell 双版本同步
- **验收标准**：
  - L1: ✅ 脚本能正常运行
  - L2: ✅ 8 项检查全部执行
  - L3: ✅ 任何一项失败，提交被阻止
  - L4: ✅ 失败消息清晰，有修复建议
  - L5: ✅ bash 和 PowerShell 版本功能一致
- **预估工作量**：2-3 小时
- **依赖**：S3-T001

#### S3-T003：增量检查优化（速度优先）

- **状态**：⏳ 待开始
- **产出**：pre-commit 脚本 + 增量检查工具
- **修改内容**：
  1. pre-commit 只检查本次变更的文件，不是全量：
     - `cargo fmt` 只格式化变更的文件
     - clippy 只 check 变更的 crate
     - 测试只跑变更模块相关的
     - 架构验证只检查变更涉及的层
     - 熵值只算增量（变更文件的熵值变化）
  2. 用 `git diff --cached` 获取变更文件列表
  3. 智能判断哪些 crate 需要重新测试
  4. **目标：pre-commit 总耗时 ≤ 15 秒**
- **验收标准**：
  - L1: ✅ 增量检查能正确运行
  - L2: ✅ 变更少量文件时，耗时 ≤ 15 秒
  - L3: ✅ 不会漏检（变更相关的都查到了）
  - L4: ✅ 全量检查（全部文件都变更）也能工作
- **验证命令**：
  ```bash
  # 测速度：只改一个文件，看 pre-commit 多久
  time git commit -m "test" --dry-run
  ```
- **预估工作量**：3-4 小时
- **依赖**：S3-T002

### 4.2 其他钩子

#### S3-T004：commit-msg 钩子

- **状态**：⏳ 待开始
- **产出**：`.githooks/commit-msg` + `.githooks/commit-msg.ps1`
- **修改内容**：
  1. 实现提交消息格式检查：
     - 必须符合 Conventional Commits 规范
     - 类型必须在允许列表内（feat, fix, docs, style, refactor, test, chore, arch...）
     - 标题长度 ≤ 72 字符
     - 正文与标题之间有空行
  2. 不符合的提交直接拒绝
  3. 给出清晰的格式说明和示例
- **验收标准**：
  - L1: ✅ 合法消息能通过
  - L2: ✅ 非法消息被拒绝
  - L3: ✅ 错误消息清晰，有示例
  - L4: ✅ 双版本（bash + PowerShell）功能一致
- **预估工作量**：1-2 小时
- **依赖**：S3-T003

#### S3-T005：pre-push 钩子

- **状态**：⏳ 待开始
- **产出**：`.githooks/pre-push` + `.githooks/pre-push.ps1`
- **修改内容**：
  1. pre-push 运行更全面的检查：
     - 全量 clippy
     - 全量单元测试
     - 全量架构测试
     - 全量熵值检查
  2. 推送前必须通过，防止"我本地没问题"
  3. 耗时较长（可能 2-5 分钟），所以放在 pre-push 而不是 pre-commit
  4. 有跳过选项，但默认启用
- **验收标准**：
  - L1: ✅ 脚本能正常运行
  - L2: ✅ 4 项全量检查都执行
  - L3: ✅ 失败则推送被阻止
  - L4: ✅ 有清晰的进度提示
- **预估工作量**：1-2 小时
- **依赖**：S3-T004

#### S3-T006：钩子管理命令

- **状态**：⏳ 待开始
- **产出**：`cell enforcement hooks` 命令
- **修改内容**：
  1. 给 cell CLI 增加钩子管理子命令：
     - `cell hooks install` - 安装所有钩子
     - `cell hooks uninstall` - 卸载所有钩子
     - `cell hooks status` - 查看钩子状态
     - `cell hooks list` - 列出所有钩子和检查项
  2. 跨平台支持（Windows / macOS / Linux）
  3. 与 Makefile 的 `setup-hooks` 功能一致但更强大
- **验收标准**：
  - L1: ✅ 命令能正常运行
  - L2: ✅ install / uninstall / status / list 都工作
  - L3: ✅ 跨平台兼容
  - L4: ✅ 有帮助文档和示例
- **预估工作量**：2-3 小时
- **依赖**：S3-T005

### 第四阶段整体验收

| 等级 | 标准 | 验证命令 | 必须通过 |
|------|------|----------|---------|
| L1 | 钩子强制安装 | 不装钩子编译不通过 | ✅ 是 |
| L2 | pre-commit 8 项检查 | 故意违规，验证被拦 | ✅ 是 |
| L3 | 增量检查 ≤ 15 秒 | 小变更测速 | ✅ 是 |
| L4 | commit-msg 生效 | 非法消息被拒 | ✅ 是 |
| L5 | pre-push 生效 | 全量检查失败则推送失败 | ✅ 是 |

---

## 六、第五阶段：CI 铁闸

> **阶段目标**：CI 是最后一道防线，必须是最坚固的。红着的代码绝对合并不了。
> **任务数**：7 个最小任务单元
> **预估周期**：3-4 天
> **优先级**：🟠 高

### 5.1 CI 七重门禁

#### S4-T001：CI 流水线重构

- **状态**：⏳ 待开始
- **产出**：`.github/workflows/ci.yml`
- **修改内容**：
  1. 将 CI 重构为 7 个 Job，按顺序依赖：
     1. **format-check** → 格式检查（最快，最先反馈）
     2. **clippy-check** → Lint 检查
     3. **arch-tests** → 架构测试
     4. **unit-tests** → 单元测试
     5. **arch-validate** → 架构验证工具
     6. **entropy-gate** → 熵值门禁
     7. **integration-tests** → 集成测试 + 多平台构建
  2. 前面的 Job 失败，后面的不跑（快速失败）
  3. 配置缓存（cargo cache, target cache）
  4. 配置重试（网络问题自动重试）
- **验收标准**：
  - L1: ✅ CI 配置语法正确
  - L2: ✅ 7 个 Job 按顺序执行
  - L3: ✅ 快速失败生效（前面挂了后面不跑）
  - L4: ✅ 缓存配置正确，第二次运行快 50%+
- **验证命令**：
  ```bash
  # 推送到测试分支，观察 CI 执行
  ```
- **预估工作量**：2-3 小时
- **依赖**：第四阶段完成

#### S4-T002：增量熵值门禁

- **状态**：⏳ 待开始
- **产出**：CI entropy-gate Job
- **修改内容**：
  1. 实现增量熵值检查，不是看绝对值：
     - 与主分支对比
     - 计算变更文件的熵值变化
     - 增量熵值 > 0 → 失败（只能降不能升）
     - 或者增量 > 0.1 → 失败（允许微小波动）
  2. 输出详细的熵值变化报告
  3. 标注哪些文件导致了熵值上升
  4. 给出改进建议
- **验收标准**：
  - L1: ✅ 熵值下降 → 通过
  - L2: ✅ 熵值上升超阈值 → 失败
  - L3: ✅ 报告清晰，有具体文件和数值
  - L4: ✅ 与基线对比准确
- **预估工作量**：2-3 小时
- **依赖**：S4-T001

#### S4-T003：零新增违规原则

- **状态**：⏳ 待开始
- **产出**：CI architecture-gate Job
- **修改内容**：
  1. 架构验证不仅看"有没有违规"，更看"违规数有没有增加"：
     - 与主分支对比违规数量
     - 新增违规 → 失败
     - 违规减少 → 通过（鼓励修历史问题）
  2. 输出新增违规的详细列表
  3. 区分 Error 级和 Warning 级：
     - Error 级零新增 → 硬门槛
     - Warning 级零新增 → 软门槛（先 warn，以后 deny）
- **验收标准**：
  - L1: ✅ 新增 Error 级违规 → CI 失败
  - L2: ✅ 违规数减少 → CI 通过
  - L3: ✅ 报告清晰，列出具体新增违规
  - L4: ✅ 历史违规不影响（只看增量）
- **预估工作量**：2-3 小时
- **依赖**：S4-T002

### 5.2 CI 增强

#### S4-T004：多平台测试矩阵

- **状态**：⏳ 待开始
- **产出**：CI 配置
- **修改内容**：
  1. 扩展测试矩阵：
     - 操作系统：Ubuntu, macOS, Windows
     - Rust 版本：stable, beta, nightly（nightly 允许失败）
     - 架构：x86_64, aarch64（如果有条件）
  2. 核心功能在所有平台都要通过
  3. nightly 版本允许失败（仅作观察）
  4. 配置合理的并行度
- **验收标准**：
  - L1: ✅ 至少 3 个操作系统都通过
  - L2: ✅ stable 和 beta 版本都通过
  - L3: ✅ nightly 有结果（允许失败）
  - L4: ✅ 并行配置合理，总耗时不翻倍
- **预估工作量**：2-3 小时
- **依赖**：S4-T003

#### S4-T005：构建时间门禁

- **状态**：⏳ 待开始
- **产出**：CI 配置 + 基准记录
- **修改内容**：
  1. 记录构建时间基线：
     - 全量构建时间
     - 增量构建时间
     - 测试运行时间
  2. CI 中加入时间检查：
     - 构建时间超过基线 15% → 警告
     - 超过 30% → 失败
  3. 防止代码越来越臃肿，构建越来越慢
  4. 时间数据归档，可看趋势
- **验收标准**：
  - L1: ✅ 有基线数据
  - L2: ✅ 超时会告警/失败
  - L3: ✅ 有历史趋势记录
  - L4: ✅ 不影响正常构建流程
- **预估工作量**：1-2 小时
- **依赖**：S4-T004

#### S4-T006：代码覆盖率门禁

- **状态**：⏳ 待开始
- **产出**：CI coverage Job
- **修改内容**：
  1. 接入 codecov 或 tarpaulin
  2. 设置覆盖率门槛：
     - 总体覆盖率 ≥ 70%
     - 核心模块（domain）≥ 85%
     - 覆盖率不能下降（只能升）
  3. PR 评论显示覆盖率变化
  4. 不达标则 CI 失败
- **验收标准**：
  - L1: ✅ 覆盖率数据正常生成
  - L2: ✅ PR 中有覆盖率报告
  - L3: ✅ 覆盖率下降 → CI 失败
  - L4: ✅ 有历史趋势
- **预估工作量**：2-3 小时
- **依赖**：S4-T005

#### S4-T007：发布门禁与自动化

- **状态**：⏳ 待开始
- **产出**：`.github/workflows/release.yml`
- **修改内容**：
  1. 实现自动化发布流程：
     - tag 触发 release
     - 自动构建多平台二进制
     - 自动运行所有检查（7 重门禁）
     - 自动生成 changelog
     - 自动发布到 GitHub Releases
  2. 发布前必须通过所有检查
  3. 发布版本号自动校验（符合 semver）
  4. 发布后自动更新文档
- **验收标准**：
  - L1: ✅ 打 tag 能触发 release 流程
  - L2: ✅ 所有检查不通过则发布失败
  - L3: ✅ 多平台二进制自动构建
  - L4: ✅ changelog 自动生成
- **预估工作量**：3-4 小时
- **依赖**：S4-T006

### 第五阶段整体验收

| 等级 | 标准 | 验证命令 | 必须通过 |
|------|------|----------|---------|
| L1 | CI 七重门禁 | 7 个 Job 全绿 | ✅ 是 |
| L2 | 增量熵值门禁 | 熵值上升则失败 | ✅ 是 |
| L3 | 零新增违规 | 新增违规则失败 | ✅ 是 |
| L4 | 多平台通过 | 3 系统 × 2 Rust 版本 | ✅ 是 |
| L5 | 覆盖率不下降 | 覆盖率降低则失败 | ✅ 是 |
| L6 | 发布自动化 | tag 触发自动发布 | ✅ 是 |

---

## 七、第六阶段：AI 增强与自举验证

> **阶段目标**：写代码的时候就不犯错，而且工具自己证明自己有用。
> **任务数**：5 个最小任务单元
> **预估周期**：5-7 天（持续迭代）
> **优先级**：🟡 中

### 6.1 AI 辅助开发

#### S5-T001：IDE 架构提示扩展

- **状态**：⏳ 待开始
- **产出**：`.vscode/settings.json` + 扩展推荐
- **修改内容**：
  1. 配置 VS Code 开发环境：
     - rust-analyzer 配置（开启所有 lint）
     - clippy 实时检查
     - rustfmt 保存自动格式化
     - 自定义架构 lint 集成
  2. 推荐扩展列表（`.vscode/extensions.json`）
  3. 工作区设置（`.vscode/settings.json`）
  4. 代码片段（架构相关的代码模板）
  5. 调试配置
- **验收标准**：
  - L1: ✅ 配置文件完整
  - L2: ✅ 保存自动格式化
  - L3: ✅ clippy 错误实时显示
  - L4: ✅ 有架构相关代码片段
- **预估工作量**：1-2 小时
- **依赖**：第五阶段完成

#### S5-T002：AI 代码审查机器人

- **状态**：⏳ 待开始
- **产出**：`.github/workflows/ai-review.yml` + 审查脚本
- **修改内容**：
  1. 实现 PR 自动 AI 审查：
     - 每个 PR 自动触发 AI 架构审查
     - AI 检查：分层违规、命名问题、复杂度问题、测试缺失
     - 在 PR 评论中输出审查结果
     - 按严重程度分类：🔴 必须改 / 🟡 建议改 / 🟢 表扬
  2. 集成到 GitHub Actions
  3. 可配置审查深度
  4. 不替代人审，是给人审提供弹药
- **验收标准**：
  - L1: ✅ PR 自动触发审查
  - L2: ✅ 审查报告格式清晰
  - L3: ✅ 能发现已知的架构问题
  - L4: ✅ 不阻塞合并（仅作建议，可后续加强）
- **预估工作量**：3-4 小时
- **依赖**：S5-T001

#### S5-T003：自动修复机器人

- **状态**：⏳ 待开始
- **产出**：自动修复工具
- **修改内容**：
  1. 实现简单违规的自动修复：
     - 格式问题 → 自动 cargo fmt
     - 简单 clippy 警告 → 自动 cargo fix
     - 简单命名问题 → 自动重命名（谨慎）
     - 导入排序 → 自动整理
  2. 提供 `cell arch fix` 命令
  3. 自动修复后自动跑测试验证
  4. 修复失败则回滚
- **验收标准**：
  - L1: ✅ `cell arch fix` 命令可运行
  - L2: ✅ 至少能自动修复 3 类问题
  - L3: ✅ 修复后测试仍然通过
  - L4: ✅ 修复失败能安全回滚
- **预估工作量**：3-4 小时
- **依赖**：S5-T002

### 6.2 自举验证

#### S5-T004：自举（Bootstrapping）验证流程

- **状态**：⏳ 待开始
- **产出**：自举验证脚本 + CI Job
- **修改内容**：
  1. 实现自举验证流程：
     - 用上一个版本的 cell 工具，验证当前版本的代码
     - 检查架构违规、熵值、测试覆盖率
     - 对比两个版本的工具输出是否一致
     - 验证通过才能发布
  2. 集成到 release 流程
  3. 发布前必须通过自举验证
  4. 记录每次发布的自举结果
- **验收标准**：
  - L1: ✅ 自举脚本能运行
  - L2: ✅ 发布前自动执行
  - L3: ✅ 自举不通过则发布失败
  - L4: ✅ 有自举验证报告
- **预估工作量**：2-3 小时
- **依赖**：S5-T003

#### S5-T005：自我进化闭环

- **状态**：⏳ 待开始
- **产出**：进化文档 + 流程
- **修改内容**：
  1. 建立"发现问题 → 变成规则 → 工具检查 → 防止再犯"的闭环流程
  2. 每次发现一个新的架构问题：
     - 记录到问题库
     - 分析根因
     - 决定是否要加为新规则
     - 如果加，实现 lint 规则 + 架构测试
     - 更新文档
  3. 每月做一次"架构健康度复盘"
  4. 建立规则生命周期管理（新增 → 观察 → 稳定 → 固化）
- **验收标准**：
  - L1: ✅ 有闭环流程文档
  - L2: ✅ 有问题库和规则库
  - L3: ✅ 有定期复盘机制
  - L4: ✅ 规则有明确的生命周期
- **预估工作量**：2-3 小时
- **依赖**：S5-T004

### 第六阶段整体验收

| 等级 | 标准 | 验证方式 | 必须通过 |
|------|------|---------|---------|
| L1 | IDE 实时提示 | 写代码时实时显示架构错误 | ✅ 是 |
| L2 | AI 审查机器人 | PR 自动生成审查报告 | ✅ 是 |
| L3 | 自动修复 | `cell arch fix` 能修复简单问题 | ✅ 是 |
| L4 | 自举验证 | 发布前自动自举验证 | ✅ 是 |
| L5 | 进化闭环 | 有明确的规则迭代流程 | ✅ 是 |

---

## 八、整体里程碑与验收总表

### 8.1 六个阶段总览

| 阶段 | 名称 | 任务数 | 预估工作量 | 核心目标 |
|------|------|--------|-----------|---------|
| 0 | 编译期铁闸 | 12 | 30-40 小时 | 违规代码编译不通过 |
| 1 | 测试铁闸 | 8 | 22-32 小时 | 测试不仅测功能，更锁架构 |
| 2 | Lint 铁闸 | 6 | 16-24 小时 | 所有 Warning 都是 Error |
| 3 | 提交铁闸 | 6 | 10-16 小时 | 违规代码提交不了 |
| 4 | CI 铁闸 | 7 | 15-22 小时 | 红的代码合并不了 |
| 5 | AI 增强 | 5 | 11-16 小时 | 写代码时就不犯错 |
| **总计** | | **44** | **104-150 小时** | |

### 8.2 关键里程碑

| 里程碑 | 对应阶段 | 完成标志 |
|--------|---------|---------|
| M1：物理隔离 | 阶段 0 | 多 crate 拆分完成，领域层零外部依赖 |
| M2：测试锁架构 | 阶段 1 | 50+ 架构测试，变异得分 ≥ 80% |
| M3：Lint 全 deny | 阶段 2 | clippy 全规则 deny + 16 条自定义 lint |
| M4：本地拦截 | 阶段 3 | pre-commit 强制安装，8 道检查 |
| M5：CI 全绿 | 阶段 4 | 七重门禁，多平台全绿 |
| M6：自举闭环 | 阶段 5 | 自举验证 + 自动修复 + 进化闭环 |

### 8.3 最终验收总表

| 维度 | 现状 | 目标 | 验证方式 |
|------|------|------|---------|
| 架构违规检测时机 | 事后人工审查 | 编译期就报错 | 故意写违规代码，编译不通过 |
| 分层依赖保证 | 文档约定，靠自觉 | 物理隔离（多 crate） | cargo tree 验证依赖方向 |
| 领域层纯度 | 98% 模块有外部依赖 | 100% 零外部依赖 | cell-domain 的 dependencies 为空 |
| 架构测试数量 | 8 个（编译不通过） | 50+ 个（全绿） | cargo test architecture_tests |
| Lint 规则数量 | ~30 条（部分空实现） | 40+ 条（全部生效） | clippy + 自定义 lint 统计 |
| pre-commit | 有脚本，没安装 | 强制安装，8 道检查 | 不装钩子编译不通过 |
| CI 门禁 | 5 个 Job（通不过） | 7 个 Job（全绿 + 增量门禁） | CI 仪表盘全绿 |
| 变异测试覆盖率 | 无 | ≥ 80% | cargo mutants |
| 自举验证 | 无 | 发布前必须自举 | release 流程自动验证 |
| **总体** | 51/100 | **95/100** | 全面重新评估 |

### 8.4 零容忍清单

以下是**绝对零容忍**的规则，违反任何一条，代码都进不了主分支：

1. ❌ 领域层有任何外部依赖
2. ❌ 依赖方向错误（内层依赖外层）
3. ❌ 编译不通过
4. ❌ 测试失败
5. ❌ clippy 有警告
6. ❌ 架构测试失败
7. ❌ 新增 Error 级架构违规
8. ❌ 熵值上升超过阈值
9. ❌ 代码覆盖率下降
10. ❌ 生产代码出现 unwrap()

---

## 九、为什么这是"最强"的

### 9.1 对比传统方案

| 方案 | 检测时机 | 强制性 | 漏检率 |
|------|---------|--------|--------|
| 文档约定 | 全靠自觉 | 0% | 90%+ |
| Code Review | 提交后 | 依赖人 | 50%+ |
| CI 检查 | 提交后 | 中 | 20-30% |
| pre-commit | 提交时 | 高（可绕过） | 10-20% |
| Lint 工具 | 编码时 | 高（可 allow） | 5-10% |
| **多 crate 物理隔离** | **编译期** | **100%（绕不过）** | **0%** |
| **类型系统保证** | **编译期** | **100%（绕不过）** | **0%** |

### 9.2 核心理念

**最强的约束，是让你感觉不到约束的存在。**

就像 Rust 的借用检查器——你不用天天想"我有没有内存安全问题"，因为编译器已经替你保证了。你只需要专注于业务逻辑。

Cell Architecture 的终极目标也是一样：
- 你不用想"我有没有违反分层规则"——因为编译不过
- 你不用想"这个状态转移合不合法"——因为写不出来
- 你不用想"我忘没忘写测试"——因为变异测试会告诉你
- 你不用想"我能不能提交"——因为 pre-commit 会告诉你

**约束越强，自由越大。** 因为你不用把脑力浪费在"守规矩"上，全部脑力都可以用来"解决问题"。

---

*文档结束*
