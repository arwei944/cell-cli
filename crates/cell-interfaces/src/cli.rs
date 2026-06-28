use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "cell",
    version,
    about = "Cell Architecture CLI - AI-native low-entropy architecture toolchain",
    long_about = "Cell 架构命令行工具：面向 AI 智能体原生开发的低熵架构工具链\n\
                  Architecture is the Prompt — 架构即提示",
    arg_required_else_help = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    #[arg(short, long, global = true, action = clap::ArgAction::Count, help = "Verbose output (-v, -vv, -vvv)")]
    pub verbose: u8,

    #[arg(short, long, global = true, help = "Output format: text, json, yaml")]
    pub format: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    #[command(about = "Initialize a new Cell project")]
    Init(InitArgs),

    #[command(about = "Generate code from spec (aliases: g)")]
    #[command(visible_aliases = ["g", "gen"])]
    Generate(GenerateArgs),

    #[command(about = "Validate architecture rules (aliases: v)")]
    #[command(visible_aliases = ["v", "val"])]
    Validate(ValidateArgs),

    #[command(about = "Calculate entropy metrics")]
    #[command(visible_aliases = ["e", "ent"])]
    Entropy(EntropyArgs),

    #[command(about = "Feature unit management")]
    #[command(visible_aliases = ["f", "feat"])]
    Feature(FeatureArgs),

    #[command(about = "Diagnose code issues with fingerprint library")]
    #[command(visible_aliases = ["diag"])]
    Diagnose(DiagnoseArgs),

    #[command(about = "Event schema management and compatibility check")]
    #[command(visible_aliases = ["evt"])]
    Event(EventArgs),

    #[command(about = "Generate/parse agent handoff context")]
    #[command(visible_aliases = ["h", "hand"])]
    Handoff(HandoffArgs),

    #[command(about = "Real-time progress tracking for agents")]
    #[command(visible_aliases = ["p", "prog"])]
    Progress(ProgressArgs),

    #[command(about = "Self-evolution system for tools")]
    #[command(visible_aliases = ["ev"])]
    Evolve(EvolveArgs),

    #[command(about = "Architecture analysis and advice")]
    #[command(visible_aliases = ["a"])]
    Arch(ArchArgs),

    #[command(about = "Manage micro-ADR records")]
    Adr(AdrArgs),

    #[command(about = "Decision records (ADR) management")]
    #[command(visible_aliases = ["dec", "d"])]
    Decision(DecisionArgs),

    #[command(about = "Start web dashboard server")]
    #[command(visible_aliases = ["dash", "web"])]
    Dashboard(DashboardArgs),

    #[command(about = "Test coverage analysis")]
    #[command(visible_aliases = ["tst", "cov"])]
    Test(TestArgs),

    #[command(about = "Fast verification (check + tests + arch rules)")]
    #[command(visible_aliases = ["vfy"])]
    Verify(VerifyArgs),

    #[command(about = "Configuration management")]
    #[command(visible_aliases = ["cfg", "c"])]
    Config(ConfigArgs),

    #[command(about = "Code simplicity and quality lint")]
    #[command(visible_aliases = ["sim", "s"])]
    Lint(LintArgs),

    #[command(about = "Dev workflow - integrated development workflow")]
    #[command(visible_aliases = ["dw", "dev-workflow"])]
    Dev(DevArgs),

    #[command(about = "Template library - best practices and templates")]
    #[command(visible_aliases = ["tpl", "temp"])]
    Template(TemplateArgs),

    #[command(about = "Git integration - git status, log, hooks")]
    #[command(visible_aliases = ["gitops"])]
    Git(GitArgs),

    #[command(about = "Multi-project / Monorepo support")]
    #[command(visible_aliases = ["proj"])]
    Project(ProjectArgs),

    #[command(about = "Performance benchmarking framework")]
    #[command(visible_aliases = ["benchmark"])]
    Bench(BenchArgs),

    #[command(about = "Automated code review (CI/CD)")]
    Review(ReviewArgs),

    #[command(about = "Workflow engine - agent-agnostic workflow protocol")]
    #[command(visible_aliases = ["wf"])]
    Workflow(WorkflowArgs),

    #[command(about = "WebSocket dashboard - real-time updates")]
    #[command(visible_aliases = ["wsd", "websocket"])]
    Ws(WsArgs),

    #[command(about = "Agent protocol - agent collaboration")]
    #[command(visible_aliases = ["ag"])]
    Agent(AgentArgs),

    #[command(about = "Document generation - architecture, API, ADR")]
    #[command(visible_aliases = ["doc"])]
    Docs(DocsArgs),

    #[command(about = "Multi-environment config management")]
    #[command(visible_aliases = ["environments"])]
    Env(EnvArgs),

    #[command(about = "Database migration management")]
    #[command(visible_aliases = ["migrate", "migration"])]
    Db(DbArgs),

    #[command(about = "Enforcement system - hard constraints for architecture")]
    #[command(visible_aliases = ["ef", "guard", "enforce"])]
    Enforcement(EnforcementArgs),

    #[command(about = "Task discovery and management")]
    #[command(visible_aliases = ["t", "tsk"])]
    Task(TaskArgs),

    #[command(about = "Self verification and auto-fix")]
    #[command(visible_aliases = ["sv", "self-check"])]
    SelfVerify(SelfVerifyArgs),

    #[command(about = "Autonomous decision engine")]
    #[command(visible_aliases = ["auto-decide"])]
    Decide(DecideArgs),

    #[command(about = "Tool/MCP whitelist and policy management")]
    #[command(visible_aliases = ["tp"])]
    ToolPolicy(ToolPolicyArgs),

    #[command(about = "Operation audit and traceability")]
    #[command(visible_aliases = ["aud", "log"])]
    Audit(AuditArgs),

    #[command(about = "Agent profile and ranking")]
    #[command(visible_aliases = ["ap", "profile"])]
    AgentProfile(AgentProfileArgs),

    #[command(about = "Self-healing and auto-recovery")]
    #[command(visible_aliases = ["heal", "sh"])]
    SelfHeal(SelfHealArgs),

    #[command(about = "Service mesh integration (Istio config generation, validation, diff)")]
    #[command(visible_aliases = ["sm", "istio"])]
    Mesh(MeshArgs),

    #[command(about = "Plugin security sandbox management")]
    #[command(visible_aliases = ["sb"])]
    Sandbox(SandboxArgs),

    #[command(about = "Enable/disable development helper tools")]
    #[command(visible_aliases = ["devtools"])]
    Tools(ToolsArgs),

    #[command(about = "Saga orchestration and compensation")]
    #[command(visible_aliases = ["saga"])]
    Saga(SagaArgs),

    #[command(about = "Contract testing and validation")]
    #[command(visible_aliases = ["contract"])]
    Contract(ContractArgs),

    #[command(about = "Entropy bank - balance and transactions")]
    #[command(visible_aliases = ["bank"])]
    EntropyBank(EntropyBankArgs),

    #[command(about = "Complexity quota management")]
    #[command(visible_aliases = ["quota"])]
    ComplexityQuota(ComplexityQuotaArgs),

    #[command(about = "Generate shell completions")]
    Completions(CompletionsArgs),

    #[command(about = "Plugin validation and auditing")]
    #[command(visible_aliases = ["pv"])]
    PluginValidate(PluginValidatorArgs),

    #[command(about = "Plugin system management")]
    #[command(visible_aliases = ["plug"])]
    Plugin(PluginArgs),

    #[command(about = "A/B experiment management")]
    #[command(visible_aliases = ["ab", "experiment"])]
    Ab(AbArgs),

    #[command(about = "Canary release management")]
    #[command(visible_aliases = ["can"])]
    Canary(CanaryArgs),

    #[command(about = "Pattern library management")]
    #[command(visible_aliases = ["pat"])]
    Pattern(PatternArgs),

    #[command(about = "Root cause analysis engine")]
    #[command(visible_aliases = ["rca"])]
    Rca(RcaArgs),

    #[command(about = "Business rule engine")]
    #[command(visible_aliases = ["rule"])]
    Rule(RuleArgs),

    #[command(about = "Refactoring assistant")]
    #[command(visible_aliases = ["ref", "refactor"])]
    Refactor(RefactorArgs),
}

#[derive(Debug, clap::Args)]
pub struct InitArgs {
    #[arg(help = "Project name")]
    pub name: Option<String>,

    #[arg(short, long, help = "Path to create project in")]
    pub path: Option<String>,

    #[arg(short, long, help = "Cell template: basic, domain, event-sourced")]
    pub template: Option<String>,

    #[arg(short = 'y', long, help = "Skip prompts, use defaults")]
    pub yes: bool,

    #[arg(long, help = "Force overwrite existing files")]
    pub force: bool,
}

#[derive(Debug, clap::Args)]
pub struct GenerateArgs {
    #[command(subcommand)]
    pub sub: GenerateSub,

    #[arg(long, help = "Disable telemetry instrumentation (metrics, tracing, logs)")]
    pub no_telemetry: bool,
}

#[derive(Debug, Subcommand)]
pub enum GenerateSub {
    #[command(about = "Generate cell from spec or name")]
    Cell {
        #[arg(help = "Cell name or spec file path")]
        name: String,

        #[arg(short, long, help = "Output directory")]
        output: Option<String>,

        #[arg(short, long, help = "Path to spec file (yaml/json/toml)")]
        spec: Option<String>,

        #[arg(long, help = "Force overwrite existing files")]
        force: bool,
    },
    #[command(about = "Generate port/use case interface")]
    Port {
        #[arg(help = "Port name (e.g. CreateUser)")]
        name: String,

        #[arg(long, help = "Port kind: usecase, query, repository, gateway, publisher, subscriber")]
        kind: Option<String>,

        #[arg(short, long, help = "Output directory")]
        output: Option<String>,
    },
    #[command(about = "Generate adapter implementation")]
    Adapter {
        #[arg(help = "Adapter name (e.g. PostgresUserRepo)")]
        name: String,

        #[arg(long, help = "Adapter kind: inmemory, postgres, redis, http, grpc, kafka, nats, file, mock")]
        kind: Option<String>,

        #[arg(long, help = "Port name this adapter implements")]
        port: Option<String>,

        #[arg(short, long, help = "Output directory")]
        output: Option<String>,
    },
    #[command(about = "Generate domain entity")]
    Entity {
        #[arg(help = "Entity name (e.g. User)")]
        name: String,

        #[arg(long, help = "Comma-separated fields: name:type")]
        fields: Option<String>,

        #[arg(short, long, help = "Output directory")]
        output: Option<String>,
    },
    #[command(about = "Generate value object")]
    ValueObject {
        #[arg(help = "ValueObject name (e.g. EmailAddress)")]
        name: String,

        #[arg(long, help = "Comma-separated fields: name:type")]
        fields: Option<String>,

        #[arg(short, long, help = "Output directory")]
        output: Option<String>,
    },
    #[command(about = "Generate aggregate root")]
    Aggregate {
        #[arg(help = "Aggregate name (e.g. Order)")]
        name: String,

        #[arg(short, long, help = "Output directory")]
        output: Option<String>,
    },
    #[command(about = "Generate domain event")]
    DomainEvent {
        #[arg(help = "Event name (e.g. UserCreated)")]
        name: String,

        #[arg(long, help = "Comma-separated fields: name:type")]
        fields: Option<String>,

        #[arg(short, long, help = "Output directory")]
        output: Option<String>,
    },
    #[command(about = "Generate domain service")]
    DomainService {
        #[arg(help = "Service name (e.g. UserRegistrationService)")]
        name: String,

        #[arg(short, long, help = "Output directory")]
        output: Option<String>,
    },
    #[command(about = "Generate repository interface/trait")]
    Repository {
        #[arg(help = "Repository name (e.g. UserRepository)")]
        name: String,

        #[arg(short, long, help = "Entity type this repo manages")]
        entity: String,

        #[arg(short, long, help = "Output directory")]
        output: Option<String>,
    },
    #[command(about = "Generate complete use case (port + input + output + impl)")]
    Usecase {
        #[arg(help = "Use case name (e.g. CreateUser)")]
        name: String,

        #[arg(long, help = "Comma-separated input fields: name:type")]
        input: Option<String>,

        #[arg(long, help = "Comma-separated output fields: name:type")]
        output: Option<String>,

        #[arg(long, help = "Generate implementation skeleton")]
        impl_: bool,

        #[arg(short, long, help = "Output directory")]
        output_dir: Option<String>,
    },
    #[command(about = "Generate K8s deployment manifests (Deployment + Service + HPA)")]
    K8s {
        #[arg(help = "Service name")]
        name: String,

        #[arg(long, help = "Container image")]
        image: String,

        #[arg(long, help = "Container port (default: 8080)")]
        port: Option<u16>,

        #[arg(long, help = "Number of replicas (default: 2)")]
        replicas: Option<u32>,

        #[arg(long, help = "Service type: ClusterIP, NodePort, LoadBalancer")]
        service_type: Option<String>,

        #[arg(long, help = "Kubernetes namespace (default: default)")]
        namespace: Option<String>,

        #[arg(long, help = "Disable HPA")]
        no_hpa: bool,

        #[arg(short, long, help = "Output directory")]
        output: Option<String>,
    },
    #[command(about = "Generate health check handler (axum)")]
    Health {
        #[arg(short, long, help = "Output file path")]
        output: Option<String>,
    },
}

#[derive(Debug, clap::Args)]
pub struct ValidateArgs {
    #[arg(help = "Path to validate")]
    pub path: Option<String>,

    #[arg(short, long, help = "Rule set: basic, strict, custom")]
    pub rules: Option<String>,

    #[arg(long, help = "Output JSON report")]
    pub json: bool,
}

#[derive(Debug, clap::Args)]
pub struct EntropyArgs {
    #[command(subcommand)]
    pub sub: EntropySub,
}

#[derive(Debug, Subcommand)]
pub enum EntropySub {
    #[command(about = "Check current entropy")]
    Check {
        #[arg(help = "Path to analyze")]
        path: Option<String>,
    },
    #[command(about = "Incremental entropy (git-diff based)")]
    Delta {
        #[arg(short, long, help = "Path to analyze")]
        path: Option<String>,

        #[arg(long, help = "Force full scan ignoring cache")]
        full: bool,
    },
    #[command(about = "Entropy gate check for CI")]
    Gate {
        #[arg(short, long, help = "Path to analyze")]
        path: Option<String>,

        #[arg(long, help = "Max allowed entropy score (default: 50.0)")]
        threshold: Option<f64>,
    },
    #[command(about = "Show entropy trend")]
    Trend {},
    #[command(about = "Manage entropy configuration")]
    Config {
        #[command(subcommand)]
        sub: EntropyConfigSub,
    },
}

#[derive(Debug, Subcommand)]
pub enum EntropyConfigSub {
    #[command(about = "Initialize entropy.yaml with defaults")]
    Init {
        #[arg(short, long, help = "Force overwrite existing config")]
        force: bool,
    },
    #[command(about = "Show current entropy configuration")]
    Show {},
}

#[derive(Debug, clap::Args)]
pub struct FeatureArgs {
    #[command(subcommand)]
    pub sub: FeatureSub,
}

#[derive(Debug, Subcommand)]
pub enum FeatureSub {
    #[command(about = "Create a new feature unit")]
    New { 
        #[arg(help = "Feature name")]
        name: String,
        
        #[arg(short, long, help = "Feature description")]
        description: Option<String>,
        
        #[arg(short, long, help = "Owner/responsible agent")]
        owner: Option<String>,
    },
    #[command(about = "Mount a feature unit")]
    Mount { 
        #[arg(help = "Feature name")]
        name: String 
    },
    #[command(about = "Unmount a feature unit")]
    Unmount { 
        #[arg(help = "Feature name")]
        name: String 
    },
    #[command(about = "Analyze feature impact")]
    Impact { 
        #[arg(help = "Feature name")]
        name: String 
    },
    #[command(about = "List all feature units")]
    List {},
    #[command(about = "Feature flag management")]
    Flag {
        #[command(subcommand)]
        action: FlagAction,
    },
}

#[derive(Debug, Subcommand)]
pub enum FlagAction {
    #[command(about = "List all feature flags")]
    List {
        #[arg(long, help = "Filter by flag type: release, ops, experiment, permission")]
        r#type: Option<String>,
    },
    #[command(about = "Enable a feature flag")]
    Enable {
        #[arg(help = "Flag name")]
        name: String,
    },
    #[command(about = "Disable a feature flag")]
    Disable {
        #[arg(help = "Flag name")]
        name: String,
    },
    #[command(about = "Show feature flag details")]
    Show {
        #[arg(help = "Flag name")]
        name: String,
    },
}

#[derive(Debug, clap::Args)]
pub struct DiagnoseArgs {
    #[command(subcommand)]
    pub sub: DiagnoseSub,
}

#[derive(Debug, Subcommand)]
pub enum DiagnoseSub {
    #[command(about = "Scan code and diagnose issues")]
    Scan {
        #[arg(short, long, help = "Path to scan (default: current directory)")]
        path: Option<String>,

        #[arg(long, help = "Filter by category: architecture, performance, security, maintainability, testing, dependency, configuration")]
        category: Option<String>,

        #[arg(long, help = "Filter by severity: critical, high, medium, low, info")]
        severity: Option<String>,
    },
    #[command(about = "List all problem fingerprints")]
    List {
        #[arg(long, help = "Filter by category")]
        category: Option<String>,

        #[arg(long, help = "Filter by severity")]
        severity: Option<String>,
    },
    #[command(about = "Show fingerprint details")]
    Detail {
        #[arg(help = "Fingerprint ID (e.g. FP001)")]
        id: String,
    },
    #[command(about = "Diagnose from error message")]
    Error {
        #[arg(help = "Error message to match")]
        message: String,
    },
    #[command(about = "Apply fix suggestion for a fingerprint")]
    Fix {
        #[arg(help = "Fingerprint ID (e.g. FP001)")]
        id: String,
    },
}

#[derive(Debug, clap::Args)]
pub struct EventArgs {
    #[command(subcommand)]
    pub sub: EventSub,
}

#[derive(Debug, Subcommand)]
pub enum EventSub {
    #[command(about = "List all event schemas")]
    List {
        #[arg(long, help = "Event name filter")]
        name: Option<String>,
    },
    #[command(about = "Show event schema details")]
    Show {
        #[arg(help = "Event schema name")]
        name: String,

        #[arg(long, help = "Specific version")]
        version: Option<String>,
    },
    #[command(about = "Check compatibility between two schema versions")]
    Check {
        #[arg(help = "Event schema name")]
        name: String,

        #[arg(long, help = "Old version file path")]
        old: String,

        #[arg(long, help = "New version file path")]
        new: String,
    },
    #[command(about = "Generate proto file from schema")]
    Proto {
        #[arg(help = "Path to schema YAML/JSON file")]
        input: String,

        #[arg(short, long, help = "Output proto file path")]
        output: Option<String>,
    },
    #[command(about = "Generate JSON Schema from event schema")]
    JsonSchema {
        #[arg(help = "Path to schema YAML/JSON file")]
        input: String,

        #[arg(short, long, help = "Output file path")]
        output: Option<String>,
    },
}

#[derive(Debug, clap::Args)]
pub struct HandoffArgs {
    #[command(subcommand)]
    pub sub: HandoffSub,
}

#[derive(Debug, Subcommand)]
pub enum HandoffSub {
    #[command(about = "Generate handoff package (json + md)")]
    Generate {
        #[arg(short, long, help = "Project name")]
        name: Option<String>,

        #[arg(short, long, help = "Output directory")]
        output: Option<String>,

        #[arg(short, long, help = "Author/agent name")]
        author: Option<String>,

        #[arg(long, help = "Skip markdown output")]
        no_md: bool,

        #[arg(long, help = "Skip json output")]
        no_json: bool,
    },
    #[command(about = "Show handoff package content")]
    Show {
        #[arg(help = "Path to handoff JSON file")]
        path: Option<String>,
    },
    #[command(about = "Validate handoff package completeness")]
    Validate {
        #[arg(help = "Path to handoff JSON file")]
        path: String,
    },
    #[command(about = "Auto-commit: generate handoff + verify + entropy check + git push")]
    Commit {
        #[arg(short, long, help = "Commit message")]
        message: Option<String>,

        #[arg(short, long, help = "Author/agent name")]
        author: Option<String>,

        #[arg(long, help = "Skip deep verification (quick mode)")]
        quick: bool,

        #[arg(long, help = "Skip git push")]
        no_push: bool,
    },
}

#[derive(Debug, clap::Args)]
pub struct ProgressArgs {
    #[command(subcommand)]
    pub sub: ProgressSub,
}

#[derive(Debug, Subcommand)]
pub enum ProgressSub {
    #[command(about = "Start a new task")]
    Start {
        #[arg(help = "Task name")]
        name: String,

        #[arg(short, long, help = "Task description")]
        description: Option<String>,

        #[arg(short, long, help = "Assignee name")]
        assignee: Option<String>,
    },
    #[command(about = "Show current task status")]
    Status {
        #[arg(short, long, help = "Show full timeline")]
        timeline: bool,
    },
    #[command(about = "Log a progress event")]
    Log {
        #[arg(help = "Event message")]
        message: String,

        #[arg(short, long, help = "Event type: start, update, decision, blocker, note, complete")]
        kind: Option<String>,

        #[arg(short, long, help = "Additional details")]
        details: Option<String>,
    },
    #[command(about = "Add a blocker")]
    Block {
        #[arg(help = "Blocker description")]
        description: String,
    },
    #[command(about = "Resolve a blocker")]
    Unblock {
        #[arg(help = "Blocker ID")]
        id: String,

        #[arg(short, long, help = "Resolution description")]
        resolution: String,
    },
    #[command(about = "Add next step")]
    Next {
        #[arg(help = "Step description")]
        description: String,

        #[arg(short, long, default_value_t = 1, help = "Priority (1-5)")]
        priority: u8,

        #[arg(short, long, help = "Estimated minutes")]
        minutes: Option<u32>,
    },
    #[command(about = "Complete a next step")]
    Done {
        #[arg(help = "Step ID")]
        id: String,
    },
    #[command(about = "Mark current task complete")]
    Complete {},
    #[command(about = "Show task history")]
    History {},
    #[command(about = "Add related file")]
    File {
        #[arg(help = "File path")]
        path: String,
    },
}

#[derive(Debug, clap::Args)]
pub struct EvolveArgs {
    #[command(subcommand)]
    pub sub: EvolveSub,
}

#[derive(Debug, Subcommand)]
pub enum EvolveSub {
    #[command(about = "Start a new evolution cycle")]
    Cycle {
        #[command(subcommand)]
        action: CycleAction,
    },
    #[command(about = "Report an issue/pain point")]
    Issue {
        #[arg(help = "Issue title")]
        title: String,

        #[arg(short, long, help = "Issue description")]
        description: Option<String>,

        #[arg(short, long, help = "Category: process, tool, quality, handoff, codegen, docs, arch, entropy, test, other")]
        category: Option<String>,

        #[arg(short, long, help = "Severity: critical, high, medium, low, trivial")]
        severity: Option<String>,
    },
    #[command(about = "Add an improvement proposal")]
    Add {
        #[arg(help = "Improvement title")]
        title: String,

        #[arg(short, long, help = "Improvement description")]
        description: Option<String>,

        #[arg(short, long, help = "Category")]
        category: Option<String>,

        #[arg(short, long, help = "Impact: transformational, high, medium, low, minimal")]
        impact: Option<String>,

        #[arg(short, long, help = "Effort: minutes, hours, days, weeks")]
        effort: Option<String>,
    },
    #[command(about = "Generate AI-powered improvement suggestions")]
    Suggest {
        #[arg(short, long, help = "Apply suggestions automatically")]
        apply: bool,
    },
    #[command(about = "Apply an improvement")]
    Apply {
        #[arg(help = "Improvement ID")]
        id: String,

        #[arg(short, long, help = "Applied by (agent name)")]
        by: Option<String>,
    },
    #[command(about = "Show current cycle status")]
    Status {},
    #[command(about = "Show evolution history")]
    History {},
    #[command(about = "Show evolution summary stats")]
    Stats {},

    #[command(about = "Auto-diagnose project issues")]
    Scan {},
}

#[derive(Debug, Subcommand)]
pub enum CycleAction {
    #[command(about = "Start a new evolution cycle")]
    Start,
    #[command(about = "Complete current cycle")]
    Complete,
}

#[derive(Debug, clap::Args)]
pub struct ArchArgs {
    #[command(subcommand)]
    pub sub: ArchSub,
}

#[derive(Debug, Subcommand)]
pub enum ArchSub {
    #[command(about = "Validate architecture rules")]
    Validate {
        #[arg(short, long)]
        path: Option<String>,
    },
    #[command(about = "Visualize architecture layers")]
    Visualize {
        #[arg(short, long)]
        output: Option<String>,
    },
    #[command(about = "Architecture lint with auto-fix")]
    Lint {
        #[arg(long, help = "Auto-fix fixable violations")]
        fix: bool,

        #[arg(long, help = "Deep scan mode")]
        deep: bool,

        #[arg(long, help = "Output JSON format")]
        json: bool,
    },
    #[command(about = "List all lint rules")]
    Rules {},
    #[command(about = "Show architecture overview")]
    Overview {},
    #[command(about = "Show dependency graph")]
    Graph {},
    #[command(about = "Impact analysis of code changes")]
    Impact {
        #[arg(short, long, help = "Base ref (commit/branch) to compare against")]
        base: Option<String>,

        #[arg(short, long, help = "Path to project")]
        path: Option<String>,
    },
    #[command(about = "Architecture advice")]
    Advise {},
}

#[derive(Debug, clap::Args)]
pub struct AdrArgs {
    #[command(subcommand)]
    pub sub: AdrSub,
}

#[derive(Debug, Subcommand)]
pub enum AdrSub {
    #[command(about = "Create new micro-ADR")]
    New { 
        #[arg(help = "ADR title")]
        title: String,
        
        #[arg(long, help = "Status: proposed, accepted, deprecated, rejected, superseded")]
        status: Option<String>,
    },
    #[command(about = "List ADRs")]
    List {},
    #[command(about = "Show ADR details")]
    Show { 
        #[arg(help = "ADR ID")]
        id: String 
    },
}

#[derive(Debug, clap::Args)]
pub struct DecisionArgs {
    #[command(subcommand)]
    pub sub: DecisionSub,
}

#[derive(Debug, Subcommand)]
pub enum DecisionSub {
    #[command(about = "Record a new decision")]
    New {
        #[arg(help = "Decision title")]
        title: String,

        #[arg(short, long, help = "Context/background")]
        context: Option<String>,

        #[arg(short, long, help = "Decision content")]
        decision: Option<String>,

        #[arg(short, long, help = "Rationale/reasoning")]
        rationale: Option<String>,

        #[arg(short, long, help = "Category: arch, tech, process, tool, design, test, deploy, security, perf, other")]
        category: Option<String>,

        #[arg(short, long, help = "Made by (agent name)")]
        by: Option<String>,
    },
    #[command(about = "List all decisions")]
    List {
        #[arg(short, long, help = "Filter by category")]
        category: Option<String>,

        #[arg(short, long, help = "Filter by status")]
        status: Option<String>,
    },
    #[command(about = "Show decision details")]
    Show {
        #[arg(help = "Decision ID")]
        id: String,
    },
    #[command(about = "Update decision status")]
    Status {
        #[arg(help = "Decision ID")]
        id: String,

        #[arg(help = "New status: proposed, accepted, rejected, deprecated, superseded")]
        status: String,
    },
    #[command(about = "Add alternative to decision")]
    Alternative {
        #[arg(help = "Decision ID")]
        id: String,

        #[arg(help = "Alternative name")]
        name: String,

        #[arg(short, long, help = "Description")]
        description: Option<String>,
    },
    #[command(about = "Add tag to decision")]
    Tag {
        #[arg(help = "Decision ID")]
        id: String,

        #[arg(help = "Tag name")]
        tag: String,
    },
    #[command(about = "Export decisions to markdown")]
    Export {
        #[arg(short, long, help = "Output file path")]
        output: Option<String>,
    },
    #[command(about = "Show decision metrics")]
    Metrics {},
}

#[derive(Debug, clap::Args)]
pub struct TestArgs {
    #[command(subcommand)]
    pub sub: TestSub,
}

#[derive(Debug, Subcommand)]
pub enum TestSub {
    #[command(about = "Show test coverage report")]
    Coverage {
        #[arg(short, long, help = "Path to project")]
        path: Option<String>,
    },
    #[command(about = "Show files missing tests")]
    Missing {
        #[arg(short, long, help = "Path to project")]
        path: Option<String>,
    },
}

#[derive(Debug, clap::Args)]
pub struct DashboardArgs {
    #[arg(short = 'P', long, default_value_t = 8080, help = "Port number")]
    pub port: u16,

    #[arg(short, long, help = "Path to project")]
    pub path: Option<String>,

    #[arg(short = 'n', long, help = "Do not open browser")]
    pub no_open: bool,
}

#[derive(Debug, clap::Args)]
pub struct VerifyArgs {
    #[arg(short, long, help = "Path to project")]
    pub path: Option<String>,

    #[arg(short, long, help = "Deep check (full test suite + entropy gate)")]
    pub deep: bool,
}

#[derive(Debug, clap::Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub sub: ConfigSub,
}

#[derive(Debug, Subcommand)]
pub enum ConfigSub {
    #[command(about = "Show current configuration")]
    Show {
        #[arg(short, long, help = "Path to project")]
        path: Option<String>,
    },
    #[command(about = "Get a config value")]
    Get {
        key: String,

        #[arg(short, long, help = "Path to project")]
        path: Option<String>,
    },
    #[command(about = "Set a config value")]
    Set {
        key: String,
        value: String,

        #[arg(short, long, help = "Path to project")]
        path: Option<String>,
    },
    #[command(about = "Initialize default config file")]
    Init {
        #[arg(short, long, help = "Path to project")]
        path: Option<String>,

        #[arg(short, long, help = "Overwrite existing config")]
        force: bool,
    },
    #[command(about = "Validate cell.yaml configuration")]
    Validate {
        #[arg(short, long, help = "Path to cell.yaml or project directory")]
        path: Option<String>,
    },
}

#[derive(Debug, clap::Args)]
pub struct LintArgs {
    #[arg(short, long, help = "Path to project")]
    pub path: Option<String>,

    #[arg(short, long, help = "Strict mode - all thresholds tightened")]
    pub strict: bool,

    #[arg(short, long, help = "Max file lines threshold")]
    pub max_file_lines: Option<usize>,

    #[arg(short = 'F', long, help = "Max function lines threshold")]
    pub max_fn_lines: Option<usize>,
}

#[derive(Debug, clap::Args)]
pub struct ToolsArgs {
    #[command(subcommand)]
    pub sub: ToolsSub,
}

#[derive(Debug, Subcommand)]
pub enum ToolsSub {
    #[command(about = "Enable all helper tools (progress, decisions, evolution, dashboard)")]
    Enable {
        #[arg(short, long, help = "Path to project")]
        path: Option<String>,
    },
    #[command(about = "Show tools status")]
    Status {
        #[arg(short, long, help = "Path to project")]
        path: Option<String>,
    },
}

#[derive(Debug, clap::Args)]
pub struct SagaArgs {
    #[command(subcommand)]
    pub sub: SagaSub,
}

#[derive(Debug, Subcommand)]
pub enum SagaSub {
    #[command(about = "Create a new saga")]
    Create {
        #[arg(help = "Saga name")]
        name: String,
    },
    #[command(about = "List all sagas")]
    List {},
}

#[derive(Debug, clap::Args)]
pub struct ContractArgs {
    #[command(subcommand)]
    pub sub: ContractSub,
}

#[derive(Debug, Subcommand)]
pub enum ContractSub {
    #[command(about = "Create a new contract")]
    Create {
        #[arg(help = "Contract ID")]
        id: String,
        #[arg(help = "Provider")]
        provider: String,
        #[arg(help = "Consumer")]
        consumer: String,
        #[arg(help = "Port")]
        port: String,
    },
    #[command(about = "List all contracts")]
    List {},
}

#[derive(Debug, clap::Args)]
pub struct EntropyBankArgs {
    #[command(subcommand)]
    pub sub: EntropyBankSub,
}

#[derive(Debug, Subcommand)]
pub enum EntropyBankSub {
    #[command(about = "Show account balance")]
    Balance {
        #[arg(help = "Owner")]
        owner: String,
    },
    #[command(about = "Deposit entropy")]
    Deposit {
        #[arg(help = "Owner")]
        owner: String,
        #[arg(help = "Amount")]
        amount: f64,
        #[arg(help = "Reason")]
        reason: String,
    },
    #[command(about = "Withdraw entropy")]
    Withdraw {
        #[arg(help = "Owner")]
        owner: String,
        #[arg(help = "Amount")]
        amount: f64,
        #[arg(help = "Reason")]
        reason: String,
    },
}

#[derive(Debug, clap::Args)]
pub struct ComplexityQuotaArgs {
    #[command(subcommand)]
    pub sub: ComplexityQuotaSub,
}

#[derive(Debug, Subcommand)]
pub enum ComplexityQuotaSub {
    #[command(about = "Show quota status")]
    Status {
        #[arg(help = "Quota name")]
        name: String,
    },
    #[command(about = "Check quota")]
    Check {
        #[arg(help = "Quota name")]
        name: String,
        #[arg(help = "Required complexity")]
        required: f64,
    },
}

#[derive(Debug, clap::Args)]
pub struct CompletionsArgs {
    #[arg(help = "Shell: bash, zsh, fish, powershell, elvish")]
    pub shell: clap_complete::Shell,
}

#[derive(Debug, clap::Args)]
pub struct DevArgs {
    #[command(subcommand)]
    pub sub: DevSub,
}

#[derive(Debug, Subcommand)]
pub enum DevSub {
    #[command(about = "Bootstrap development environment (one-click ready)")]
    Bootstrap {},

    #[command(about = "Doctor - check development environment")]
    Doctor {},

    #[command(about = "Start a new development task")]
    Start {
        #[arg(help = "Task name")]
        name: String,

        #[arg(short, long, help = "Task description")]
        description: Option<String>,
    },
    #[command(about = "Design phase - architecture analysis")]
    Design {},
    #[command(about = "Code checkpoint - quality & entropy check")]
    Checkpoint {
        #[arg(help = "Checkpoint message")]
        message: Option<String>,
    },
    #[command(about = "Verify phase - full validation")]
    Verify {
        #[arg(short, long, help = "Deep verification (full tests + entropy gate)")]
        deep: bool,
    },
    #[command(about = "Handoff phase - generate handoff package")]
    Handoff {
        #[arg(short, long, help = "Handoff message")]
        message: Option<String>,
    },
    #[command(about = "Show current environment status")]
    Status {},
    #[command(about = "Record a development decision")]
    Decision {
        #[arg(help = "Decision title")]
        title: String,

        #[arg(short, long, help = "Context/background")]
        context: Option<String>,

        #[arg(short, long, help = "The decision")]
        decision: Option<String>,
    },
    #[command(about = "Show next step suggestions")]
    Next {},

    #[command(about = "Generate context snapshot for new agents")]
    Context {},

    #[command(about = "Reset development environment")]
    Reset {
        #[arg(short, long, help = "Reset scope: all, agent, progress")]
        scope: Option<String>,
    },
}

#[derive(Debug, clap::Args)]
pub struct WorkflowArgs {
    #[command(subcommand)]
    pub sub: WorkflowSub,

    #[arg(short, long, help = "Agent ID")]
    pub agent_id: Option<String>,

    #[arg(short, long, help = "Output format: text, json, yaml")]
    pub format: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum WorkflowSub {
    #[command(about = "Start a new workflow")]
    Start {
        #[arg(help = "Task name")]
        name: String,

        #[arg(short, long, help = "Task description")]
        description: Option<String>,
    },
    #[command(about = "Show workflow status")]
    Status {},
    #[command(about = "Run phase gate checks")]
    Check {},
    #[command(about = "Advance to next phase")]
    Advance {},
    #[command(about = "Show next phase info")]
    Next {},
    #[command(about = "List all phases and gates")]
    Phases {},
    #[command(about = "Register an agent")]
    Register {
        #[arg(help = "Agent ID")]
        id: String,

        #[arg(short, long, help = "Agent name")]
        name: Option<String>,

        #[arg(short, long, help = "Agent version")]
        version: Option<String>,
    },
    #[command(about = "Record a gate result")]
    Gate {
        #[arg(help = "Gate name")]
        name: String,

        #[arg(short, long, help = "Gate passed")]
        passed: bool,

        #[arg(short, long, help = "Detail message")]
        detail: Option<String>,
    },
    #[command(about = "Execute a raw workflow command (JSON)")]
    Exec {
        #[arg(help = "JSON command")]
        command: String,
    },
    #[command(about = "Abort current workflow")]
    Abort {},
}

#[derive(Debug, clap::Args)]
pub struct TemplateArgs {
    #[command(subcommand)]
    pub sub: TemplateSub,
}

#[derive(Debug, Subcommand)]
pub enum TemplateSub {
    #[command(about = "List available templates")]
    List {
        #[arg(short, long, help = "Filter by category")]
        category: Option<String>,
    },
    #[command(about = "Show template details")]
    Show {
        #[arg(help = "Template ID")]
        id: String,
    },
    #[command(about = "Apply a template to project")]
    Apply {
        #[arg(help = "Template ID")]
        id: String,

        #[arg(short, long, help = "Target path")]
        path: Option<String>,

        #[arg(short, long = "var", help = "Variable key=value")]
        var: Vec<String>,

        #[arg(short, long, help = "Overwrite existing files")]
        force: bool,
    },
    #[command(about = "List template categories")]
    Categories {},
}

#[derive(Debug, clap::Args)]
pub struct GitArgs {
    #[command(subcommand)]
    pub sub: GitSub,
}

#[derive(Debug, Subcommand)]
pub enum GitSub {
    #[command(about = "Show git status")]
    Status {},
    #[command(about = "List branches")]
    Branches {},
    #[command(about = "Show commit log")]
    Log {
        #[arg(short, long, help = "Number of commits")]
        count: Option<usize>,
    },
    #[command(about = "Show diff stats")]
    Diff {
        #[arg(help = "Diff target (branch, commit, etc.)")]
        target: Option<String>,
    },
    #[command(about = "Git hook management")]
    Hooks {
        #[arg(long, help = "Install git hooks")]
        install: bool,
    },
}

#[derive(Debug, clap::Args)]
pub struct ProjectArgs {
    #[command(subcommand)]
    pub sub: ProjectSub,
}

#[derive(Debug, Subcommand)]
pub enum ProjectSub {
    #[command(about = "List all projects")]
    List {},
    #[command(about = "Show current project")]
    Current {},
    #[command(about = "Switch to a project")]
    Switch { name: String },
    #[command(about = "Add a new project")]
    Add {
        name: String,
        #[arg(short, long, help = "Project path")]
        path: String,
        #[arg(short, long, help = "Project description")]
        description: Option<String>,
    },
    #[command(about = "Remove a project")]
    Remove { name: String },
}

#[derive(Debug, clap::Args)]
pub struct BenchArgs {
    #[command(subcommand)]
    pub sub: BenchSub,
}

#[derive(Debug, Subcommand)]
pub enum BenchSub {
    #[command(about = "Run all benchmarks")]
    Run {},
    #[command(about = "Compare with baseline")]
    Compare {
        #[arg(help = "Benchmark name")]
        name: String,
    },
    #[command(about = "List available benchmarks")]
    List {},
}

#[derive(Debug, clap::Args)]
pub struct ReviewArgs {
    #[arg(short, long, help = "Deep review mode")]
    pub deep: bool,
}

#[derive(Debug, clap::Args)]
pub struct WsArgs {
    #[command(subcommand)]
    pub sub: WsSub,
}

#[derive(Debug, Subcommand)]
pub enum WsSub {
    #[command(about = "Serve WebSocket dashboard")]
    Serve {
        #[arg(short, long, default_value = "3000", help = "Port")]
        port: u16,
    },
    #[command(about = "Show WebSocket HTML")]
    Html {},
    #[command(about = "Test WebSocket messages")]
    Test {},
}

#[derive(Debug, clap::Args)]
pub struct AgentArgs {
    #[command(subcommand)]
    pub sub: AgentSub,
}

#[derive(Debug, Subcommand)]
pub enum AgentSub {
    #[command(about = "Register a new agent")]
    Register {
        #[arg(help = "Agent name")]
        name: String,
        #[arg(short, long, default_value = "developer", help = "Agent role")]
        role: String,
    },
    #[command(about = "List all agents")]
    List {},
    #[command(about = "Show agent status")]
    Status {
        #[arg(help = "Agent ID")]
        id: String,
    },
    #[command(about = "Task management")]
    Task {
        #[command(subcommand)]
        sub: AgentTaskSub,
    },
    #[command(about = "Delegate task to agent")]
    Delegate {
        #[arg(help = "Task ID")]
        task_id: String,
        #[arg(help = "Agent ID")]
        agent_id: String,
    },
    #[command(about = "Create handoff package")]
    Handoff {
        #[arg(help = "From agent ID")]
        from: String,
        #[arg(help = "To agent ID")]
        to: Option<String>,
        #[arg(help = "Task ID")]
        task_id: String,
        #[arg(short, long, default_value = "", help = "Notes")]
        notes: String,
    },
    #[command(about = "Send heartbeat")]
    Heartbeat {
        #[arg(help = "Agent ID")]
        id: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum AgentTaskSub {
    #[command(about = "Create a new task")]
    Create {
        #[arg(help = "Task name")]
        name: String,
        #[arg(short, long, help = "Description")]
        description: String,
    },
    #[command(about = "Assign task to agent")]
    Assign {
        #[arg(help = "Task ID")]
        task_id: String,
        #[arg(help = "Agent ID")]
        agent_id: String,
    },
    #[command(about = "Complete a task")]
    Complete {
        #[arg(help = "Task ID")]
        task_id: String,
        #[arg(long, default_value = "true", help = "Success")]
        success: bool,
    },
    #[command(about = "List all tasks")]
    List {},
}

#[derive(Debug, clap::Args)]
pub struct DocsArgs {
    #[command(subcommand)]
    pub sub: DocsSub,

    #[arg(short, long, default_value = "markdown", help = "Format: markdown, html, pdf, openapi, all")]
    pub format: String,

    #[arg(short, long, default_value = "docs", help = "Output directory")]
    pub output: String,

    #[arg(long, help = "Include private items")]
    pub include_private: bool,

    #[arg(long, help = "Include test code")]
    pub include_tests: bool,
}

#[derive(Debug, Subcommand)]
pub enum DocsSub {
    #[command(about = "Generate all documentation")]
    Generate {},
    #[command(about = "Generate architecture docs")]
    Architecture {},
    #[command(about = "Generate API docs")]
    Api {},
    #[command(about = "Generate decision docs")]
    Decisions {},
    #[command(about = "Serve documentation")]
    Serve {
        #[arg(short, long, default_value = "8080", help = "Port")]
        port: u16,
    },
}

#[derive(Debug, clap::Args)]
pub struct EnvArgs {
    #[command(subcommand)]
    pub sub: EnvSub,
}

#[derive(Debug, Subcommand)]
pub enum EnvSub {
    #[command(about = "Create an environment")]
    Create {
        #[arg(help = "Environment name (dev, staging, prod, custom)")]
        name: String,
    },
    #[command(about = "List all environments")]
    List {},
    #[command(about = "Set config value")]
    Set {
        #[arg(help = "Environment name")]
        env: String,
        #[arg(help = "Config key")]
        key: String,
        #[arg(help = "Config value")]
        value: String,
    },
    #[command(about = "Get config value")]
    Get {
        #[arg(help = "Environment name")]
        env: String,
        #[arg(help = "Config key")]
        key: String,
    },
    #[command(about = "Diff two environments")]
    Diff {
        #[arg(help = "Base environment")]
        base: String,
        #[arg(help = "Target environment")]
        target: String,
    },
    #[command(about = "Detect config drift")]
    Drift {},
    #[command(about = "Sync config between environments")]
    Sync {
        #[arg(help = "From environment")]
        from: String,
        #[arg(help = "To environment")]
        to: String,
    },
}

#[derive(Debug, clap::Args)]
pub struct DbArgs {
    #[command(subcommand)]
    pub sub: DbSub,
}

#[derive(Debug, Subcommand)]
pub enum DbSub {
    #[command(about = "Create a migration")]
    Create {
        #[arg(help = "Migration name")]
        name: String,
        #[arg(short, long, default_value = "", help = "Description")]
        description: String,
    },
    #[command(about = "List all migrations")]
    List {},
    #[command(about = "Apply pending migrations")]
    Migrate {
        #[arg(help = "Target version")]
        version: Option<String>,
    },
    #[command(about = "Rollback migrations")]
    Rollback {
        #[arg(help = "Target version")]
        version: String,
    },
    #[command(about = "Show migration status")]
    Status {},
    #[command(about = "Validate a migration")]
    Validate {
        #[arg(help = "Migration ID")]
        id: String,
    },
    #[command(about = "Detect schema drift")]
    Drift {
        #[arg(help = "Actual schema content")]
        schema: String,
    },
}

#[derive(Debug, clap::Args)]
pub struct EnforcementArgs {
    #[command(subcommand)]
    pub sub: EnforcementSub,
}

#[derive(Debug, Subcommand)]
pub enum EnforcementSub {
    #[command(about = "Show enforcement status and config")]
    Status {},
    #[command(about = "Run pre-commit checks")]
    PreCommit {},
    #[command(about = "Run pre-push checks")]
    PrePush {},
    #[command(about = "Run build guard checks")]
    BuildGuard {},
    #[command(about = "Install git hooks")]
    InstallHooks {},
    #[command(about = "Uninstall git hooks")]
    UninstallHooks {},
    #[command(about = "Set policy level")]
    SetPolicy {
        #[arg(help = "Policy name")]
        policy: String,
        #[arg(help = "Level: allow, warn, block")]
        level: String,
    },
    #[command(about = "Enable enforcement")]
    Enable {},
    #[command(about = "Disable enforcement")]
    Disable {},
    #[command(about = "CI/CD template management")]
    Ci {
        #[command(subcommand)]
        sub: CiSub,
    },
}

#[derive(Debug, Subcommand)]
pub enum CiSub {
    #[command(about = "Generate CI template")]
    Generate {
        #[arg(short, long, default_value = "all", help = "Provider: github, gitlab, jenkins, gitee, all")]
        provider: String,
    },
    #[command(about = "Apply CI template to project")]
    Apply {
        #[arg(short, long, default_value = "all", help = "Provider")]
        provider: String,
    },
    #[command(about = "List supported CI providers")]
    List {},
}

#[derive(Debug, clap::Args)]
pub struct TaskArgs {
    #[command(subcommand)]
    pub sub: TaskSub,

    #[arg(short, long, help = "Filter by priority: p0, p1, p2, p3")]
    pub priority: Option<String>,

    #[arg(short, long, help = "Filter by status: pending, in_progress, done, blocked")]
    pub status: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum TaskSub {
    #[command(about = "List all tasks")]
    List {},
    #[command(about = "Discover tasks from roadmap, todos, issues")]
    Discover {},
    #[command(about = "Show next recommended task")]
    Next {},
    #[command(about = "Show task details")]
    Show {
        #[arg(help = "Task ID")]
        id: String,
    },
    #[command(about = "Claim/start a task")]
    Claim {
        #[arg(help = "Task ID")]
        id: String,
    },
    #[command(about = "Complete a task")]
    Done {
        #[arg(help = "Task ID")]
        id: String,
    },
}

#[derive(Debug, clap::Args)]
pub struct SelfVerifyArgs {
    #[arg(short, long, default_value = "3", help = "Max retry attempts")]
    pub attempts: u32,

    #[arg(long, help = "Skip architecture check")]
    pub no_arch: bool,

    #[arg(long, help = "Skip tests")]
    pub no_tests: bool,

    #[arg(long, help = "Skip entropy check")]
    pub no_entropy: bool,

    #[arg(long, help = "Disable auto-fix")]
    pub no_fix: bool,

    #[arg(long, help = "Rollback to stable on failure")]
    pub rollback: bool,
}

#[derive(Debug, clap::Args)]
pub struct DecideArgs {
    #[command(subcommand)]
    pub sub: DecideSub,

    #[arg(short, long, help = "Agent ID")]
    pub agent: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum DecideSub {
    #[command(about = "Make an autonomous decision")]
    Make {
        #[arg(help = "Decision title")]
        title: String,

        #[arg(short, long, help = "Context/background")]
        context: Option<String>,
    },
    #[command(about = "List all decisions")]
    List {
        #[arg(long, help = "Only show pending review")]
        pending: bool,
    },
    #[command(about = "Show decision details")]
    Show {
        #[arg(help = "Decision ID")]
        id: String,
    },
    #[command(about = "List decision rules")]
    Rules {},
}

#[derive(Debug, clap::Args)]
pub struct ToolPolicyArgs {
    #[command(subcommand)]
    pub sub: ToolPolicySub,

    #[arg(short, long, help = "Filter by role: architect, developer, tester, reviewer, observer")]
    pub role: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum ToolPolicySub {
    #[command(about = "List all available tools")]
    List {},
    #[command(about = "Show tool details")]
    Show {
        #[arg(help = "Tool ID")]
        id: String,
    },
    #[command(about = "Check if agent can use a tool")]
    Check {
        #[arg(help = "Tool ID")]
        tool_id: String,
        #[arg(short, long, help = "Agent role")]
        role: String,
    },
    #[command(about = "Record tool usage")]
    Record {
        #[arg(help = "Tool ID")]
        tool_id: String,
        #[arg(short, long, help = "Agent ID")]
        agent: String,
        #[arg(long, help = "Duration in ms")]
        duration: Option<u64>,
        #[arg(long, help = "Success/failure")]
        success: bool,
    },
}

#[derive(Debug, clap::Args)]
pub struct AuditArgs {
    #[command(subcommand)]
    pub sub: AuditSub,

    #[arg(short, long, help = "Agent ID filter")]
    pub agent: Option<String>,

    #[arg(short, long, help = "Max results")]
    pub limit: Option<usize>,
}

#[derive(Debug, Subcommand)]
pub enum AuditSub {
    #[command(about = "Show audit log")]
    Log {},
    #[command(about = "Query audit logs")]
    Query {
        #[arg(long, help = "Action type: tool_call, file_read, file_write, etc.")]
        action: Option<String>,
        #[arg(long, help = "Result: success, failure, blocked, warning")]
        result: Option<String>,
    },
    #[command(about = "Trace a file to find who modified it")]
    Trace {
        #[arg(help = "File path")]
        file: String,
    },
}

#[derive(Debug, clap::Args)]
pub struct AgentProfileArgs {
    #[command(subcommand)]
    pub sub: AgentProfileSub,

    #[arg(short, long, help = "Agent ID")]
    pub agent: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum AgentProfileSub {
    #[command(about = "Show agent profile")]
    Show {
        #[arg(help = "Agent ID")]
        id: Option<String>,
    },
    #[command(about = "List all agent profiles")]
    List {},
    #[command(about = "Show agent ranking")]
    Rank {},
    #[command(about = "Record task completion for stats")]
    Record {
        #[arg(help = "Agent ID")]
        agent: String,
        #[arg(long, help = "Task was successful")]
        success: bool,
        #[arg(long, help = "Task was on time")]
        on_time: bool,
        #[arg(long, help = "Duration in minutes")]
        duration: Option<u64>,
    },
}

#[derive(Debug, clap::Args)]
pub struct SelfHealArgs {
    #[command(subcommand)]
    pub sub: SelfHealSub,
}

#[derive(Debug, Subcommand)]
pub enum SelfHealSub {
    #[command(about = "Show healing status and report")]
    Status {},
    #[command(about = "Detect anomalies")]
    Detect {},
    #[command(about = "Attempt recovery for an anomaly")]
    Recover {
        #[arg(help = "Anomaly ID")]
        id: String,
    },
    #[command(about = "Report a new anomaly")]
    Report {
        #[arg(help = "Anomaly description")]
        description: String,
        #[arg(short, long, help = "Severity: info, warning, critical, fatal")]
        severity: Option<String>,
        #[arg(short, long, help = "Agent ID")]
        agent: Option<String>,
    },
    #[command(about = "Generate human intervention report")]
    Escalate {
        #[arg(help = "Anomaly ID")]
        id: String,
    },
}

#[derive(Debug, clap::Args)]
pub struct SandboxArgs {
    #[command(subcommand)]
    pub sub: SandboxSub,
}

#[derive(Debug, Subcommand)]
pub enum SandboxSub {
    #[command(about = "Create a new plugin sandbox")]
    Create {
        #[arg(help = "Sandbox name")]
        name: String,
    },
    #[command(about = "List all sandboxes")]
    List {},
    #[command(about = "Show sandbox resource limits")]
    Limits {
        #[arg(help = "Sandbox name")]
        name: String,
    },
    #[command(about = "Execute command in sandbox")]
    Exec {
        #[arg(help = "Sandbox name")]
        name: String,
        #[arg(help = "Command to execute (use -- to separate from cell args)")]
        cmd: Vec<String>,
    },
}

#[derive(Debug, clap::Args)]
pub struct PluginValidatorArgs {
    #[command(subcommand)]
    pub sub: PluginValidatorSub,
}

#[derive(Debug, Subcommand)]
pub enum PluginValidatorSub {
    #[command(about = "Validate a plugin manifest")]
    Validate {
        #[arg(help = "Path to plugin manifest file or directory")]
        path: Option<String>,

        #[arg(long, help = "Output JSON format")]
        json: bool,
    },
    #[command(about = "Scan directory for plugins")]
    Scan {
        #[arg(help = "Path to scan for plugins")]
        path: Option<String>,

        #[arg(long, help = "Output JSON format")]
        json: bool,
    },
    #[command(about = "Audit plugins for security and quality issues")]
    Audit {
        #[arg(help = "Path to audit (directory or manifest file)")]
        path: Option<String>,

        #[arg(long, help = "Output JSON format")]
        json: bool,
    },
}

#[derive(Debug, clap::Args)]
pub struct PluginArgs {
    #[command(subcommand)]
    pub sub: PluginSub,
}

#[derive(Debug, Subcommand)]
pub enum PluginSub {
    #[command(about = "List all loaded plugins")]
    List {},
    #[command(about = "Load a plugin from manifest file")]
    Load {
        #[arg(help = "Path to plugin manifest (json/yaml/toml)")]
        path: String,
    },
    #[command(about = "Activate a loaded plugin")]
    Activate {
        #[arg(help = "Plugin ID")]
        id: String,
    },
    #[command(about = "Deactivate an active plugin")]
    Deactivate {
        #[arg(help = "Plugin ID")]
        id: String,
    },
    #[command(about = "Show plugin status and details")]
    Status {
        #[arg(help = "Plugin ID")]
        id: String,
    },
}

#[derive(Debug, clap::Args)]
pub struct CanaryArgs {
    #[command(subcommand)]
    pub sub: CanarySub,
}

#[derive(Debug, Subcommand)]
pub enum CanarySub {
    #[command(about = "Create a new canary release")]
    Create {
        #[arg(help = "Canary release name")]
        name: String,
        #[arg(short, long, help = "Old version (default: v1.0.0)")]
        old_version: Option<String>,
        #[arg(short, long, help = "New version (default: v2.0.0)")]
        new_version: Option<String>,
    },
    #[command(about = "List all canary releases")]
    List {},
    #[command(about = "Promote a canary release to 100%")]
    Promote {
        #[arg(help = "Canary release name")]
        name: String,
    },
    #[command(about = "Rollback a canary release")]
    Rollback {
        #[arg(help = "Canary release name")]
        name: String,
        #[arg(short, long, help = "Rollback reason")]
        reason: Option<String>,
    },
    #[command(about = "Show canary release status")]
    Status {
        #[arg(help = "Canary release name")]
        name: String,
    },
    #[command(about = "Start a canary release (begin traffic shift)")]
    Start {
        #[arg(help = "Canary release name")]
        name: String,
    },
}

#[derive(Debug, clap::Args)]
pub struct MeshArgs {
    #[command(subcommand)]
    pub sub: MeshSub,
}

#[derive(Debug, Subcommand)]
pub enum MeshSub {
    #[command(about = "Generate Istio configuration for a service")]
    Generate {
        #[arg(help = "Service name")]
        name: String,

        #[arg(short, long, help = "Kubernetes namespace (default: default)")]
        namespace: Option<String>,

        #[arg(short = 'V', long, help = "Service versions (default: v1)")]
        versions: Option<Vec<String>>,

        #[arg(short = 'g', long, help = "Gateway name")]
        gateway: Option<String>,

        #[arg(short = 'o', long, help = "Output file path")]
        output: Option<String>,
    },
    #[command(about = "Validate Istio configuration file")]
    Validate {
        #[arg(help = "Path to Istio configuration file")]
        path: String,
    },
    #[command(about = "Diff two Istio configuration files")]
    Diff {
        #[arg(help = "Path to old configuration file")]
        old: String,

        #[arg(help = "Path to new configuration file")]
        new: String,
    },
}

#[derive(Debug, clap::Args)]
pub struct AbArgs {
    #[command(subcommand)]
    pub sub: AbSub,
}

#[derive(Debug, Subcommand)]
pub enum AbSub {
    #[command(about = "Create a new A/B experiment")]
    Create {
        #[arg(help = "Experiment name")]
        name: String,
        #[arg(short, long, help = "Experiment type: ui, algorithm, feature, price")]
        experiment_type: Option<String>,
    },
    #[command(about = "List all experiments")]
    List {},
    #[command(about = "Start an experiment")]
    Start {
        #[arg(help = "Experiment name")]
        name: String,
    },
    #[command(about = "Pause an experiment")]
    Pause {
        #[arg(help = "Experiment name")]
        name: String,
    },
    #[command(about = "Show experiment results")]
    Result {
        #[arg(help = "Experiment name")]
        name: String,
    },
    #[command(about = "End an experiment")]
    End {
        #[arg(help = "Experiment name")]
        name: String,
    },
}

#[derive(Debug, clap::Args)]
pub struct PatternArgs {
    #[command(subcommand)]
    pub sub: PatternSub,
}

#[derive(Debug, Subcommand)]
pub enum PatternSub {
    #[command(about = "List all patterns")]
    List {
        #[arg(long, help = "Filter by category")]
        category: Option<String>,
    },
    #[command(about = "Search patterns")]
    Search {
        #[arg(help = "Search keyword")]
        keyword: String,
    },
    #[command(about = "Show pattern details")]
    Detail {
        #[arg(help = "Pattern ID")]
        id: String,
    },
    #[command(about = "Recommend related patterns")]
    Recommend {
        #[arg(help = "Pattern ID (optional)")]
        id: Option<String>,
    },
}

#[derive(Debug, clap::Args)]
pub struct RcaArgs {
    #[command(subcommand)]
    pub sub: RcaSub,
}

#[derive(Debug, Subcommand)]
pub enum RcaSub {
    #[command(about = "Analyze a signal for root cause")]
    Analyze {
        #[arg(help = "Signal description")]
        signal: String,
    },
    #[command(about = "List RCA analysis results")]
    List {},
    #[command(about = "Show RCA analysis details")]
    Detail {
        #[arg(help = "Analysis ID")]
        id: String,
    },
}

#[derive(Debug, clap::Args)]
pub struct RuleArgs {
    #[command(subcommand)]
    pub sub: RuleSub,
}

#[derive(Debug, Subcommand)]
pub enum RuleSub {
    #[command(about = "List all rules")]
    List {
        #[arg(long, help = "Filter by type")]
        rule_type: Option<String>,
    },
    #[command(about = "Evaluate a rule")]
    Evaluate {
        #[arg(help = "Rule ID")]
        id: String,
        #[arg(short, long, help = "Input data as JSON")]
        data: Option<String>,
    },
    #[command(about = "Activate a rule")]
    Activate {
        #[arg(help = "Rule ID")]
        id: String,
    },
    #[command(about = "Show rule version history")]
    Version {
        #[arg(help = "Rule ID")]
        id: String,
    },
}

#[derive(Debug, clap::Args)]
pub struct RefactorArgs {
    #[command(subcommand)]
    pub sub: RefactorSub,
}

#[derive(Debug, Subcommand)]
pub enum RefactorSub {
    #[command(about = "Analyze code smells")]
    Analyze {
        #[arg(short, long, help = "Path to analyze")]
        path: Option<String>,
    },
    #[command(about = "List refactor proposals")]
    List {
        #[arg(long, help = "Filter by severity")]
        severity: Option<String>,
    },
    #[command(about = "Apply a refactor proposal")]
    Apply {
        #[arg(help = "Proposal ID")]
        id: String,
    },
}

