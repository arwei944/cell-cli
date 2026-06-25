# Cell Architecture - Project Rules

## 核心铁律（零容忍）

以下规则是**不可违反**的硬约束，任何代码变更必须通过所有校验。

### 1. 架构分层约束（Architecture is the Prompt）

Cell 架构严格遵循四层分层，依赖只能向内：

```
interfaces → application → domain
              ↑
         adapters
```

- **domain 层**：禁止依赖任何外层（application/adapters/interfaces）
- **application 层**：禁止依赖 interfaces/adapters（只能依赖 domain + 通过 Port 与 adapters 交互）
- **adapters 层**：禁止依赖 interfaces
- **interfaces 层**：只能依赖 application（通过用例入口）

### 2. 模块可见性约束

- `domain/` 下的类型：只有真正需要对外暴露的才加 `pub`
- `application/` 下的服务：只有用例入口函数才 `pub`
- `adapters/` 下的实现：默认私有，通过 Port trait 暴露
- `interfaces/` 下的 CLI 类型：参数结构体可以 pub（与应用层解耦）

### 3. 依赖注入约束

- application 层不直接 `use adapters 模块
- adapters 通过实现 domain/application 中定义的 Port trait
- 依赖组装在 interfaces 层或 main 中完成

## 开发流程硬约束

### 4. 提交前必须通过（pre-commit）

- `cargo fmt --check` 格式化检查
- `cargo clippy -- -D warnings` Lint 检查
- `cargo test` 单元测试
- `cargo run -- arch validate -p .` 架构验证
- `cargo run -- entropy check src` 熵值检查（低于阈值）

### 5. CI 必须通过（全绿才能合并）

- 所有平台构建通过
- 所有测试通过
- 架构验证通过
- 熵值门禁通过
- 代码覆盖率不下降

## 代码规范

### 6. 命名约束

- 领域实体：名词（User, Order, Cell）
- 用例服务：动词+名词（InitService, EntropyService）
- 适配器：技术+Adapter（AstAnalyzer, FileSystemAdapter）
- Port trait：名词 + Port（RepositoryPort, NotificationPort）

### 7. 文件组织约束

- 每个文件不超过 300 行（超过必须拆分）
- 每个函数不超过 50 行（超过必须拆分）
- 每个模块最多 10 个公开项

## 熵值约束

### 8. 熵值门禁

- 整体熵值得分不得超过 5.0
- 单个文件复杂度熵值不得超过 20.0
- 每次提交熵值不得上升超过 0.5

## 禁止事项

### 9. 绝对禁止

- ❌ domain 层直接使用任何 I/O（文件、网络、环境变量）
- ❌ application 层直接使用第三方库（除了基础库和领域库）
- ❌ 循环依赖
- ❌ 全局可变状态
- ❌ unwrap() 在生产代码中（测试除外）
- ❌ 新增依赖未在 Cargo.toml 中声明的外部 crate

## 验证机制

所有规则通过以下机制强制执行：

1. **编译期**：模块可见性 + 架构单元测试
2. **静态检查**：Clippy 自定义规则
3. **测试期**：架构测试 + 集成测试
4. **提交前**：pre-commit hooks
5. **CI 期**：GitHub Actions 全量检查
6. **运行期**：熵值门禁
7. **IDE 期**：TRAE 项目规则

任何一层失败，代码不能进入下一层。
