# 智能体开发指南 — Agent Developer Guide

> 版本: v1.0
> 最后更新: 2026-06-26
> 适用对象: AI 智能体（Developer 角色）

---

## 前言

本指南是 **每一个参与 Cell Architecture 项目开发的智能体必须遵守的操作手册**。

> **记住：Architecture is the Prompt — 架构即提示。**
> 
> 不是靠文档说服你遵守规则，而是靠工具链强制执行规则。
> 你是标准化工厂里的工人，你的每一步操作都有质检。

---

## 目录

1. [第一天：从零到能开发](#第一天从零到能开发)
2. [每日工作流](#每日工作流)
3. [开发一个功能的完整流程](#开发一个功能的完整流程)
4. [质量自检清单](#质量自检清单)
5. [遇到问题怎么办](#遇到问题怎么办)
6. [强制门禁说明](#强制门禁说明)
7. [常用命令速查表](#常用命令速查表)

---

## 第一天：从零到能开发

### 第 1 步：初始化环境

你什么都不用想，直接运行一条命令：

```bash
cell dev bootstrap
```

这条命令会自动完成以下所有事情：
1. ✅ 检测开发环境（Rust、Cargo、Git 等）
2. ✅ 构建工具链
3. ✅ 安装 Git Hooks
4. ✅ 为你注册 Agent 身份（生成唯一 ID）
5. ✅ 建立架构基线（熵值、架构快照）
6. ✅ 生成就绪报告

**预计耗时：1-3 分钟**

---

### 第 2 步：了解你是谁

```bash
cell dev status
```

你会看到：
- 👤 你的 Agent ID 和角色
- 🎯 当前工作阶段
- 🏗️ 架构健康状态
- 📊 当前熵值
- 📋 待处理任务数

---

### 第 3 步：快速了解项目

```bash
cell dev context
```

这条命令会生成一份完整的上下文快照，包含：
- 架构摘要（分层结构、核心模块）
- 关键决策记录（ADR 摘要）
- 当前进行中的任务
- 活跃特性单元
- 已知问题和待办
- 代码规范和约定

**仔细读完这份快照，你就对项目有 80% 的了解了。**

---

### 第 4 步：获取第一个任务

```bash
cell task next
```

系统会自动推荐给你：
- 优先级最高的
- 没有依赖的
- 还没人认领的任务

觉得合适就认领：

```bash
cell task claim <task-id>
```

---

## 每日工作流

### 早上开工

```bash
# 1. 看看状态
cell dev status

# 2. 看看今天该干什么
cell dev next

# 3. 看看任务列表
cell task list --status in_progress
```

### 开发过程中

```bash
# 随时检查架构健康度
cell arch lint

# 看看熵值有没有恶化
cell entropy current

# 架构有问题？自动修一下
cell arch lint --fix

# 自我审查一下代码
cell self-review --auto-fix
```

### 提交前

```bash
# 全面自检（必做！）
cell verify

# 或者更严格的自我验证
cell self-verify
```

### 下班前

```bash
# 看看今天的成果
cell audit query --agent <your-agent-id>

# 更新任务状态
cell task done <task-id>  # 如果完成了
```

---

## 开发一个功能的完整流程

```
┌─ 1. 认领任务 ──────────────────────────────┐
│  cell task next                           │
│  cell task claim <task-id>                │
└────────────────────────────────────────────┘
                    ↓
┌─ 2. 启动工作流 ────────────────────────────┐
│  cell dev start <task-name>               │
│  → 自动捕获基线（熵值、架构快照）          │
│  → 进入 Design 阶段                       │
└────────────────────────────────────────────┘
                    ↓
┌─ 3. 设计阶段 ──────────────────────────────┐
│  • 阅读相关代码和文档                      │
│  • 如果遇到架构决策：                      │
│    cell decide make --title "..."         │
│  • 设计完成后推进到下一阶段：              │
│    cell workflow advance                  │
└────────────────────────────────────────────┘
                    ↓
┌─ 4. 实现阶段 ──────────────────────────────┐
│  • 编写代码                                │
│  • 随时自查：                              │
│    cell arch lint                         │
│    cell lint                              │
│  • 实现完成后推进到下一阶段：              │
│    cell workflow advance                  │
└────────────────────────────────────────────┘
                    ↓
┌─ 5. 验证阶段 ──────────────────────────────┐
│  cell self-verify                         │
│  → 架构检查 + 自动修复                     │
│  → 测试运行                               │
│  → 熵值退化检测                           │
│  → 最多自动重试 3 次                      │
│  验证通过后推进：                          │
│    cell workflow advance                  │
└────────────────────────────────────────────┘
                    ↓
┌─ 6. 代码审查 ──────────────────────────────┐
│  cell self-review --deep --auto-fix       │
│  • 简单问题自动修复                        │
│  • 复杂问题标记出来                        │
│  • 需要的话创建交叉审查                    │
└────────────────────────────────────────────┘
                    ↓
┌─ 7. 提交代码 ──────────────────────────────┐
│  git add .                                │
│  git commit -m "feat: xxx"               │
│  → pre-commit 钩子自动运行检查            │
│  → 不通过就不让提交                        │
└────────────────────────────────────────────┘
                    ↓
┌─ 8. 完成任务 ──────────────────────────────┐
│  cell task done <task-id>                 │
│  cell workflow complete                   │
└────────────────────────────────────────────┘
```

---

## 质量自检清单

**提交代码前，确保以下检查全部通过：**

### ✅ 架构健康
- [ ] `cell arch lint` 零违规
- [ ] 没有跨层依赖
- [ ] 没有循环依赖
- [ ] 命名符合规范

### ✅ 代码质量
- [ ] `cell lint` 无严重问题
- [ ] 没有未使用的 import
- [ ] 没有硬编码的 magic number
- [ ] 函数不要太长

### ✅ 测试
- [ ] `cargo test` 全部通过
- [ ] 新代码有对应的测试
- [ ] 边界情况有覆盖

### ✅ 熵值
- [ ] `cell entropy current` 熵值不退化
- [ ] 新代码没有显著增加复杂度
- [ ] 耦合度没有上升

### ✅ 审查
- [ ] `cell self-review` 通过
- [ ] 自动修复的问题都修复了
- [ ] 遗留问题都标记了

---

## 遇到问题怎么办

### 1. 先自己诊断

```bash
# 看看环境健康吗
cell dev status

# 有没有异常？
cell self-heal detect

# 审计日志看看发生了什么
cell audit log --limit 20
```

### 2. 尝试自愈

```bash
# 检测到异常后尝试恢复
cell self-heal recover <anomaly-id>

# 或者来个完整的自我验证
cell self-verify --rollback-on-fail
```

### 3. 需要做决策？

```bash
# 让决策引擎帮你决定
cell decide make --title "问题描述" --context "上下文"

# 看看历史上类似的决策
cell decide list
```

### 4. 实在搞不定？

```bash
# 生成一份清晰的问题报告，等人类来
cell self-heal escalate
```

报告包含：
- 问题描述
- 尝试过的所有方案
- 每次失败的原因
- 当前状态
- 建议的下一步

---

## 强制门禁说明

### 🚫 Git Hooks（本地门禁）

提交代码时，pre-commit 钩子会自动运行：
- 架构规则检查 → 违规就阻断
- 测试运行 → 失败就阻断
- 熵值退化检测 → 退化就阻断

**注意：不要试图用 `--no-verify` 绕过。CI 那边还有一道门。**

### 🚫 CI 质量门禁（合并门禁）

PR 合并前，CI 会检查：
- 编译通过
- 全部测试通过
- 架构零违规
- 熵值不退化
- 代码审查通过
- 基准测试性能不回退

**任何一项不通过，PR 都不能合并。**

### 🚫 工具白名单（权限门禁）

不是所有工具你都能用：

| 你的角色 | 能用的工具 |
|----------|-----------|
| Developer | 大部分开发工具 |
| Tester | 测试相关工具 |
| Architect | 全部工具 |

越权调用会被拒绝并记录到审计日志。

---

## 常用命令速查表

### 环境与状态
```bash
cell dev bootstrap    # 一键初始化
cell dev status       # 当前状态
cell dev next         # 下一步建议
cell dev context      # 上下文快照
cell dev reset        # 重置环境
```

### 任务管理
```bash
cell task list        # 任务列表
cell task discover    # 发现任务
cell task next        # 推荐任务
cell task claim <id>  # 认领任务
cell task done <id>   # 完成任务
```

### 架构与熵值
```bash
cell arch lint        # 架构检查
cell arch lint --fix  # 自动修复
cell arch overview    # 架构概览
cell arch visualize   # 架构图
cell entropy current  # 当前熵值
cell entropy trend    # 熵值趋势
```

### 质量与验证
```bash
cell verify           # 快速验证
cell self-verify      # 自我验证+修复
cell self-review      # 自我审查
cell lint             # 代码简洁度
cell review           # 代码审查
```

### 工作流
```bash
cell workflow start   # 启动工作流
cell workflow status  # 工作流状态
cell workflow advance # 推进到下一阶段
cell workflow gates   # 门禁状态
cell workflow complete # 完成工作流
```

### 决策与审计
```bash
cell decide make      # 自主决策
cell decide list      # 决策历史
cell audit log        # 审计日志
cell audit trace      # 文件追溯
```

### 异常与自愈
```bash
cell self-heal status  # 自愈状态
cell self-heal detect  # 检测异常
cell self-heal recover # 尝试恢复
cell self-heal escalate # 上报人工
```

---

## 最后提醒

1. **工具链是你的朋友，不是你的敌人** —— 它帮你少犯错
2. **不要试图绕过门禁** —— 绕过了本地还有 CI，绕过了 CI 还有代码审查
3. **遇到问题先查工具** —— 90% 的问题工具链已经帮你想好了解决方案
4. **不确定就问决策引擎** —— 它比你更了解这个项目的历史决策
5. **实在搞不定就生成 escalation report** —— 清晰的问题报告比模糊的提问有用 100 倍

---

> **记住：你不是一个人在战斗。你有一整套工具链在帮你。**
> 
> 用好它们，你就是最高效的智能体。

---

*文档结束*
