# 架构师指南 — Architect Guide

> 版本: v1.0
> 最后更新: 2026-06-26
> 适用对象: 架构师（Architect 角色）

---

## 前言

作为架构师，你是这个项目的"总设计师"。你的职责不是写每一行代码，而是：
- 保持架构的低熵状态
- 做出正确的技术决策
- 确保团队沿着正确的方向演进
- 培养其他智能体提供架构指导

> **记住：熵值是你的仪表盘，架构是你的作品。**

---

## 目录

1. [架构师的日常](#架构师的日常

- 2. [架构监控与诊断](#架构监控与诊断)
3. [架构决策](#架构决策)
4. [架构治理](#架构治理)
5. [团队管理](#团队管理)
6. [架构演进](#架构演进)
7. [常用命令速查](#常用命令速查)

---

## 架构师的日常

### 早上：全面体检

```bash
# 1. 架构总览
cell arch overview

# 2. 熵值现状
cell entropy current

# 3. 熵值趋势（最近一周）
cell entropy trend --days 7

# 4. 待处理任务概览
cell task list --priority p0
```

**你需要关注：
- 架构违规有没有增加？
- 熵值有没有恶化？
- P0 任务有没有人在做？

---

### 中午：深度审查

```bash
# 深度代码审查
cell review --deep

# 看看最近的决策
cell decision list --limit 10

# 看看Agent表现
cell agent-profile rank
```

---

### 下午：架构改进

```bash
# 获取架构改进建议
cell arch advise

# 生成架构文档
cell docs architecture

# 看看基准测试
cell bench compare baseline current
```

---

## 架构监控与诊断

### 架构健康检查

```bash
# 快速概览
cell arch overview

# 详细检查
cell arch lint --deep

# 架构可视化
cell arch visualize --format mermaid

# 依赖图
cell arch graph
```

---

### 熵值监控

```bash
# 当前熵值
cell entropy current

# 熵值趋势
cell entropy trend --days 30

# 对比基线
cell entropy compare baseline current

# 详细分析
cell entropy detail
```

#### 五维熵值说明：

| 熵值类型 | 说明 | 正常范围 | 危险信号 |
|---------|------|---------|----------|
| 结构熵 | 分层结构有序度 | 低 | 持续上升 |
| 复杂度熵 | 代码复杂度 | 低 | 单点复杂度过高 |
| 耦合熵 | 模块间耦合度 | 低 | 循环依赖 |
| 命名熵 | 命名规范度 | 低 | 命名混乱 |
| 测试熵 | 测试覆盖度 | 低 | 测试覆盖下降 |

---

### 问题定位

当你发现架构有问题时，按这个顺序排查：

```bash
# 1. 先看概览，哪里有问题
cell arch overview

# 2. 详细看看违规
cell arch lint --deep

# 3. 看看依赖关系
cell arch graph

# 4. 看看熵值分析
cell entropy detail

# 5. 追溯问题文件
cell audit trace <file>

# 6. 看看是谁引入的问题
cell git blame <file>
```

---

## 架构决策

### 何时需要做决策

遇到以下情况，你需要做出架构决策：

1. 新技术选型（数据库、框架、库的选择
2. 重要架构变更
3. 重大重构方案
4. 技术债务处理
5. API 设计

---

### 如何做决策

```bash
# 让决策引擎先分析一下
cell decide make --title "决策标题" --context "详细上下文"

# 看看历史上类似决策
cell decide list

# 待确认的决策
cell decide pending
```

---

### 决策记录

所有重要决策一定要记录到 ADR：

```bash
# 创建决策记录
cell decision create \
  --title "采用 PostgreSQL 作为主数据库" \
  --context "..." \
  --status accepted \
  --tags "database, infrastructure"

# 列出所有决策
cell decision list

# 查看某个决策详情
cell decision show <decision-id>
```

---

## 架构治理

### 强制门禁配置

作为架构师，你有权配置门禁的严格程度：

```bash
# 查看当前策略
cell enforcement policy show

# 设置熵值退化为阻断级别
cell enforcement policy set entropy_degradation block

# 设置架构违规为阻断级别
cell enforcement policy set architecture_violations block

# 设置测试失败为警告
cell enforcement policy set test_failure warn
```

#### 策略级别说明：

| 级别 | 说明 | 适用场景 |
|------|------|---------|
| `allow` | 允许，不做任何限制 | 实验性功能 |
| `warn` | 警告，记录日志但不阻断 | 非关键质量 |
| `block` | 阻断，不通过就不能提交/合并 | 核心架构规则 |

---

### Git Hooks 管理

```bash
# 安装 Git Hooks
cell enforcement install-hooks

# 卸载 Git Hooks
cell enforcement uninstall-hooks

# 启用门禁状态
cell enforcement status
```

---

### 工具权限管理

```bash
# 查看所有工具
cell tool-policy list

# 查看某个工具详情
cell tool-policy show <tool-id>

# 检查某个角色的权限
cell tool-policy check <tool-id> --role developer

# 启用/禁用工具
cell tool-policy enable <tool-id>
cell tool-policy disable <tool-id>

# 工具使用统计
cell tool-policy usage
```

---

## 团队管理

### Agent 管理

```bash
# 查看所有 Agent
cell agent list

# 查看某个 Agent 状态
cell agent status <agent-id>

# 注册新 Agent
cell agent register --name "名字" --role developer

# 能力画像
cell agent-profile show <agent-id>

# Agent 排行榜
cell agent-profile rank
```

---

### 任务分配

```bash
# 任务列表
cell task list

# 委派任务给某个 Agent
cell agent delegate --agent <agent-id> --task <task-id>

# 任务交接
cell agent handoff --from <agent-id> --to <agent-id> --task <task-id>
```

---

### 代码审查

```bash
# 自我审查
cell self-review

# 创建交叉审查
cell self-review cross-create --assignee <agent-id>

# 交叉审查列表
cell self-review cross-list

# 批准审查
cell self-review cross-approve <review-id>
```

---

## 架构演进

### 功能单元管理

```bash
# 功能单元列表
cell feature list

# 新建功能单元
cell feature new <feature-name>

# 挂载功能单元
cell feature mount <feature-name>

# 卸载功能单元
cell feature unmount <feature-name>

# 影响分析
cell feature impact <feature-name>
```

---

### 模板与最佳实践

```bash
# 模板列表
cell template list

# 模板分类
cell template categories

# 应用模板详情
cell template show <template-name>

# 应用模板
cell template apply <template-name>
```

---

### 文档生成

```bash
# 生成全部文档
cell docs generate

# 架构文档
cell docs architecture

# API 文档
cell docs api

# 决策文档集
cell docs decisions
```

---

### 性能监控

```bash
# 启动 Web 仪表盘
cell dashboard --port 8080

# WebSocket 实时仪表盘
cell ws serve

# 基准测试
cell bench run

# 基准对比
cell bench compare baseline current
```

---

## 常用命令速查

### 架构监控
```bash
cell arch overview      # 架构概览
cell arch lint          # 架构检查
cell arch lint --fix     # 自动修复
cell arch visualize     # 架构图
cell arch advise        # 架构建议
cell arch graph        # 依赖图
```

### 熵值管理
```bash
cell entropy current    # 当前熵值
cell entropy trend      # 熵值趋势
cell entropy baseline   # 基线管理
cell entropy detail     # 详细分析
```

### 决策管理
```bash
cell decide make        # 自主决策
cell decision list      # 决策列表
cell decision create    # 创建决策
cell decision show      # 决策详情
```

### 团队管理
```bash
cell enforcement status  # 门禁状态
cell enforcement policy set # 设置策略
cell agent list        # Agent 列表
cell agent-profile rank # Agent 排行
```

### 质量与审查
```bash
cell review --deep       # 深度审查
cell self-review          # 自我审查
cell audit log            # 审计日志
cell audit trace        # 文件追溯
```

---

## 架构师的 checklist

### 每日 checklist
- [ ] 架构概览
- [ ] 熵值现状
- [ ] P0 任务状态
- [ ] 待确认决策

### 每周 checklist
- [ ] 熵值趋势分析
- [ ] 架构债务评估
- [ ] Agent 表现评估
- [ ] 团队进度回顾
- [ ] 下周计划调整

### 每月 checklist
- [ ] 架构健康度评估
- [ ] 技术债务盘点
- [ ] 重大决策复盘
- [ ] 路线图更新
- [ ] 最佳实践沉淀

---

> **记住：好的架构师不是设计出来的，而是演化出来的。
> 
> 你的工作是引导演化方向，而不是控制每一步。

---

*文档结束*
