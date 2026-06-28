use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FingerprintCategory {
    Architecture,
    Performance,
    Security,
    Maintainability,
    Testing,
    Dependency,
    Configuration,
}

impl FingerprintCategory {
    pub fn description(&self) -> &'static str {
        match self {
            Self::Architecture => "架构问题",
            Self::Performance => "性能问题",
            Self::Security => "安全问题",
            Self::Maintainability => "可维护性问题",
            Self::Testing => "测试问题",
            Self::Dependency => "依赖问题",
            Self::Configuration => "配置问题",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FingerprintSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl FingerprintSeverity {
    pub fn description(&self) -> &'static str {
        match self {
            Self::Critical => "严重",
            Self::High => "高",
            Self::Medium => "中",
            Self::Low => "低",
            Self::Info => "信息",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProblemFingerprint {
    pub id: String,
    pub name: String,
    pub category: FingerprintCategory,
    pub severity: FingerprintSeverity,
    pub patterns: Vec<String>,
    pub error_patterns: Vec<String>,
    pub symptoms: Vec<String>,
    pub root_cause: String,
    pub fix_suggestion: String,
    pub auto_fixable: bool,
}

impl ProblemFingerprint {
    pub fn matches_code(&self, content: &str) -> bool {
        self.patterns.iter().any(|p| content.contains(p))
    }

    pub fn matches_error(&self, error_msg: &str) -> bool {
        let normalized = Self::normalize_error(error_msg);
        self.error_patterns.iter().any(|p| {
            let normalized_pattern = Self::normalize_error(p);
            normalized.contains(&normalized_pattern)
        })
    }

    fn normalize_error(s: &str) -> String {
        s.to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FingerprintMatch {
    pub fingerprint: ProblemFingerprint,
    pub confidence: f64,
    pub matched_patterns: Vec<String>,
    pub matched_lines: Vec<usize>,
}

pub struct FingerprintLibrary {
    fingerprints: Vec<ProblemFingerprint>,
    custom_fingerprints: HashMap<String, ProblemFingerprint>,
}

impl FingerprintLibrary {
    pub fn new() -> Self {
        let mut lib = Self {
            fingerprints: Vec::new(),
            custom_fingerprints: HashMap::new(),
        };
        lib.register_builtin_fingerprints();
        lib
    }

    fn register_builtin_fingerprints(&mut self) {
        let fps = vec![
            ProblemFingerprint {
                id: "FP001".to_string(),
                name: "循环依赖".to_string(),
                category: FingerprintCategory::Architecture,
                severity: FingerprintSeverity::High,
                patterns: vec!["use crate::domain".to_string(), "use crate::application".to_string()],
                error_patterns: vec!["cyclic dependency".to_string(), "circular dependency".to_string()],
                symptoms: vec!["模块间相互引用".to_string(), "编译时出现循环依赖错误".to_string(), "架构层次混乱".to_string()],
                root_cause: "领域层、应用层、适配器层之间存在反向依赖，违反了依赖倒置原则".to_string(),
                fix_suggestion: "1. 检查依赖方向，确保依赖从外向内（适配器→应用→领域）\n2. 使用依赖注入（DI）解耦\n3. 引入接口/端口（Port）来隔离实现细节".to_string(),
                auto_fixable: false,
            },
            ProblemFingerprint {
                id: "FP002".to_string(),
                name: "上帝对象".to_string(),
                category: FingerprintCategory::Maintainability,
                severity: FingerprintSeverity::High,
                patterns: vec!["pub struct ".to_string(), "fn ".to_string()],
                error_patterns: vec!["too many arguments".to_string(), "struct is too large".to_string()],
                symptoms: vec!["单个结构体有大量字段（>20个）".to_string(), "单个方法有大量参数（>5个）".to_string(), "一个类/模块承担了太多职责".to_string()],
                root_cause: "单一职责原则（SRP）被违反，一个对象承担了太多不相关的功能".to_string(),
                fix_suggestion: "1. 将大类拆分为多个小类，每个类只负责一个职责\n2. 使用值对象（Value Object）封装相关字段\n3. 提取中间层（如领域服务、应用服务）".to_string(),
                auto_fixable: false,
            },
            ProblemFingerprint {
                id: "FP003".to_string(),
                name: "硬编码配置".to_string(),
                category: FingerprintCategory::Configuration,
                severity: FingerprintSeverity::Medium,
                patterns: vec!["\"http://".to_string(), "\"postgres://".to_string(), "let timeout = 30".to_string()],
                error_patterns: vec!["connection refused".to_string(), "invalid configuration".to_string()],
                symptoms: vec!["代码中直接写死了连接字符串、超时时间等配置".to_string(), "不同环境需要修改代码才能部署".to_string()],
                root_cause: "配置信息硬编码在代码中，没有外部化，违反了配置与代码分离的原则".to_string(),
                fix_suggestion: "1. 使用配置文件（cell.yaml / entropy.yaml）管理配置\n2. 使用环境变量覆盖敏感配置\n3. 引入配置服务（ConfigService）统一读取配置".to_string(),
                auto_fixable: false,
            },
            ProblemFingerprint {
                id: "FP004".to_string(),
                name: "缺少错误处理".to_string(),
                category: FingerprintCategory::Maintainability,
                severity: FingerprintSeverity::High,
                patterns: vec![".unwrap()".to_string(), ".expect(".to_string()],
                error_patterns: vec!["called `Result::unwrap()` on an `Err` value".to_string(), "panicked at".to_string()],
                symptoms: vec!["代码中大量使用 unwrap() 或 expect()".to_string(), "运行时出现 panic 而不是优雅降级".to_string(), "错误没有被正确传播和处理".to_string()],
                root_cause: "错误处理不完善，使用 unwrap 等会导致 panic 的操作，没有使用 Result 类型进行错误传播".to_string(),
                fix_suggestion: "1. 使用 ? 运算符传播错误\n2. 为函数返回 Result 类型\n3. 定义统一的错误类型（CellError）\n4. 只有在确定不会失败时才使用 unwrap（并添加注释说明）".to_string(),
                auto_fixable: false,
            },
            ProblemFingerprint {
                id: "FP005".to_string(),
                name: "测试覆盖率不足".to_string(),
                category: FingerprintCategory::Testing,
                severity: FingerprintSeverity::Medium,
                patterns: vec!["#[cfg(test)]".to_string()],
                error_patterns: vec!["test result: FAILED".to_string(), "0 passed".to_string()],
                symptoms: vec!["核心业务逻辑没有单元测试".to_string(), "测试只覆盖了 happy path".to_string(), "边界条件和异常情况没有测试".to_string()],
                root_cause: "测试不充分，核心逻辑和边界情况没有被覆盖，容易引入回归 bug".to_string(),
                fix_suggestion: "1. 为领域模型和业务逻辑编写单元测试\n2. 使用 TDD 方式，先写测试再实现\n3. 目标：核心业务逻辑覆盖率 > 80%\n4. 编写集成测试验证用例流程".to_string(),
                auto_fixable: false,
            },
            ProblemFingerprint {
                id: "FP006".to_string(),
                name: "贫血领域模型".to_string(),
                category: FingerprintCategory::Architecture,
                severity: FingerprintSeverity::Medium,
                patterns: vec!["pub struct ".to_string(), "pub ".to_string()],
                error_patterns: vec![],
                symptoms: vec!["领域对象只有 getter/setter，没有业务方法".to_string(), "业务逻辑全部在服务层".to_string(), "领域模型只是数据容器".to_string()],
                root_cause: "领域驱动设计（DDD）没有正确应用，领域对象退化为纯数据结构，业务逻辑外泄到服务层".to_string(),
                fix_suggestion: "1. 将业务逻辑移入领域对象（实体、值对象、聚合）\n2. 确保聚合根封装业务规则\n3. 使用领域服务协调跨聚合的业务逻辑\n4. 应用服务只负责编排，不包含业务规则".to_string(),
                auto_fixable: false,
            },
            ProblemFingerprint {
                id: "FP007".to_string(),
                name: "过度嵌套".to_string(),
                category: FingerprintCategory::Maintainability,
                severity: FingerprintSeverity::Medium,
                patterns: vec!["if ".to_string(), "for ".to_string(), "match ".to_string()],
                error_patterns: vec!["too deeply nested".to_string()],
                symptoms: vec!["函数嵌套层级过深（>4层）".to_string(), "代码可读性差，难以理解".to_string(), "圈复杂度过高".to_string()],
                root_cause: "函数逻辑过于复杂，嵌套层级过多，违反了保持函数简洁的原则".to_string(),
                fix_suggestion: "1. 使用提前返回（early return）减少嵌套\n2. 将复杂逻辑提取为小函数\n3. 使用卫语句（guard clause）处理边界条件\n4. 使用策略模式替代多层条件判断".to_string(),
                auto_fixable: false,
            },
            ProblemFingerprint {
                id: "FP008".to_string(),
                name: "魔法数字".to_string(),
                category: FingerprintCategory::Maintainability,
                severity: FingerprintSeverity::Low,
                patterns: vec!["1000".to_string(), "60 * 60".to_string(), "1024".to_string()],
                error_patterns: vec![],
                symptoms: vec!["代码中出现未命名的数字常量".to_string(), "相同数字在多处出现，难以维护".to_string()],
                root_cause: "使用了没有语义的魔法数字，代码可读性和可维护性差".to_string(),
                fix_suggestion: "1. 提取为具名常量（const 或 static）\n2. 使用配置项管理可调整的数值\n3. 为常量添加清晰的文档注释".to_string(),
                auto_fixable: false,
            },
            ProblemFingerprint {
                id: "FP009".to_string(),
                name: "命名不一致".to_string(),
                category: FingerprintCategory::Maintainability,
                severity: FingerprintSeverity::Low,
                patterns: vec!["user_id".to_string(), "userId".to_string(), "UserID".to_string()],
                error_patterns: vec![],
                symptoms: vec!["同一概念在不同地方有不同的命名".to_string(), "命名风格不统一（驼峰 vs 下划线）".to_string(), "缩写不一致".to_string()],
                root_cause: "缺乏统一的命名规范，或者规范没有被严格遵守".to_string(),
                fix_suggestion: "1. 遵循 Rust 命名规范（snake_case 变量/函数，PascalCase 类型）\n2. 建立团队统一的术语表\n3. 使用代码检查工具（clippy）自动化检查".to_string(),
                auto_fixable: false,
            },
            ProblemFingerprint {
                id: "FP010".to_string(),
                name: "依赖版本冲突".to_string(),
                category: FingerprintCategory::Dependency,
                severity: FingerprintSeverity::High,
                patterns: vec!["cargo tree".to_string()],
                error_patterns: vec!["version conflict".to_string(), "duplicate version".to_string(), "failed to select a version".to_string()],
                symptoms: vec!["同一个依赖有多个版本".to_string(), "编译时出现版本冲突错误".to_string(), "依赖树过大".to_string()],
                root_cause: "项目依赖管理不当，不同间接依赖引入了同一库的不同版本，或者版本范围过宽".to_string(),
                fix_suggestion: "1. 使用 cargo tree 查看依赖树\n2. 统一关键依赖的版本\n3. 使用 cargo update 更新依赖\n4. 精简不必要的依赖".to_string(),
                auto_fixable: false,
            },
            ProblemFingerprint {
                id: "FP011".to_string(),
                name: "泄漏内部实现".to_string(),
                category: FingerprintCategory::Architecture,
                severity: FingerprintSeverity::Medium,
                patterns: vec!["pub use ".to_string(), "pub mod ".to_string()],
                error_patterns: vec![],
                symptoms: vec!["模块公开了过多内部细节".to_string(), "其他模块直接依赖内部实现".to_string(), "修改内部实现导致大量外部代码变更".to_string()],
                root_cause: "封装不充分，模块边界不清晰，内部实现细节泄漏给外部使用者".to_string(),
                fix_suggestion: "1. 使用 pub(crate) 限制可见性\n2. 只公开必要的 API\n3. 使用 Facade 模式提供统一的对外接口\n4. 遵循最小知识原则（迪米特法则）".to_string(),
                auto_fixable: false,
            },
            ProblemFingerprint {
                id: "FP012".to_string(),
                name: "缺少日志埋点".to_string(),
                category: FingerprintCategory::Maintainability,
                severity: FingerprintSeverity::Low,
                patterns: vec!["pub async fn ".to_string()],
                error_patterns: vec!["no log output".to_string()],
                symptoms: vec!["关键操作没有日志记录".to_string(), "出现问题时难以排查".to_string(), "没有区分日志级别".to_string()],
                root_cause: "可观测性不足，关键路径缺少日志埋点，问题排查困难".to_string(),
                fix_suggestion: "1. 使用 tracing 库添加结构化日志\n2. 在入口/出口、关键决策点、错误处添加日志\n3. 合理使用日志级别（debug/info/warn/error）\n4. 添加 trace_id 便于链路追踪".to_string(),
                auto_fixable: false,
            },
            ProblemFingerprint {
                id: "FP013".to_string(),
                name: "同步阻塞操作".to_string(),
                category: FingerprintCategory::Performance,
                severity: FingerprintSeverity::High,
                patterns: vec!["std::fs::".to_string(), "std::net::".to_string(), "thread::sleep(".to_string()],
                error_patterns: vec!["blocking operation".to_string(), "slow I/O".to_string()],
                symptoms: vec!["异步代码中调用同步文件操作".to_string(), "使用 thread::sleep 替代 tokio::time::sleep".to_string(), "高并发场景下性能下降".to_string()],
                root_cause: "在异步上下文中使用了阻塞 I/O 操作，导致线程池被占满，影响并发性能".to_string(),
                fix_suggestion: "1. 使用 tokio::fs 替代 std::fs\n2. 使用 tokio::time::sleep 替代 thread::sleep\n3. 使用 spawn_blocking 包装不可避免的阻塞操作\n4. 使用 async 版本的网络库".to_string(),
                auto_fixable: false,
            },
            ProblemFingerprint {
                id: "FP014".to_string(),
                name: "SQL 注入风险".to_string(),
                category: FingerprintCategory::Security,
                severity: FingerprintSeverity::Critical,
                patterns: vec!["format!(".to_string(), "\"SELECT".to_string(), "\"INSERT".to_string(), "\"UPDATE".to_string()],
                error_patterns: vec!["SQL injection".to_string(), "untrusted input".to_string()],
                symptoms: vec!["使用字符串拼接构建 SQL 查询".to_string(), "直接将用户输入嵌入 SQL".to_string(), "缺少参数化查询".to_string()],
                root_cause: "SQL 查询使用字符串拼接方式构建，用户输入未经验证直接嵌入，存在 SQL 注入风险".to_string(),
                fix_suggestion: "1. 使用 ORM 框架（如 sqlx、diesel）的参数化查询\n2. 使用 prepare 语句\n3. 对用户输入进行严格验证和过滤\n4. 使用 SQL 白名单限制查询模板".to_string(),
                auto_fixable: false,
            },
            ProblemFingerprint {
                id: "FP015".to_string(),
                name: "未验证的用户输入".to_string(),
                category: FingerprintCategory::Security,
                severity: FingerprintSeverity::High,
                patterns: vec!["let input = ".to_string(), "request.query(".to_string(), "request.body(".to_string()],
                error_patterns: vec!["invalid input".to_string(), "bad request".to_string()],
                symptoms: vec!["直接使用用户输入而不进行验证".to_string(), "缺少输入边界检查".to_string(), "没有验证数据类型".to_string()],
                root_cause: "用户输入未经验证直接使用，可能导致数据错误、安全漏洞或拒绝服务攻击".to_string(),
                fix_suggestion: "1. 使用验证库（如 validator）进行输入验证\n2. 检查输入长度、类型、格式\n3. 使用类型系统强制约束（newtype 模式）\n4. 对敏感输入进行 sanitize".to_string(),
                auto_fixable: false,
            },
            ProblemFingerprint {
                id: "FP016".to_string(),
                name: "竞态条件".to_string(),
                category: FingerprintCategory::Performance,
                severity: FingerprintSeverity::High,
                patterns: vec!["std::sync::Mutex".to_string(), "std::sync::RwLock".to_string(), "Arc<".to_string()],
                error_patterns: vec!["deadlock".to_string(), "race condition".to_string(), "data race".to_string()],
                symptoms: vec!["多线程环境下共享状态访问".to_string(), "锁的使用顺序不一致".to_string(), "长时间持有锁".to_string()],
                root_cause: "多线程环境下对共享状态的并发访问没有正确同步，可能导致数据不一致或死锁".to_string(),
                fix_suggestion: "1. 使用无锁数据结构（如 dashmap）\n2. 缩小锁的作用域\n3. 使用原子操作替代锁\n4. 使用消息传递（channel）替代共享内存\n5. 确保锁的获取顺序一致".to_string(),
                auto_fixable: false,
            },
            ProblemFingerprint {
                id: "FP017".to_string(),
                name: "内存泄漏".to_string(),
                category: FingerprintCategory::Performance,
                severity: FingerprintSeverity::Medium,
                patterns: vec!["std::mem::forget(".to_string(), "Box::leak(".to_string(), "Rc::new(".to_string()],
                error_patterns: vec!["memory leak".to_string(), "growing memory".to_string()],
                symptoms: vec!["内存使用持续增长".to_string(), "长时间运行后 OOM".to_string(), "引用循环导致内存无法释放".to_string()],
                root_cause: "内存资源没有被正确释放，可能是显式泄漏、引用循环或生命周期管理不当".to_string(),
                fix_suggestion: "1. 使用 Weak 打破引用循环\n2. 避免使用 Box::leak 和 mem::forget\n3. 使用 RAII 模式管理资源\n4. 运行 cargo clippy 检查内存问题".to_string(),
                auto_fixable: false,
            },
            ProblemFingerprint {
                id: "FP018".to_string(),
                name: "密码明文存储".to_string(),
                category: FingerprintCategory::Security,
                severity: FingerprintSeverity::Critical,
                patterns: vec!["password".to_string(), "secret".to_string()],
                error_patterns: vec!["plain text password".to_string(), "credential leak".to_string()],
                symptoms: vec!["密码或密钥以明文形式存储".to_string(), "配置文件中包含明文密码".to_string(), "日志中打印敏感信息".to_string()],
                root_cause: "敏感信息（密码、密钥、令牌）以明文形式存储或传输，容易被泄露".to_string(),
                fix_suggestion: "1. 使用 bcrypt/scrypt 等强哈希算法存储密码\n2. 使用密钥管理服务（KMS）\n3. 使用环境变量或密钥文件\n4. 禁止在日志中打印敏感信息".to_string(),
                auto_fixable: false,
            },
            ProblemFingerprint {
                id: "FP019".to_string(),
                name: "硬编码密钥".to_string(),
                category: FingerprintCategory::Security,
                severity: FingerprintSeverity::Critical,
                patterns: vec!["sk-".to_string(), "pk-".to_string(), "api_key".to_string(), "SECRET_KEY".to_string()],
                error_patterns: vec!["hardcoded secret".to_string(), "exposed credential".to_string()],
                symptoms: vec!["API 密钥、加密密钥硬编码在代码中".to_string(), "密钥提交到版本控制系统".to_string(), "不同环境使用相同密钥".to_string()],
                root_cause: "密钥等敏感配置硬编码在源代码中，一旦代码泄露，密钥也会随之泄露".to_string(),
                fix_suggestion: "1. 使用环境变量存储密钥\n2. 使用 vault/secrets manager\n3. 使用 .env 文件并加入 .gitignore\n4. 使用配置服务动态获取密钥".to_string(),
                auto_fixable: false,
            },
            ProblemFingerprint {
                id: "FP020".to_string(),
                name: "跨站脚本攻击(XSS)".to_string(),
                category: FingerprintCategory::Security,
                severity: FingerprintSeverity::High,
                patterns: vec!["html!(".to_string(), "format!(".to_string(), ".push_str(".to_string()],
                error_patterns: vec!["XSS".to_string(), "cross-site scripting".to_string(), "script injection".to_string()],
                symptoms: vec!["直接将用户输入渲染到 HTML".to_string(), "使用字符串拼接构建 HTML".to_string(), "缺少 HTML 转义".to_string()],
                root_cause: "用户输入未经转义直接渲染到页面，可能被注入恶意脚本".to_string(),
                fix_suggestion: "1. 使用模板引擎自动转义\n2. 对用户输入进行 HTML 转义\n3. 使用 Content-Security-Policy 头部\n4. 避免使用 innerHTML".to_string(),
                auto_fixable: false,
            },
            ProblemFingerprint {
                id: "FP021".to_string(),
                name: "未授权访问".to_string(),
                category: FingerprintCategory::Security,
                severity: FingerprintSeverity::High,
                patterns: vec!["pub async fn ".to_string(), "GET /".to_string(), "POST /".to_string()],
                error_patterns: vec!["unauthorized".to_string(), "access denied".to_string()],
                symptoms: vec!["API 端点没有认证检查".to_string(), "敏感操作缺少权限验证".to_string(), "公开暴露内部接口".to_string()],
                root_cause: "访问控制不完善，未授权用户可以访问或操作敏感资源".to_string(),
                fix_suggestion: "1. 使用中间件进行认证\n2. 实现 RBAC（基于角色的访问控制）\n3. 对敏感操作进行权限检查\n4. 使用 JWT/OAuth2 进行身份验证".to_string(),
                auto_fixable: false,
            },
            ProblemFingerprint {
                id: "FP022".to_string(),
                name: "N+1 查询问题".to_string(),
                category: FingerprintCategory::Performance,
                severity: FingerprintSeverity::High,
                patterns: vec![".find(".to_string(), ".get(".to_string(), ".query(".to_string()],
                error_patterns: vec!["too many queries".to_string(), "slow query".to_string()],
                symptoms: vec!["循环中执行数据库查询".to_string(), "加载列表时为每个项单独查询".to_string(), "数据库查询次数随数据量线性增长".to_string()],
                root_cause: "没有使用 JOIN 或预加载，导致需要执行 N+1 次数据库查询".to_string(),
                fix_suggestion: "1. 使用 JOIN 查询一次性获取关联数据\n2. 使用 ORM 的预加载功能（preload/include）\n3. 使用批量查询\n4. 添加适当的索引".to_string(),
                auto_fixable: false,
            },
            ProblemFingerprint {
                id: "FP023".to_string(),
                name: "缺少索引".to_string(),
                category: FingerprintCategory::Performance,
                severity: FingerprintSeverity::Medium,
                patterns: vec!["CREATE TABLE".to_string(), "table!(".to_string()],
                error_patterns: vec!["full table scan".to_string(), "slow query".to_string()],
                symptoms: vec!["查询响应时间随数据量增长".to_string(), "频繁的全表扫描".to_string(), "WHERE 条件字段没有索引".to_string()],
                root_cause: "数据库表缺少必要的索引，导致查询性能低下".to_string(),
                fix_suggestion: "1. 为 WHERE 条件和 JOIN 字段添加索引\n2. 使用 EXPLAIN 分析查询计划\n3. 考虑复合索引\n4. 定期审查和优化索引".to_string(),
                auto_fixable: false,
            },
            ProblemFingerprint {
                id: "FP024".to_string(),
                name: "过度递归".to_string(),
                category: FingerprintCategory::Performance,
                severity: FingerprintSeverity::Medium,
                patterns: vec!["fn ".to_string(), "recursive".to_string()],
                error_patterns: vec!["stack overflow".to_string(), "recursion depth".to_string()],
                symptoms: vec!["递归调用深度过大".to_string(), "大输入导致栈溢出".to_string(), "递归没有终止条件检查".to_string()],
                root_cause: "递归实现没有考虑深度限制，或者可以用迭代方式优化".to_string(),
                fix_suggestion: "1. 使用迭代替代递归\n2. 增加递归深度限制\n3. 使用尾递归优化（Rust 支持）\n4. 使用动态规划缓存中间结果".to_string(),
                auto_fixable: false,
            },
            ProblemFingerprint {
                id: "FP025".to_string(),
                name: "重复计算".to_string(),
                category: FingerprintCategory::Performance,
                severity: FingerprintSeverity::Low,
                patterns: vec!["let result = ".to_string(), "fn compute(".to_string()],
                error_patterns: vec![],
                symptoms: vec!["相同计算在循环中重复执行".to_string(), "没有缓存计算结果".to_string(), "昂贵的操作被频繁调用".to_string()],
                root_cause: "没有缓存中间结果，导致相同的计算被重复执行".to_string(),
                fix_suggestion: "1. 将计算结果提取到循环外\n2. 使用 memoization 缓存结果\n3. 使用惰性求值\n4. 预计算并缓存".to_string(),
                auto_fixable: false,
            },
            ProblemFingerprint {
                id: "FP026".to_string(),
                name: "不必要的克隆".to_string(),
                category: FingerprintCategory::Performance,
                severity: FingerprintSeverity::Low,
                patterns: vec![".clone()".to_string(), ".to_string()".to_string()],
                error_patterns: vec![],
                symptoms: vec!["频繁使用 clone()".to_string(), "大对象被不必要地复制".to_string(), "可以使用引用的地方使用了克隆".to_string()],
                root_cause: "没有正确使用借用和生命周期，导致不必要的内存分配和复制".to_string(),
                fix_suggestion: "1. 使用引用（&）替代克隆\n2. 使用 Cow 智能指针\n3. 使用 move 语义转移所有权\n4. 运行 cargo clippy 检查".to_string(),
                auto_fixable: false,
            },
            ProblemFingerprint {
                id: "FP027".to_string(),
                name: "缺乏错误上下文".to_string(),
                category: FingerprintCategory::Maintainability,
                severity: FingerprintSeverity::Medium,
                patterns: vec!["return Err(".to_string(), "Err(".to_string()],
                error_patterns: vec!["unknown error".to_string(), "error occurred".to_string()],
                symptoms: vec!["错误信息过于笼统".to_string(), "缺少上下文信息".to_string(), "难以定位问题根源".to_string()],
                root_cause: "错误信息缺乏上下文，难以理解问题发生的原因和位置".to_string(),
                fix_suggestion: "1. 使用 thiserror 定义详细的错误类型\n2. 添加文件名、行号、参数等上下文\n3. 使用 eyre 或 anyhow 提供更好的错误链\n4. 错误信息应包含：what、where、why、how".to_string(),
                auto_fixable: false,
            },
            ProblemFingerprint {
                id: "FP028".to_string(),
                name: "魔法字符串".to_string(),
                category: FingerprintCategory::Maintainability,
                severity: FingerprintSeverity::Low,
                patterns: vec!["\"status\"".to_string(), "\"type\"".to_string(), "\"id\"".to_string()],
                error_patterns: vec![],
                symptoms: vec!["代码中出现未命名的字符串常量".to_string(), "相同字符串在多处硬编码".to_string(), "修改需要全局搜索替换".to_string()],
                root_cause: "使用了没有语义的魔法字符串，代码可读性和可维护性差".to_string(),
                fix_suggestion: "1. 提取为常量或枚举\n2. 使用强类型替代字符串\n3. 使用配置管理字符串\n4. 使用 serde 宏自动生成".to_string(),
                auto_fixable: false,
            },
            ProblemFingerprint {
                id: "FP029".to_string(),
                name: "过长函数".to_string(),
                category: FingerprintCategory::Maintainability,
                severity: FingerprintSeverity::Medium,
                patterns: vec!["fn ".to_string()],
                error_patterns: vec!["function is too long".to_string()],
                symptoms: vec!["单个函数超过 50 行".to_string(), "函数做了太多事情".to_string(), "难以测试和理解".to_string()],
                root_cause: "函数职责过多，违反了单一职责原则，导致可读性和可维护性下降".to_string(),
                fix_suggestion: "1. 将大函数拆分为多个小函数\n2. 每个函数只做一件事\n3. 使用辅助函数提取重复逻辑\n4. 使用闭包或迭代器简化代码".to_string(),
                auto_fixable: false,
            },
            ProblemFingerprint {
                id: "FP030".to_string(),
                name: "重复代码".to_string(),
                category: FingerprintCategory::Maintainability,
                severity: FingerprintSeverity::Medium,
                patterns: vec!["if ".to_string(), "match ".to_string()],
                error_patterns: vec![],
                symptoms: vec!["相似的代码块在多处出现".to_string(), "修改一处需要同步修改多处".to_string(), "代码膨胀".to_string()],
                root_cause: "没有抽取公共逻辑，导致代码重复，维护成本增加".to_string(),
                fix_suggestion: "1. 提取公共函数或 trait\n2. 使用宏消除重复\n3. 使用模板方法模式\n4. 使用代码生成工具".to_string(),
                auto_fixable: false,
            },
            ProblemFingerprint {
                id: "FP031".to_string(),
                name: "缺失文档".to_string(),
                category: FingerprintCategory::Maintainability,
                severity: FingerprintSeverity::Low,
                patterns: vec!["pub fn ".to_string(), "pub struct ".to_string()],
                error_patterns: vec![],
                symptoms: vec!["公共 API 缺少文档注释".to_string(), "复杂逻辑没有解释".to_string(), "难以理解代码意图".to_string()],
                root_cause: "缺乏代码文档，影响团队协作和代码理解".to_string(),
                fix_suggestion: "1. 使用 rustdoc 格式编写文档\n2. 为公共 API 添加 #[doc] 注释\n3. 解释复杂逻辑和设计决策\n4. 添加示例代码".to_string(),
                auto_fixable: false,
            },
            ProblemFingerprint {
                id: "FP032".to_string(),
                name: "接口污染".to_string(),
                category: FingerprintCategory::Architecture,
                severity: FingerprintSeverity::Medium,
                patterns: vec!["pub trait ".to_string(), "impl ".to_string()],
                error_patterns: vec![],
                symptoms: vec!["接口包含过多方法".to_string(), "实现类只需要接口的部分方法".to_string(), "接口变更影响所有实现".to_string()],
                root_cause: "接口设计过大，违反了接口隔离原则（ISP）".to_string(),
                fix_suggestion: "1. 将大接口拆分为多个小接口\n2. 使用 trait 组合\n3. 遵循接口隔离原则\n4. 使用 marker trait".to_string(),
                auto_fixable: false,
            },
            ProblemFingerprint {
                id: "FP033".to_string(),
                name: "继承滥用".to_string(),
                category: FingerprintCategory::Architecture,
                severity: FingerprintSeverity::Medium,
                patterns: vec!["struct ".to_string(), "enum ".to_string()],
                error_patterns: vec![],
                symptoms: vec!["使用 enum 变体模拟继承".to_string(), "深层嵌套的枚举".to_string(), "需要类型转换的复杂层次".to_string()],
                root_cause: "在 Rust 中过度模拟继承模式，而不是使用组合".to_string(),
                fix_suggestion: "1. 使用组合替代继承\n2. 使用 trait 对象\n3. 使用 newtype 模式\n4. 使用泛型".to_string(),
                auto_fixable: false,
            },
            ProblemFingerprint {
                id: "FP034".to_string(),
                name: "缺少抽象".to_string(),
                category: FingerprintCategory::Architecture,
                severity: FingerprintSeverity::Medium,
                patterns: vec!["impl ".to_string(), "pub struct ".to_string()],
                error_patterns: vec![],
                symptoms: vec!["具体实现直接暴露".to_string(), "难以替换实现".to_string(), "紧耦合".to_string()],
                root_cause: "缺少接口抽象，导致调用方与具体实现紧耦合".to_string(),
                fix_suggestion: "1. 定义 trait 抽象\n2. 使用依赖注入\n3. 编程面向接口\n4. 使用策略模式".to_string(),
                auto_fixable: false,
            },
            ProblemFingerprint {
                id: "FP035".to_string(),
                name: "领域服务膨胀".to_string(),
                category: FingerprintCategory::Architecture,
                severity: FingerprintSeverity::High,
                patterns: vec!["pub struct ".to_string(), "pub async fn ".to_string()],
                error_patterns: vec![],
                symptoms: vec!["领域服务类包含大量方法".to_string(), "服务承担了不属于领域层的职责".to_string(), "服务之间互相依赖".to_string()],
                root_cause: "领域服务设计不合理，承担了过多职责，违反了单一职责原则".to_string(),
                fix_suggestion: "1. 将服务拆分为多个小服务\n2. 确保服务只包含领域逻辑\n3. 使用应用服务编排领域服务\n4. 使用聚合根封装业务规则".to_string(),
                auto_fixable: false,
            },
            ProblemFingerprint {
                id: "FP036".to_string(),
                name: "聚合边界模糊".to_string(),
                category: FingerprintCategory::Architecture,
                severity: FingerprintSeverity::High,
                patterns: vec!["struct ".to_string(), "pub ".to_string()],
                error_patterns: vec![],
                symptoms: vec!["聚合根没有正确封装".to_string(), "外部直接访问聚合内部实体".to_string(), "事务边界不清晰".to_string()],
                root_cause: "聚合边界设计不合理，导致数据一致性难以保证".to_string(),
                fix_suggestion: "1. 明确聚合边界\n2. 只有聚合根可以被外部访问\n3. 通过聚合根操作内部实体\n4. 使用值对象保证不变性".to_string(),
                auto_fixable: false,
            },
            ProblemFingerprint {
                id: "FP037".to_string(),
                name: "缺少集成测试".to_string(),
                category: FingerprintCategory::Testing,
                severity: FingerprintSeverity::Medium,
                patterns: vec!["#[cfg(test)]".to_string(), "#[test]".to_string()],
                error_patterns: vec![],
                symptoms: vec!["只有单元测试，没有集成测试".to_string(), "模块之间的交互没有测试".to_string(), "端到端流程没有验证".to_string()],
                root_cause: "只测试了单个组件，没有验证组件之间的协作是否正确".to_string(),
                fix_suggestion: "1. 编写集成测试验证模块协作\n2. 使用 Testcontainers 测试外部依赖\n3. 编写端到端测试\n4. 使用 property-based testing".to_string(),
                auto_fixable: false,
            },
            ProblemFingerprint {
                id: "FP038".to_string(),
                name: "测试缺乏断言".to_string(),
                category: FingerprintCategory::Testing,
                severity: FingerprintSeverity::Medium,
                patterns: vec!["#[test]".to_string(), "async fn ".to_string()],
                error_patterns: vec![],
                symptoms: vec!["测试代码没有断言".to_string(), "测试只执行不验证结果".to_string(), "测试总是通过".to_string()],
                root_cause: "测试没有验证实际结果，无法发现回归问题".to_string(),
                fix_suggestion: "1. 为每个测试添加明确的断言\n2. 使用 assert_eq、assert_ne 等宏\n3. 验证返回值和副作用\n4. 使用快照测试".to_string(),
                auto_fixable: false,
            },
            ProblemFingerprint {
                id: "FP039".to_string(),
                name: "慢测试".to_string(),
                category: FingerprintCategory::Testing,
                severity: FingerprintSeverity::Low,
                patterns: vec!["#[test]".to_string(), "sleep(".to_string()],
                error_patterns: vec![],
                symptoms: vec!["单元测试执行时间过长".to_string(), "测试依赖外部服务".to_string(), "使用 sleep 等待".to_string()],
                root_cause: "测试设计不合理，包含不必要的等待或外部依赖".to_string(),
                fix_suggestion: "1. 使用 mock 替代外部依赖\n2. 使用 async/await 替代 sleep\n3. 并行运行测试\n4. 将慢测试移到集成测试套件".to_string(),
                auto_fixable: false,
            },
            ProblemFingerprint {
                id: "FP040".to_string(),
                name: "依赖过多".to_string(),
                category: FingerprintCategory::Dependency,
                severity: FingerprintSeverity::Medium,
                patterns: vec![],
                error_patterns: vec!["cargo tree".to_string(), "too many dependencies".to_string()],
                symptoms: vec!["Cargo.toml 中依赖数量过多".to_string(), "依赖树过于庞大".to_string(), "编译时间过长".to_string()],
                root_cause: "项目引入了过多不必要的依赖，增加了维护成本和安全风险".to_string(),
                fix_suggestion: "1. 定期审查依赖，移除不必要的\n2. 使用 cargo tree 分析依赖\n3. 考虑使用标准库替代第三方库\n4. 使用 features 只启用需要的功能".to_string(),
                auto_fixable: false,
            },
            ProblemFingerprint {
                id: "FP041".to_string(),
                name: "依赖版本过时".to_string(),
                category: FingerprintCategory::Dependency,
                severity: FingerprintSeverity::Low,
                patterns: vec![],
                error_patterns: vec!["outdated".to_string(), "security advisory".to_string()],
                symptoms: vec!["依赖版本长时间未更新".to_string(), "收到安全告警".to_string(), "缺少新特性和 bug 修复".to_string()],
                root_cause: "依赖版本管理不善，没有定期更新".to_string(),
                fix_suggestion: "1. 定期运行 cargo update\n2. 使用 cargo audit 检查安全漏洞\n3. 设置 Dependabot 自动更新\n4. 关注依赖的 release notes".to_string(),
                auto_fixable: false,
            },
            ProblemFingerprint {
                id: "FP042".to_string(),
                name: "配置缺失".to_string(),
                category: FingerprintCategory::Configuration,
                severity: FingerprintSeverity::Medium,
                patterns: vec!["cell.yaml".to_string(), "entropy.yaml".to_string()],
                error_patterns: vec![],
                symptoms: vec!["缺少项目配置文件".to_string(), "配置文件不完整".to_string(), "不同环境使用相同配置".to_string()],
                root_cause: "项目缺少必要的配置管理，导致部署和运行时问题".to_string(),
                fix_suggestion: "1. 创建 cell.yaml 配置文件\n2. 定义环境特定的配置\n3. 使用配置继承\n4. 添加配置验证".to_string(),
                auto_fixable: false,
            },
            ProblemFingerprint {
                id: "FP043".to_string(),
                name: "环境配置混乱".to_string(),
                category: FingerprintCategory::Configuration,
                severity: FingerprintSeverity::Medium,
                patterns: vec!["if cfg!(".to_string(), "std::env::var(".to_string()],
                error_patterns: vec![],
                symptoms: vec!["环境相关代码分散在各处".to_string(), "缺少统一的环境管理".to_string(), "难以切换环境".to_string()],
                root_cause: "环境配置管理不当，没有使用统一的配置服务".to_string(),
                fix_suggestion: "1. 使用配置服务统一管理环境配置\n2. 使用 Feature Flags\n3. 定义环境变量命名规范\n4. 使用多环境配置文件".to_string(),
                auto_fixable: false,
            },
            ProblemFingerprint {
                id: "FP044".to_string(),
                name: "缺少健康检查".to_string(),
                category: FingerprintCategory::Maintainability,
                severity: FingerprintSeverity::Low,
                patterns: vec!["axum::Router".to_string(), "Route".to_string()],
                error_patterns: vec![],
                symptoms: vec!["没有健康检查端点".to_string(), "无法监控服务状态".to_string(), "容器编排无法判断服务是否正常".to_string()],
                root_cause: "缺少健康检查机制，影响运维和监控".to_string(),
                fix_suggestion: "1. 添加 /health 端点\n2. 检查关键依赖（数据库、缓存）\n3. 返回详细的健康状态\n4. 集成到监控系统".to_string(),
                auto_fixable: false,
            },
            ProblemFingerprint {
                id: "FP045".to_string(),
                name: "日志敏感信息".to_string(),
                category: FingerprintCategory::Security,
                severity: FingerprintSeverity::High,
                patterns: vec!["info!(".to_string(), "debug!(".to_string(), "println!(".to_string()],
                error_patterns: vec![],
                symptoms: vec!["日志中包含密码、令牌等敏感信息".to_string(), "使用 println! 打印敏感数据".to_string(), "没有对日志进行脱敏".to_string()],
                root_cause: "日志记录了敏感信息，可能导致信息泄露".to_string(),
                fix_suggestion: "1. 对敏感字段进行脱敏处理\n2. 使用结构化日志并标记敏感字段\n3. 禁止在日志中记录密码、令牌\n4. 使用日志级别控制敏感信息输出".to_string(),
                auto_fixable: false,
            },
            ProblemFingerprint {
                id: "FP046".to_string(),
                name: "未使用的依赖".to_string(),
                category: FingerprintCategory::Dependency,
                severity: FingerprintSeverity::Low,
                patterns: vec![],
                error_patterns: vec!["unused dependency".to_string(), "package is not used".to_string()],
                symptoms: vec!["Cargo.toml 中有未使用的依赖".to_string(), "编译时间增加".to_string(), "安全风险面扩大".to_string()],
                root_cause: "依赖管理不善，引入了不需要的包".to_string(),
                fix_suggestion: "1. 运行 cargo unused-deps 检查\n2. 移除未使用的依赖\n3. 使用 features 精简依赖\n4. 定期审查".to_string(),
                auto_fixable: false,
            },
            ProblemFingerprint {
                id: "FP047".to_string(),
                name: "类型转换不安全".to_string(),
                category: FingerprintCategory::Maintainability,
                severity: FingerprintSeverity::Medium,
                patterns: vec!["as ".to_string(), "unsafe ".to_string(), "transmute".to_string()],
                error_patterns: vec!["overflow".to_string(), "invalid cast".to_string()],
                symptoms: vec!["使用 as 进行可能溢出的类型转换".to_string(), "使用 unsafe 代码".to_string(), "使用 transmute".to_string()],
                root_cause: "类型转换没有进行安全检查，可能导致未定义行为".to_string(),
                fix_suggestion: "1. 使用 TryFrom/TryInto 进行安全转换\n2. 使用 checked_* 方法\n3. 避免 unsafe 代码\n4. 确保转换不会溢出".to_string(),
                auto_fixable: false,
            },
            ProblemFingerprint {
                id: "FP048".to_string(),
                name: "缺少重试机制".to_string(),
                category: FingerprintCategory::Maintainability,
                severity: FingerprintSeverity::Medium,
                patterns: vec![".await".to_string(), "reqwest::".to_string(), "sqlx::".to_string()],
                error_patterns: vec!["connection refused".to_string(), "temporary failure".to_string(), "timeout".to_string()],
                symptoms: vec!["网络请求没有重试".to_string(), "数据库连接失败没有恢复".to_string(), "对临时错误没有容错".to_string()],
                root_cause: "没有为可能失败的操作添加重试机制，系统稳定性差".to_string(),
                fix_suggestion: "1. 使用 retry 或 backoff 库\n2. 实现指数退避策略\n3. 设置最大重试次数\n4. 区分可重试和不可重试错误".to_string(),
                auto_fixable: false,
            },
            ProblemFingerprint {
                id: "FP049".to_string(),
                name: "未处理的 panic".to_string(),
                category: FingerprintCategory::Maintainability,
                severity: FingerprintSeverity::High,
                patterns: vec!["panic!(".to_string(), "unreachable!(".to_string(), "todo!(".to_string()],
                error_patterns: vec!["panicked at".to_string(), "thread panicked".to_string()],
                symptoms: vec!["生产代码中使用 panic!".to_string(), "使用 unreachable! 但实际可能到达".to_string(), "遗留的 todo! 宏".to_string()],
                root_cause: "使用 panic 替代错误处理，导致程序崩溃而不是优雅降级".to_string(),
                fix_suggestion: "1. 使用 Result 类型替代 panic\n2. 使用 expect 并提供详细错误信息\n3. 移除遗留的 todo!\n4. 只在开发阶段使用 panic".to_string(),
                auto_fixable: false,
            },
            ProblemFingerprint {
                id: "FP050".to_string(),
                name: "缺少超时控制".to_string(),
                category: FingerprintCategory::Performance,
                severity: FingerprintSeverity::Medium,
                patterns: vec![".await".to_string(), "reqwest::".to_string()],
                error_patterns: vec!["timeout".to_string(), "hang".to_string()],
                symptoms: vec!["网络请求没有超时设置".to_string(), "长时间等待导致资源耗尽".to_string(), "没有取消机制".to_string()],
                root_cause: "操作没有超时控制，可能导致资源长时间占用".to_string(),
                fix_suggestion: "1. 使用 tokio::time::timeout\n2. 设置合理的超时时间\n3. 使用 cancel-safe 的 futures\n4. 添加请求取消机制".to_string(),
                auto_fixable: false,
            },
            ProblemFingerprint {
                id: "FP051".to_string(),
                name: "数据库连接池配置不当".to_string(),
                category: FingerprintCategory::Performance,
                severity: FingerprintSeverity::Medium,
                patterns: vec!["sqlx::Pool".to_string(), "postgres::Client".to_string()],
                error_patterns: vec![],
                symptoms: vec!["连接池大小设置不合理".to_string(), "连接泄漏".to_string(), "连接超时".to_string()],
                root_cause: "数据库连接池配置不当，影响应用性能和稳定性".to_string(),
                fix_suggestion: "1. 根据并发需求调整池大小\n2. 设置连接超时和空闲超时\n3. 监控连接池状态\n4. 使用连接池健康检查".to_string(),
                auto_fixable: false,
            },
            ProblemFingerprint {
                id: "FP052".to_string(),
                name: "缺少限流".to_string(),
                category: FingerprintCategory::Security,
                severity: FingerprintSeverity::Medium,
                patterns: vec!["axum::Router".to_string(), "handler".to_string()],
                error_patterns: vec!["rate limit".to_string(), "too many requests".to_string()],
                symptoms: vec!["API 没有限流保护".to_string(), "容易受到 DDoS 攻击".to_string(), "资源耗尽".to_string()],
                root_cause: "没有对 API 访问进行限流，容易受到恶意请求攻击".to_string(),
                fix_suggestion: "1. 使用 tower-http 的限流中间件\n2. 实现令牌桶或漏桶算法\n3. 根据用户/IP 进行限流\n4. 设置合理的限流阈值".to_string(),
                auto_fixable: false,
            },
        ];

        for fp in fps {
            self.fingerprints.push(fp);
        }
    }

    pub fn register(&mut self, fingerprint: ProblemFingerprint) {
        self.custom_fingerprints.insert(fingerprint.id.clone(), fingerprint);
    }

    pub fn all_fingerprints(&self) -> Vec<ProblemFingerprint> {
        let mut all = self.fingerprints.clone();
        all.extend(self.custom_fingerprints.values().cloned());
        all
    }

    pub fn get_by_id(&self, id: &str) -> Option<ProblemFingerprint> {
        self.fingerprints
            .iter()
            .find(|f| f.id == id)
            .cloned()
            .or_else(|| self.custom_fingerprints.get(id).cloned())
    }

    pub fn match_code(&self, content: &str) -> Vec<FingerprintMatch> {
        let mut matches = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        for fp in self.all_fingerprints() {
            let mut matched_patterns = Vec::new();
            let mut matched_lines = Vec::new();

            for (i, line) in lines.iter().enumerate() {
                for pattern in &fp.patterns {
                    if line.contains(pattern) && !matched_patterns.contains(pattern) {
                        matched_patterns.push(pattern.clone());
                        matched_lines.push(i + 1);
                    }
                }
            }

            if !matched_patterns.is_empty() {
                let confidence = matched_patterns.len() as f64 / fp.patterns.len() as f64;
                matches.push(FingerprintMatch {
                    fingerprint: fp,
                    confidence: confidence.min(1.0),
                    matched_patterns,
                    matched_lines,
                });
            }
        }

        matches.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
        matches
    }

    pub fn match_error(&self, error_msg: &str) -> Vec<FingerprintMatch> {
        let mut matches = Vec::new();

        for fp in self.all_fingerprints() {
            if fp.matches_error(error_msg) {
                matches.push(FingerprintMatch {
                    confidence: 0.8,
                    matched_patterns: fp.error_patterns.clone(),
                    matched_lines: vec![],
                    fingerprint: fp,
                });
            }
        }

        matches.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
        matches
    }

    pub fn by_category(&self, category: &FingerprintCategory) -> Vec<ProblemFingerprint> {
        self.all_fingerprints()
            .into_iter()
            .filter(|f| &f.category == category)
            .collect()
    }

    pub fn by_severity(&self, severity: &FingerprintSeverity) -> Vec<ProblemFingerprint> {
        self.all_fingerprints()
            .into_iter()
            .filter(|f| &f.severity == severity)
            .collect()
    }
}

impl Default for FingerprintLibrary {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_fingerprints_count() {
        let lib = FingerprintLibrary::new();
        assert!(lib.all_fingerprints().len() >= 50);
    }

    #[test]
    fn test_match_unwrap_pattern() {
        let lib = FingerprintLibrary::new();
        let code = r"fn test() { let x = Some(1); let y = x.unwrap(); }";
        let matches = lib.match_code(code);
        assert!(matches.iter().any(|m| m.fingerprint.id == "FP004"));
    }

    #[test]
    fn test_match_error_message() {
        let lib = FingerprintLibrary::new();
        let error_msg = "called `Result::unwrap()` on an `Err` value";
        let matches = lib.match_error(error_msg);
        assert!(!matches.is_empty());
        assert!(matches.iter().any(|m| m.fingerprint.id == "FP004"));
    }

    #[test]
    fn test_custom_fingerprint_registration() {
        let mut lib = FingerprintLibrary::new();
        let custom = ProblemFingerprint {
            id: "CUSTOM001".to_string(),
            name: "自定义问题".to_string(),
            category: FingerprintCategory::Performance,
            severity: FingerprintSeverity::Medium,
            patterns: vec!["custom_pattern".to_string()],
            error_patterns: vec![],
            symptoms: vec![],
            root_cause: "测试".to_string(),
            fix_suggestion: "测试".to_string(),
            auto_fixable: false,
        };
        lib.register(custom);
        assert!(lib.get_by_id("CUSTOM001").is_some());
    }

    #[test]
    fn test_fingerprint_categories() {
        let lib = FingerprintLibrary::new();
        let arch_fps = lib.by_category(&FingerprintCategory::Architecture);
        assert!(!arch_fps.is_empty());
    }

    #[test]
    fn test_fingerprint_severities() {
        let lib = FingerprintLibrary::new();
        let high_fps = lib.by_severity(&FingerprintSeverity::High);
        assert!(!high_fps.is_empty());
    }

    #[test]
    fn test_match_sql_injection_pattern() {
        let lib = FingerprintLibrary::new();
        let code = r#"let query = format!("SELECT * FROM users WHERE id = {}", user_id);"#;
        let matches = lib.match_code(code);
        assert!(matches.iter().any(|m| m.fingerprint.id == "FP014"));
    }

    #[test]
    fn test_match_sync_blocking_pattern() {
        let lib = FingerprintLibrary::new();
        let code = r#"std::fs::read_to_string("file.txt");"#;
        let matches = lib.match_code(code);
        assert!(matches.iter().any(|m| m.fingerprint.id == "FP013"));
    }

    #[test]
    fn test_match_panic_pattern() {
        let lib = FingerprintLibrary::new();
        let code = r#"panic!("Something went wrong");"#;
        let matches = lib.match_code(code);
        assert!(matches.iter().any(|m| m.fingerprint.id == "FP049"));
    }

    #[test]
    fn test_match_clone_pattern() {
        let lib = FingerprintLibrary::new();
        let code = r"let s = some_string.clone();";
        let matches = lib.match_code(code);
        assert!(matches.iter().any(|m| m.fingerprint.id == "FP026"));
    }
}
