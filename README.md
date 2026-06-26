# Cell Architecture — 面向 AI 智能体原生开发的低熵架构工具链

> **Architecture is the Prompt（架构即提示）**

不是靠文档说服智能体遵守规则，而是靠类型系统和工具链强制执行规则。让 AI 智能体像工人在标准化工厂里生产零件一样生产代码——每个 Cell 都是标准件，每个步骤都有质检，每批产出都有熵值保障。

---

## 🚀 快速开始

### 安装与构建

```bash
# 克隆项目
git clone <repository-url>
cd cell-architecture

# 构建
cargo build --release

# 验证安装
./target/release/cell --help
```

### 一键初始化开发环境（智能体专用）

```bash
# 一键完成所有准备工作（环境检测+构建+注册+基线建立）
cargo run -- dev bootstrap
```

初始化完成后，你将获得：
- ✅ 工具链已构建
- ✅ Agent 身份已注册
- ✅ 架构基线已建立
- ✅ Git Hooks 已安装

### 常用命令速查

```bash
# 查看当前状态
cell dev status

# 获取下一步建议
cell dev next

# 查看待开发任务
cell task list

# 架构健康检查
cell arch lint

# 计算熵值
cell entropy current

# 自我验证（架构+测试+熵值）
cell verify

# 代码自我审查
cell self-review
```

---

## 📚 文档目录

### 🚀 快速开始

| 文档 | 说明 | 状态 |
|------|------|------|
| [README.md](./README.md) | 项目介绍 + 快速开始 + 完整命令索引 | ✅ 已完成 |
| [CLI_REFERENCE.md](docs/CLI_REFERENCE.md) | CLI 命令完整参考手册 | ✅ 已完成 |

### 🤖 智能体文档

| 文档 | 适用对象 | 说明 | 状态 |
|------|----------|------|------|
| [AGENT_GUIDE.md](docs/AGENT_GUIDE.md) | Developer 智能体 | 智能体开发操作手册 | ✅ 已完成 |
| [ARCHITECT_GUIDE.md](docs/ARCHITECT_GUIDE.md) | Architect 智能体 | 架构师操作指南 | ✅ 已完成 |
| [autonomous-agent-dev-plan.md](docs/autonomous-agent-dev-plan.md) | 全部 | 智能体自举式开发工作台开发计划 | ✅ 已完成 |

### 📘 架构文档

| 文档 | 说明 | 状态 |
|------|------|------|
| [ARCHITECTURE.md](docs/ARCHITECTURE.md) | 架构分层说明 | ✅ 已完成 |
| [cell-architecture-whitepaper.md](./cell-architecture-whitepaper.md) | Cell 架构白皮书（15章完整版） | ✅ 已完成 |
| [commercial-upgrade-roadmap.md](./commercial-upgrade-roadmap.md) | 商业级升级路线图 | ✅ 已完成 |

### 📄 白皮书章节目录

1.  **架构总纲** — 设计哲学、核心理念、设计原则
2.  **核心概念体系** — Cell单元、Port契约、Domain内核、Adapter适配器
3.  **分层架构模型** — 领域层 / 应用层 / 适配器层 / 基础设施层
4.  **智能体开发工作流** — 6种Agent角色 + 7步开发循环
5.  **低熵监控体系** — 五维熵值模型 + 异常检测 + 可视化
6.  **事件驱动与插件化** — 事件总线 + Saga + 插件系统
7.  **可观测性体系** — OpenTelemetry 三支柱 + 熵值指标
8.  **运维与治理体系** — K8s部署 + 弹性伸缩 + 零信任安全 + SRE
9.  **版本管理与无感迭代** — 12大版本化对象 + 四层无感迭代
10. **功能无限扩展与低熵保持** — 功能单元化 + 熵值银行 + 功能代谢
11. **积木式功能架构** — 标准接口 + 依赖反转 + 原子操作 + 六大不变量
12. **CLI 工具详细设计** — cell init / generate / entropy / validate
13. **技术实现方案** — 技术栈选型 + 代码生成器 + 静态分析
14. **落地路线图** — 4个Phase + 70+ 任务项
15. **总结** — 13大核心创新点 + 对比矩阵

## 核心特性

### 🤖 智能体原生
- Spec-First → 代码生成 → 填空式开发
- 6 个专业 Agent 角色分工协作
- 结构化上下文 + 无损交接
- 问题指纹库 + 自动 RCA，一秒定位

### 📊 低熵监控
- 五维熵值模型（结构/复杂度/耦合/命名/测试）
- 基于 Shannon 熵的严谨数学模型
- 熵值银行 + 熵增预算 + 熵值门禁
- 功能代谢：有增必有减，净熵增长趋近于零

### 🧱 积木式架构
- 功能增删像搭积木一样简单
- 标准接口 + 扩展点（5种类型）
- 依赖反转：删掉任何功能，核心照常跑
- 原子操作：挂载/卸载要么全成要么全败
- 六大不变量保证架构正确性

### 🚀 企业级成熟
- OpenTelemetry 全链路可观测
- K8s 原生部署 + HPA 弹性伸缩
- 零信任安全 + DevSecOps
- 多活架构 + 灾备 + RTO/RPO 保障
- SRE 自动化 + 混沌工程 + FinOps

### 🔄 全维度版本管理
- 12 大版本化对象（代码/API/数据/事件/配置/规则/...）
- 四层无感迭代（部署/接口/数据/业务）
- API 多版本共存 + 数据库在线迁移
- 功能开关四级体系

### 📈 自适应规模
- 六级复杂度模型（L0 原型到 L5 巨型）
- 分形架构：从函数到系统，每层结构一样
- 自动检测架构健康度，主动建议升级时机
- 万行到亿行平滑演进，永远可以降级

## 架构全景图

```
┌─────────────────────────────────────────────────────────────┐
│                    可观测性（OTel 三支柱）                    │
│  Traces  │  Metrics  │  Logs  │  熵值指标                    │
└─────────────────────────────────────────────────────────────┘
                            │
┌─────────────────────────────────────────────────────────────┐
│                  运维与治理（SRE / DevOps）                   │
│  部署 │ 弹性 │ 安全 │ 配置 │ 灾备 │ 成本 │ 混沌               │
└─────────────────────────────────────────────────────────────┘
                            │
┌─────────────────────────────────────────────────────────────┐
│                  版本管理与无感迭代                           │
│  12类版本对象 │ 四层无感 │ API版本 │ 数据迁移 │ 功能开关      │
└─────────────────────────────────────────────────────────────┘
                            │
┌─────────────────────────────────────────────────────────────┐
│                  积木式功能架构                               │
│  FeatureUnit │ ExtensionPoint │ 依赖反转 │ 原子操作 │ 不变量 │
└─────────────────────────────────────────────────────────────┘
                            │
┌─────────────────────────────────────────────────────────────┐
│                   功能扩展与低熵保持                          │
│  功能单元化 │ 熵值银行 │ 功能代谢 │ 组合式扩展 │ 分形架构      │
└─────────────────────────────────────────────────────────────┘
                            │
┌─────────────────────────────────────────────────────────────┐
│                    事件驱动与插件化                           │
│  事件总线 │ Schema 注册 │ Saga │ 插件系统 │ 模式库            │
└─────────────────────────────────────────────────────────────┘
                            │
┌─────────────────────────────────────────────────────────────┐
│                   低熵监控体系（五维）                        │
│  结构熵 │ 复杂度熵 │ 耦合熵 │ 命名熵 │ 测试熵                  │
└─────────────────────────────────────────────────────────────┘
                            │
┌─────────────────────────────────────────────────────────────┐
│                     Cell 分层架构                             │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  Domain 层（领域核心）— 最稳定，最少修改                │  │
│  │  Application 层（应用用例）— 业务编排                   │  │
│  │  Adapter 层（适配器）— 外部交互，最易变                 │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

## 快速开始

### 阅读建议

1. **先读** [cell-architecture-whitepaper.md](./cell-architecture-whitepaper.md) 第一章，理解设计理念
2. **再看** 第二~四章，掌握核心概念和开发流程
3. **深入** 第五~十一章，了解各个子系统
4. **落地** 参考第十四章路线图，按阶段实施

### 已有项目升级

如果是从 low-entropy-core 升级到 Cell 架构，参考：
[commercial-upgrade-roadmap.md](./commercial-upgrade-roadmap.md)

## 与现有架构的对比

| 维度 | 传统架构 | low-entropy-core | Cell 架构 |
|------|---------|-----------------|-----------|
| 智能体友好度 | ⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| 熵值可观测性 | ⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| 积木式增删 | ⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐⭐ |
| 原子性保证 | ⭐⭐ | ⭐ | ⭐⭐⭐⭐⭐ |
| 功能代谢机制 | ⭐ | ⭐ | ⭐⭐⭐⭐⭐ |
| 无感迭代能力 | ⭐⭐ | ⭐ | ⭐⭐⭐⭐⭐ |
| 可观测性 | ⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐⭐ |
| 运维成熟度 | ⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐⭐ |
| 规模适应性 | ⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐⭐ |

## 版本信息

- **当前版本**: v1.0 (白皮书完整版)
- **最后更新**: 2026-06-25
- **文档总章节**: 15 章
- **核心创新点**: 13 个

## 许可

本架构设计文档仅供内部参考使用。

---

## 📖 完整命令索引

### 🏗️ 架构核心类

| 命令 | 说明 | 文档 |
|------|------|------|
| `cell init` | 初始化 Cell 项目 | [CLI 参考](docs/CLI_REFERENCE.md#init) |
| `cell generate` | 从规范生成代码 | [CLI 参考](docs/CLI_REFERENCE.md#generate) |
| `cell validate` | 验证架构规则 | [CLI 参考](docs/CLI_REFERENCE.md#validate) |
| `cell arch lint` | 架构规则检查 | [CLI 参考](docs/CLI_REFERENCE.md#arch) |
| `cell arch fix` | 自动修复架构违规 | [CLI 参考](docs/CLI_REFERENCE.md#arch) |
| `cell arch visualize` | 生成架构图 | [CLI 参考](docs/CLI_REFERENCE.md#arch) |
| `cell arch overview` | 架构概览 | [CLI 参考](docs/CLI_REFERENCE.md#arch) |
| `cell arch advise` | 架构改进建议 | [CLI 参考](docs/CLI_REFERENCE.md#arch) |

### 📊 熵值与质量

| 命令 | 说明 | 文档 |
|------|------|------|
| `cell entropy current` | 当前熵值 | [CLI 参考](docs/CLI_REFERENCE.md#entropy) |
| `cell entropy trend` | 熵值趋势 | [CLI 参考](docs/CLI_REFERENCE.md#entropy) |
| `cell entropy baseline` | 熵值基线管理 | [CLI 参考](docs/CLI_REFERENCE.md#entropy) |
| `cell lint` | 代码简洁度检查 | [CLI 参考](docs/CLI_REFERENCE.md#lint) |
| `cell review` | 代码审查 | [CLI 参考](docs/CLI_REFERENCE.md#review) |

### 🤖 智能体开发工作流

| 命令 | 说明 | 文档 |
|------|------|------|
| `cell dev bootstrap` | 一键初始化环境 | [CLI 参考](docs/CLI_REFERENCE.md#dev) |
| `cell dev status` | 开发环境状态 | [CLI 参考](docs/CLI_REFERENCE.md#dev) |
| `cell dev next` | 下一步建议 | [CLI 参考](docs/CLI_REFERENCE.md#dev) |
| `cell dev context` | 上下文快照 | [CLI 参考](docs/CLI_REFERENCE.md#dev) |
| `cell dev reset` | 重置环境 | [CLI 参考](docs/CLI_REFERENCE.md#dev) |
| `cell task list` | 任务列表 | [CLI 参考](docs/CLI_REFERENCE.md#task) |
| `cell task discover` | 发现任务 | [CLI 参考](docs/CLI_REFERENCE.md#task) |
| `cell task next` | 下一个任务 | [CLI 参考](docs/CLI_REFERENCE.md#task) |
| `cell task claim` | 认领任务 | [CLI 参考](docs/CLI_REFERENCE.md#task) |
| `cell task done` | 完成任务 | [CLI 参考](docs/CLI_REFERENCE.md#task) |

### ✅ 验证与门禁

| 命令 | 说明 | 文档 |
|------|------|------|
| `cell verify` | 快速验证 | [CLI 参考](docs/CLI_REFERENCE.md#verify) |
| `cell self-verify` | 自我验证与修复 | [CLI 参考](docs/CLI_REFERENCE.md#self-verify) |
| `cell self-review` | 自我代码审查 | [CLI 参考](docs/CLI_REFERENCE.md#self-review) |
| `cell enforcement status` | 强制门禁状态 | [CLI 参考](docs/CLI_REFERENCE.md#enforcement) |
| `cell enforcement install-hooks` | 安装 Git Hooks | [CLI 参考](docs/CLI_REFERENCE.md#enforcement) |

### 🧩 功能与模板

| 命令 | 说明 | 文档 |
|------|------|------|
| `cell feature list` | 功能单元列表 | [CLI 参考](docs/CLI_REFERENCE.md#feature) |
| `cell feature new` | 新建功能单元 | [CLI 参考](docs/CLI_REFERENCE.md#feature) |
| `cell feature mount` | 挂载功能单元 | [CLI 参考](docs/CLI_REFERENCE.md#feature) |
| `cell feature unmount` | 卸载功能单元 | [CLI 参考](docs/CLI_REFERENCE.md#feature) |
| `cell template list` | 模板列表 | [CLI 参考](docs/CLI_REFERENCE.md#template) |
| `cell template apply` | 应用模板 | [CLI 参考](docs/CLI_REFERENCE.md#template) |
| `cell template categories` | 模板分类 | [CLI 参考](docs/CLI_REFERENCE.md#template) |

### 📋 决策与审计

| 命令 | 说明 | 文档 |
|------|------|------|
| `cell decision list` | 决策记录列表 | [CLI 参考](docs/CLI_REFERENCE.md#decision) |
| `cell decision create` | 创建决策记录 | [CLI 参考](docs/CLI_REFERENCE.md#decision) |
| `cell decide make` | 自主决策 | [CLI 参考](docs/CLI_REFERENCE.md#decide) |
| `cell decide list` | 决策历史 | [CLI 参考](docs/CLI_REFERENCE.md#decide) |
| `cell audit log` | 审计日志 | [CLI 参考](docs/CLI_REFERENCE.md#audit) |
| `cell audit query` | 审计查询 | [CLI 参考](docs/CLI_REFERENCE.md#audit) |
| `cell audit trace` | 文件追溯 | [CLI 参考](docs/CLI_REFERENCE.md#audit) |

### 🧪 测试与基准

| 命令 | 说明 | 文档 |
|------|------|------|
| `cell test coverage` | 测试覆盖率 | [CLI 参考](docs/CLI_REFERENCE.md#test) |
| `cell bench list` | 基准测试列表 | [CLI 参考](docs/CLI_REFERENCE.md#bench) |
| `cell bench run` | 运行基准测试 | [CLI 参考](docs/CLI_REFERENCE.md#bench) |
| `cell bench compare` | 基准对比 | [CLI 参考](docs/CLI_REFERENCE.md#bench) |

### 🗃️ 项目与环境

| 命令 | 说明 | 文档 |
|------|------|------|
| `cell project list` | 项目列表 | [CLI 参考](docs/CLI_REFERENCE.md#project) |
| `cell project current` | 当前项目 | [CLI 参考](docs/CLI_REFERENCE.md#project) |
| `cell project switch` | 切换项目 | [CLI 参考](docs/CLI_REFERENCE.md#project) |
| `cell env list` | 环境列表 | [CLI 参考](docs/CLI_REFERENCE.md#env) |
| `cell env create` | 创建环境 | [CLI 参考](docs/CLI_REFERENCE.md#env) |
| `cell env get` | 获取配置 | [CLI 参考](docs/CLI_REFERENCE.md#env) |
| `cell env set` | 设置配置 | [CLI 参考](docs/CLI_REFERENCE.md#env) |
| `cell env diff` | 环境对比 | [CLI 参考](docs/CLI_REFERENCE.md#env) |
| `cell env drift` | 漂移检测 | [CLI 参考](docs/CLI_REFERENCE.md#env) |
| `cell db status` | 迁移状态 | [CLI 参考](docs/CLI_REFERENCE.md#db) |
| `cell db migrate` | 执行迁移 | [CLI 参考](docs/CLI_REFERENCE.md#db) |
| `cell db rollback` | 回滚迁移 | [CLI 参考](docs/CLI_REFERENCE.md#db) |

### 👥 多智能体协作

| 命令 | 说明 | 文档 |
|------|------|------|
| `cell agent list` | Agent 列表 | [CLI 参考](docs/CLI_REFERENCE.md#agent) |
| `cell agent register` | 注册 Agent | [CLI 参考](docs/CLI_REFERENCE.md#agent) |
| `cell agent delegate` | 委派任务 | [CLI 参考](docs/CLI_REFERENCE.md#agent) |
| `cell agent handoff` | 任务交接 | [CLI 参考](docs/CLI_REFERENCE.md#agent) |
| `cell agent-profile show` | 能力画像 | [CLI 参考](docs/CLI_REFERENCE.md#agent-profile) |
| `cell agent-profile rank` | Agent 排行 | [CLI 参考](docs/CLI_REFERENCE.md#agent-profile) |
| `cell tool-policy list` | 工具白名单 | [CLI 参考](docs/CLI_REFERENCE.md#tool-policy) |
| `cell self-heal status` | 自愈状态 | [CLI 参考](docs/CLI_REFERENCE.md#self-heal) |
| `cell self-heal detect` | 异常检测 | [CLI 参考](docs/CLI_REFERENCE.md#self-heal) |
| `cell self-heal recover` | 尝试恢复 | [CLI 参考](docs/CLI_REFERENCE.md#self-heal) |
| `cell workflow start` | 启动工作流 | [CLI 参考](docs/CLI_REFERENCE.md#workflow) |
| `cell workflow status` | 工作流状态 | [CLI 参考](docs/CLI_REFERENCE.md#workflow) |

### 📝 文档生成

| 命令 | 说明 | 文档 |
|------|------|------|
| `cell docs generate` | 生成全部文档 | [CLI 参考](docs/CLI_REFERENCE.md#docs) |
| `cell docs architecture` | 架构文档 | [CLI 参考](docs/CLI_REFERENCE.md#docs) |
| `cell docs api` | API 文档 | [CLI 参考](docs/CLI_REFERENCE.md#docs) |
| `cell docs decisions` | 决策文档 | [CLI 参考](docs/CLI_REFERENCE.md#docs) |

### 📊 可视化

| 命令 | 说明 | 文档 |
|------|------|------|
| `cell dashboard` | Web 仪表盘 | [CLI 参考](docs/CLI_REFERENCE.md#dashboard) |
| `cell ws serve` | WebSocket 服务 | [CLI 参考](docs/CLI_REFERENCE.md#ws) |
| `cell git status` | Git 状态 | [CLI 参考](docs/CLI_REFERENCE.md#git) |
| `cell git log` | Git 日志 | [CLI 参考](docs/CLI_REFERENCE.md#git) |
| `cell git diff` | Git 差异 | [CLI 参考](docs/CLI_REFERENCE.md#git) |

---

## 🎯 典型工作流

### 智能体开发一个新功能的完整流程

```
1. cell dev bootstrap    # 初始化环境（首次）
2. cell dev status       # 了解当前状态
3. cell task next        # 获取下一个任务
4. cell task claim <id>  # 认领任务
5. cell dev start <name> # 启动开发工作流
6. （开发中...）
7. cell arch lint        # 架构检查
8. cell arch lint --fix  # 自动修复
9. cell self-review      # 代码审查
10. cell verify          # 全面验证
11. cell task done <id>  # 完成任务
```

### 架构师日常工作流

```
1. cell arch overview    # 架构概览
2. cell entropy current  # 熵值现状
3. cell entropy trend    # 熵值趋势
4. cell arch advise      # 改进建议
5. cell arch visualize   # 架构可视化
6. cell review --deep    # 深度审查
7. cell docs architecture # 生成架构文档
```

---

## 🔧 项目结构

```
cell-architecture/
├── src/
│   ├── domain/              # 领域层 - 核心业务逻辑
│   │   ├── entropy.rs       # 熵值模型
│   │   ├── workflow.rs      # 工作流状态机
│   │   ├── cell_spec.rs     # Cell 规范
│   │   └── ...
│   ├── application/         # 应用层 - 服务编排
│   │   ├── entropy_service.rs
│   │   ├── arch_service.rs
│   │   ├── onboarding_service.rs
│   │   ├── dev_env_service.rs
│   │   └── ... (40+ 服务)
│   ├── adapters/            # 适配器层 - 外部交互
│   │   ├── file_*.rs
│   │   ├── template_engine.rs
│   │   └── ...
│   └── interfaces/          # 接口层 - CLI/API
│       ├── cli.rs
│       └── commands/        # 30+ 命令组
├── docs/                    # 文档
└── README.md
```

---

## 📈 当前状态

- **代码规模**: ~25,000 行 Rust
- **测试用例**: 123 个（全部通过）
- **服务模块**: 40+ 个
- **CLI 命令**: 30+ 个命令组
- **架构分层**: 完整的 Domain/Application/Adapters/Interfaces 四层

---
