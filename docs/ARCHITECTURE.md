# Cell Architecture

低熵架构，面向AI原生开发

## 架构分层

### Domain

领域层 - 核心业务逻辑，零外部依赖

**模块:**
- entropy.rs
- workflow.rs
- errors.rs

**规则:**
- 禁止依赖外部库
- 禁止依赖其他层

### Application

应用层 - 服务编排，调用领域层

**模块:**
- entropy_service.rs
- arch_service.rs

**规则:**
- 只能调用领域层
- 通过端口调用适配器

### Adapters

适配器层 - 实现端口接口

**模块:**
- file_adapter.rs
- git_adapter.rs

**规则:**
- 实现端口接口
- 可依赖外部库

### Interfaces

接口层 - CLI/API入口

**模块:**
- cli.rs
- commands/

**规则:**
- 调用应用层
- 不直接访问领域层

## 指标

- 熵值分数: 35.36
- 熵值等级: Notice
- 架构违规: 0 个

