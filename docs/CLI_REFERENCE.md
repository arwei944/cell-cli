# Cell CLI 命令参考手册

> 版本: v1.0
> 最后更新: 2026-06-26

---

## 全局选项

所有命令都支持以下全局选项：

| 选项 | 说明 |
|------|------|
| `-v, --verbose` | 详细输出（-v, -vv, -vvv） |
| `-f, --format <FORMAT>` | 输出格式: text, json, yaml |
| `-h, --help` | 显示帮助信息 |
| `-V, --version` | 显示版本 |

---

## 目录

- [🏗️ 架构核心类](#架构核心类)
- [📊 熵值与质量](#熵值与质量)
- [🤖 智能体开发工作流](#智能体开发工作流)
- [✅ 验证与门禁](#验证与门禁)
- [🧩 功能与模板](#功能与模板)
- [📋 决策与审计](#决策与审计)
- [🧪 测试与基准](#测试与基准)
- [🗃️ 项目与环境](#项目与环境)
- [👥 多智能体协作](#多智能体协作)
- [📝 文档生成](#文档生成)
- [📊 可视化与监控](#可视化与监控)

---

## 🏗️ 架构核心类

### init - 初始化项目

初始化一个新的 Cell Architecture 项目。

```bash
cell init [OPTIONS] <project-name>
```

**参数:**
- `project-name` - 项目名称

**选项:**
- `-t, --template <TEMPLATE>` - 使用的模板
- `--path <PATH>` - 项目路径（默认当前目录）

**示例:**
```bash
cell init my-project
cell init my-service --template crud-service
```

---

### generate - 代码生成

从规范生成 Cell、Port、Adapter 等代码。

```bash
cell generate <SUBCOMMAND>
```

**别名:** `g`, `gen`

**子命令:**

| 子命令 | 说明 |
|--------|------|
| `cell <spec-file>` | 生成 Cell 代码 |
| `port <spec-file>` | 生成 Port 接口 |
| `adapter <spec-file>` | 生成 Adapter 实现 |
| `all <spec-file>` | 生成全部代码 |

**示例:**
```bash
cell generate cell user_cell.spec.json
cell generate all user_cell.spec.json
```

---

### validate - 架构验证

验证架构规则是否符合规范。

```bash
cell validate [OPTIONS]
```

**别名:** `v`, `val`

**选项:**
- `-s, --strict` - 严格模式
- `--fix` - 自动修复可修复的问题

**示例:**
```bash
cell validate
cell validate --strict
```

---

### arch - 架构分析

架构分析与建议的主命令。

```bash
cell arch <SUBCOMMAND>
```

**别名:** `a`

**子命令:**

| 子命令 | 说明 |
|--------|------|
| `lint` | 架构规则检查 |
| `fix` | 自动修复架构违规 |
| `overview` | 架构概览 |
| `visualize` | 生成架构图（Mermaid/PlantUML） |
| `graph` | 依赖关系图 |
| `advise` | 架构改进建议 |
| `status` | 架构状态总览 |

#### arch lint

```bash
cell arch lint [OPTIONS]
```

**选项:**
- `--fix` - 自动修复
- `--deep` - 深度检查

#### arch visualize

```bash
cell arch visualize [OPTIONS]
```

**选项:**
- `-f, --format <FORMAT>` - 输出格式: mermaid, plantuml, ascii
- `-o, --output <FILE>` - 输出文件

**示例:**
```bash
cell arch lint
cell arch lint --fix
cell arch overview
cell arch visualize --format mermaid
cell arch advise
```

---

## 📊 熵值与质量

### entropy - 熵值计算

计算和管理架构熵值。

```bash
cell entropy <SUBCOMMAND>
```

**别名:** `e`, `ent`

**子命令:**

| 子命令 | 说明 |
|--------|------|
| `current` | 当前熵值 |
| `trend` | 熵值趋势（历史对比） |
| `baseline` | 熵值基线管理 |
| `compare` | 两次熵值对比 |
| `detail` | 详细熵值分析 |

#### entropy current

```bash
cell entropy current [OPTIONS]
```

**选项:**
- `--json` - JSON 格式输出

#### entropy trend

```bash
cell entropy trend [OPTIONS]
```

**选项:**
- `--days <DAYS>` - 查看最近 N 天
- `--json` - JSON 格式输出

#### entropy baseline

```bash
cell entropy baseline <SUBCOMMAND>
```

| 子命令 | 说明 |
|--------|------|
| `show` | 显示当前基线 |
| `set` | 设置基线 |
| `reset` | 重置基线 |

**示例:**
```bash
cell entropy current
cell entropy trend
cell entropy trend --days 30
cell entropy baseline show
```

---

### lint - 代码简洁度检查

检查代码简洁度和质量问题。

```bash
cell lint [OPTIONS]
```

**别名:** `sim`, `s`

**选项:**
- `--deep` - 深度检查
- `--fix` - 自动修复

**示例:**
```bash
cell lint
cell lint --deep
```

---

### review - 代码审查

自动化代码审查。

```bash
cell review [OPTIONS]
```

**选项:**
- `--deep` - 深度审查
- `--diff <COMMIT>` | 审查与某 commit 的差异
- `--output <FILE>` - 输出报告文件

**示例:**
```bash
cell review
cell review --deep
```

---

## 🤖 智能体开发工作流

### dev - 开发工作流

集成式开发工作流命令。

```bash
cell dev <SUBCOMMAND>
```

**别名:** `dw`, `dev-workflow`

**子命令:**

| 子命令 | 说明 |
|--------|------|
| `bootstrap` | 一键初始化开发环境 |
| `doctor` | 环境检查 |
| `start` | 启动开发任务 |
| `status` | 开发环境状态 |
| `next` | 下一步建议 |
| `context` | 生成上下文快照 |
| `reset` | 重置开发环境 |

#### dev bootstrap

一键完成所有准备工作：环境检测 → 工具链构建 → Git Hooks 安装 → Agent 注册 → 基线建立。

```bash
cell dev bootstrap
```

#### dev status

查看当前开发环境状态。

```bash
cell dev status
```

输出包含：
- Agent 身份信息
- 当前工作阶段
- 架构健康状态
- 熵值现状
- 待处理任务数

#### dev next

智能推荐下一步操作。

```bash
cell dev next
```

#### dev context

生成完整的上下文快照（给新 Agent 快速上手用）。

```bash
cell dev context
```

快照内容：
- 架构摘要
- 关键决策记录
- 当前任务
- 活跃特性
- 已知问题

#### dev reset

重置开发环境到干净状态。

```bash
cell dev reset [OPTIONS]
```

**选项:**
- `--scope <SCOPE>` - 重置范围: all, agent, progress

**示例:**
```bash
cell dev bootstrap
cell dev status
cell dev next
cell dev context
cell dev reset --scope agent
```

---

### task - 任务管理

任务发现与管理。

```bash
cell task <SUBCOMMAND>
```

**别名:** `t`, `tsk`

**子命令:**

| 子命令 | 说明 |
|--------|------|
| `list` | 任务列表 |
| `discover` | 发现任务（ROADMAP + TODO + Issues） |
| `next` | 推荐下一个任务 |
| `show <id>` | 任务详情 |
| `claim <id>` | 认领任务 |
| `done <id>` | 完成任务 |

**选项（全局）:**
- `-p, --priority <PRIORITY>` - 按优先级筛选: p0, p1, p2, p3
- `-s, --status <STATUS>` - 按状态筛选: pending, in_progress, done, blocked

**示例:**
```bash
cell task list
cell task list --priority p0
cell task discover
cell task next
cell task claim task-001
cell task done task-001
```

---

## ✅ 验证与门禁

### verify - 快速验证

快速验证（编译 + 测试 + 架构规则）。

```bash
cell verify [OPTIONS]
```

**别名:** `vfy`

**选项:**
- `--skip-tests` - 跳过测试
- `--skip-arch` - 跳过架构检查
- `--fix` - 自动修复

**示例:**
```bash
cell verify
cell verify --fix
```

---

### self-verify - 自我验证与修复

完整的自我验证与自动修复循环。

```bash
cell self-verify [OPTIONS]
```

**别名:** `sv`, `self-check`

**选项:**
- `--max-attempts <N>` - 最大重试次数（默认 3）
- `--no-fix` - 不自动修复
- `--rollback-on-fail` - 失败时自动回滚

**验证项:**
1. 架构检查 + 自动修复
2. 测试运行
3. 熵值退化检测
4. （可选）编译错误自动修复

**示例:**
```bash
cell self-verify
cell self-verify --max-attempts 5
cell self-verify --rollback-on-fail
```

---

### self-review - 自我代码审查

自我代码审查与自动修复。

```bash
cell self-review [OPTIONS]
```

**子命令:**

| 子命令 | 说明 |
|--------|------|
| `self` | 自我审查 |
| `cross-create` | 创建交叉审查 |
| `cross-list` | 交叉审查列表 |
| `cross-approve <id>` | 批准交叉审查 |

**选项:**
- `--deep` - 深度审查
- `--auto-fix` - 自动修复可修复的问题

**示例:**
```bash
cell self-review
cell self-review --deep --auto-fix
cell self-review cross-create --assignee agent-002
```

---

### enforcement - 强制门禁系统

架构强制约束管理。

```bash
cell enforcement <SUBCOMMAND>
```

**别名:** `ef`, `guard`, `enforce`

**子命令:**

| 子命令 | 说明 |
|--------|------|
| `status` | 门禁状态 |
| `enable` | 启用门禁 |
| `disable` | 禁用门禁 |
| `install-hooks` | 安装 Git Hooks |
| `uninstall-hooks` | 卸载 Git Hooks |
| `policy show` | 显示策略配置 |
| `policy set <key> <value>` | 设置策略 |

**策略级别:**
- `allow` - 允许
- `warn` - 警告
- `block` - 阻断

**可配置策略:**
- `entropy_degradation` - 熵值退化
- `architecture_violations` - 架构违规
- `test_failure` - 测试失败
- `naming_violations` - 命名违规
- `circular_dependency` - 循环依赖
- `untracked_decisions` - 未追踪决策

**示例:**
```bash
cell enforcement status
cell enforcement install-hooks
cell enforcement policy show
cell enforcement policy set entropy_degradation block
```

---

## 🧩 功能与模板

### feature - 功能单元管理

功能单元的生命周期管理。

```bash
cell feature <SUBCOMMAND>
```

**别名:** `f`, `feat`

**子命令:**

| 子命令 | 说明 |
|--------|------|
| `list` | 功能单元列表 |
| `new <name>` | 新建功能单元 |
| `mount <name>` | 挂载功能单元 |
| `unmount <name>` | 卸载功能单元 |
| `impact <name>` | 影响分析 |
| `show <name>` | 功能详情 |

**示例:**
```bash
cell feature list
cell feature new user-authentication
cell feature mount user-authentication
cell feature impact user-authentication
```

---

### template - 模板库

最佳实践模板与脚手架。

```bash
cell template <SUBCOMMAND>
```

**别名:** `tpl`, `temp`

**子命令:**

| 子命令 | 说明 |
|--------|------|
| `list` | 模板列表 |
| `categories` | 模板分类 |
| `show <name>` | 模板详情 |
| `apply <name>` | 应用模板 |
| `search <keyword>` | 搜索模板 |

**内置模板分类:**
- 🏗️ 架构模式（CRUD Service、Microservice、CLI Tool 等）
- 📦 项目模板（完整项目脚手架）
- 🔌 适配器模板（Database、API、Message Queue 等）

**示例:**
```bash
cell template list
cell template categories
cell template show crud-service
cell template apply crud-service
```

---

## 📋 决策与审计

### decision - 决策记录管理

ADR（架构决策记录）管理。

```bash
cell decision <SUBCOMMAND>
```

**别名:** `dec`, `d`

**子命令:**

| 子命令 | 说明 |
|--------|------|
| `list` | 决策列表 |
| `create` | 创建决策记录 |
| `show <id>` | 决策详情 |
| `update <id>` | 更新决策状态 |
| `export` | 导出决策文档 |

**示例:**
```bash
cell decision list
cell decision create --title "采用 PostgreSQL" --context "..."
cell decision show dec-001
```

---

### decide - 自主决策引擎

基于规则的自主决策。

```bash
cell decide <SUBCOMMAND>
```

**别名:** `auto-decide`

**子命令:**

| 子命令 | 说明 |
|--------|------|
| `make` | 做出决策 |
| `list` | 决策历史 |
| `rules` | 决策规则列表 |
| `pending` | 待确认决策 |
| `approve <id>` | 批准待确认决策 |
| `reject <id>` | 拒绝待确认决策 |

**示例:**
```bash
cell decide make --title "数据库选型" --context "..."
cell decide list
cell decide pending
```

---

### audit - 操作审计

操作审计与追溯。

```bash
cell audit <SUBCOMMAND>
```

**别名:** `aud`, `log`

**子命令:**

| 子命令 | 说明 |
|--------|------|
| `log` | 审计日志 |
| `query` | 按条件查询 |
| `trace <file>` | 追溯文件变更历史 |
| `stats` | 审计统计 |

**查询选项:**
- `--agent <AGENT_ID>` - 按 Agent 筛选
- `--action <ACTION>` - 按操作类型筛选
- `--from <DATE>` - 起始时间
- `--to <DATE>` - 结束时间
- `--result <RESULT>` - 按结果筛选: success, failed

**示例:**
```bash
cell audit log
cell audit query --agent agent-001
cell audit trace src/application/arch_service.rs
cell audit stats
```

---

## 🧪 测试与基准

### test - 测试覆盖率

测试覆盖率分析。

```bash
cell test <SUBCOMMAND>
```

**别名:** `tst`, `cov`

**子命令:**

| 子命令 | 说明 |
|--------|------|
| `coverage` | 测试覆盖率 |
| `summary` | 测试摘要 |
| `trend` | 覆盖率趋势 |

**示例:**
```bash
cell test coverage
cell test summary
```

---

### bench - 性能基准测试

性能基准测试框架。

```bash
cell bench <SUBCOMMAND>
```

**别名:** `benchmark`

**子命令:**

| 子命令 | 说明 |
|--------|------|
| `list` | 基准测试列表 |
| `run` | 运行基准测试 |
| `compare` | 基准对比 |
| `history` | 历史记录 |
| `report` | 生成报告 |

**示例:**
```bash
cell bench list
cell bench run
cell bench compare baseline current
```

---

## 🗃️ 项目与环境

### project - 多项目管理

多项目 / Monorepo 支持。

```bash
cell project <SUBCOMMAND>
```

**别名:** `proj`

**子命令:**

| 子命令 | 说明 |
|--------|------|
| `list` | 项目列表 |
| `current` | 当前项目 |
| `switch <name>` | 切换项目 |
| `add <path>` | 添加项目 |
| `remove <name>` | 移除项目 |
| `cross-report` | 跨项目报告 |

**示例:**
```bash
cell project list
cell project current
cell project switch my-project
cell project add ../other-project
```

---

### env - 多环境配置

多环境配置管理。

```bash
cell env <SUBCOMMAND>
```

**别名:** `environments`

**子命令:**

| 子命令 | 说明 |
|--------|------|
| `list` | 环境列表 |
| `current` | 当前环境 |
| `create <name>` | 创建环境 |
| `switch <name>` | 切换环境 |
| `get <key>` | 获取配置项 |
| `set <key> <value>` | 设置配置项 |
| `diff <env1> <env2>` | 环境对比 |
| `drift` | 漂移检测 |
| `sync <source>` | 从源环境同步 |
| `delete <name>` | 删除环境 |

**示例:**
```bash
cell env list
cell env create staging
cell env switch development
cell env get database.url
cell env set database.url "postgres://localhost:5432/db"
cell env diff development staging
cell env drift
```

---

### db - 数据库迁移

数据库迁移管理。

```bash
cell db <SUBCOMMAND>
```

**别名:** `migrate`, `migration`

**子命令:**

| 子命令 | 说明 |
|--------|------|
| `status` | 迁移状态 |
| `list` | 迁移列表 |
| `create <name>` | 创建迁移 |
| `migrate` | 执行待运行的迁移 |
| `rollback` | 回滚上一个迁移 |
| `validate` | 验证迁移 |
| `drift` | 漂移检测 |
| `history` | 迁移历史 |

**示例:**
```bash
cell db status
cell db create add-users-table
cell db migrate
cell db rollback
cell db validate
```

---

## 👥 多智能体协作

### agent - 智能体协议

多智能体协作协议。

```bash
cell agent <SUBCOMMAND>
```

**别名:** `ag`

**子命令:**

| 子命令 | 说明 |
|--------|------|
| `list` | Agent 列表 |
| `register` | 注册 Agent |
| `status <id>` | Agent 状态 |
| `delegate` | 委派任务 |
| `handoff` | 任务交接 |
| `message` | 发送消息 |
| `inbox` | 收件箱 |

**示例:**
```bash
cell agent list
cell agent register --name "Code Reviewer" --role reviewer
cell agent delegate --agent reviewer-01 --task task-001
cell agent handoff --to agent-002 --task task-001
```

---

### agent-profile - 能力画像

Agent 能力画像与排行榜。

```bash
cell agent-profile <SUBCOMMAND>
```

**别名:** `ap`, `profile`

**子命令:**

| 子命令 | 说明 |
|--------|------|
| `show <agent-id>` | 能力画像详情 |
| `list` | 全部画像列表 |
| `rank` | Agent 排行榜 |
| `record` | 记录任务完成 |

**画像维度:**
- 代码质量分
- 熵值影响
- Bug 率
- 审查通过率
- 任务按时完成率

**示例:**
```bash
cell agent-profile show agent-001
cell agent-profile rank
cell agent-profile record --agent agent-001 --task task-001 --success
```

---

### tool-policy - 工具白名单

工具/MCP 白名单与权限管理。

```bash
cell tool-policy <SUBCOMMAND>
```

**别名:** `tp`

**子命令:**

| 子命令 | 说明 |
|--------|------|
| `list` | 工具列表 |
| `show <tool-id>` | 工具详情 |
| `check <tool-id>` | 检查权限 |
| `enable <tool-id>` | 启用工具 |
| `disable <tool-id>` | 禁用工具 |
| `usage` | 使用统计 |
| `audit` | 工具调用审计 |

**风险等级:**
- `low` - 低风险
- `medium` - 中风险
- `high` - 高风险

**Agent 角色:**
- `Architect` - 架构师（全部权限）
- `Developer` - 开发者（标准权限）
- `Tester` - 测试员（只读+测试）

**示例:**
```bash
cell tool-policy list
cell tool-policy show arch-lint
cell tool-policy check arch-lint --role developer
cell tool-policy usage
```

---

### self-heal - 异常自愈

异常检测与自动恢复。

```bash
cell self-heal <SUBCOMMAND>
```

**别名:** `heal`, `sh`

**子命令:**

| 子命令 | 说明 |
|--------|------|
| `status` | 自愈系统状态 |
| `detect` | 检测异常 |
| `recover <anomaly-id>` | 尝试恢复 |
| `report` | 自愈报告 |
| `escalate` | 生成人工介入报告 |
| `history` | 历史记录 |

**异常严重程度:**
- `info` - 信息
- `warning` - 警告
- `critical` - 严重
- `fatal` - 致命

**恢复方式:**
- `retry` - 重试
- `rollback` - 回滚
- `escalate` - 上报人工

**示例:**
```bash
cell self-heal status
cell self-heal detect
cell self-heal recover anomaly-001
cell self-heal report
cell self-heal escalate
```

---

### workflow - 工作流引擎

智能体无关的工作流协议。

```bash
cell workflow <SUBCOMMAND>
```

**别名:** `wf`

**子命令:**

| 子命令 | 说明 |
|--------|------|
| `start` | 启动工作流 |
| `status` | 工作流状态 |
| `gates` | 门禁状态 |
| `advance` | 推进到下一阶段 |
| `complete` | 完成工作流 |
| `abort` | 中止工作流 |
| `handoff` | 工作流交接 |

**工作流阶段:**
- `Idle` - 空闲
- `Design` - 设计阶段
- `Implement` - 实现阶段
- `Verify` - 验证阶段
- `Handoff` - 交接阶段
- `Complete` - 完成

**示例:**
```bash
cell workflow start --task "新功能开发" --agent agent-001
cell workflow status
cell workflow gates
cell workflow advance
cell workflow complete
```

---

## 📝 文档生成

### docs - 文档自动生成

自动生成各类文档。

```bash
cell docs <SUBCOMMAND>
```

**别名:** `doc`

**子命令:**

| 子命令 | 说明 |
|--------|------|
| `generate` | 生成全部文档 |
| `architecture` | 架构文档 |
| `api` | API 文档 |
| `decisions` | 决策文档集 |
| `readme` | README 文档 |
| `serve` | 启动文档服务器 |

**示例:**
```bash
cell docs generate
cell docs architecture
cell docs api
cell docs decisions
```

---

## 📊 可视化与监控

### dashboard - Web 仪表盘

启动 Web 仪表盘。

```bash
cell dashboard [OPTIONS]
```

**别名:** `dash`, `web`

**选项:**
- `--port <PORT>` - 端口号（默认 8080）
- `--host <HOST>` - 主机地址（默认 localhost）
- `--open` - 自动打开浏览器

**示例:**
```bash
cell dashboard
cell dashboard --port 3000 --open
```

---

### ws - WebSocket 仪表盘

WebSocket 实时更新仪表盘。

```bash
cell ws <SUBCOMMAND>
```

**别名:** `wsd`, `websocket`

**子命令:**

| 子命令 | 说明 |
|--------|------|
| `serve` | 启动 WebSocket 服务 |
| `html` | 生成 HTML 仪表盘 |
| `test` | 测试 WebSocket 连接 |

**示例:**
```bash
cell ws serve
cell ws html --output dashboard.html
cell ws test
```

---

### git - Git 集成

Git 集成与增强命令。

```bash
cell git <SUBCOMMAND>
```

**别名:** `gitops`

**子命令:**

| 子命令 | 说明 |
|--------|------|
| `status` | Git 状态（增强版） |
| `branches` | 分支列表 |
| `log` | 提交日志（增强版） |
| `diff` | 差异分析 |
| `hooks` | Git Hooks 管理 |
| `blame <file>` | 代码归属分析 |

**示例:**
```bash
cell git status
cell git log
cell git diff
cell git hooks status
cell git blame src/main.rs
```

---

*文档结束*
