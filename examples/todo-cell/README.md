# Todo Cell - 示例项目

这是一个最小可运行的 Cell Architecture 示例项目，展示如何使用 Cell CLI 管理一个待办事项服务。

## 项目结构

```
todo-cell/
├── cell.yaml          # Cell 项目配置
├── src/
│   ├── domain/        # 领域层
│   │   └── todo.rs
│   ├── application/   # 应用层
│   │   └── todo_service.rs
│   └── interfaces/    # 接口层
│       └── cli.rs
└── README.md
```

## 快速开始

```bash
# 1. 验证配置
cell config validate

# 2. 检查熵值
cell entropy current

# 3. 架构检查
cell arch lint

# 4. 功能管理
cell feature list
```

## Cell 配置说明

详见 [cell.yaml](cell.yaml)。

## 功能单元

| 功能 | 状态 | 说明 |
|------|------|------|
| todo-crud | Production | 待办事项 CRUD |
| todo-search | Staging | 待办搜索 |

## 扩展点

- **validators**: 验证链
- **notifiers**: 通知广播
- **filters**: 搜索过滤器
