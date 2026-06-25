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

    #[command(about = "Enable/disable development helper tools")]
    #[command(visible_aliases = ["t", "tool"])]
    Tools(ToolsArgs),

    #[command(about = "Generate shell completions")]
    Completions(CompletionsArgs),
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
}

#[derive(Debug, clap::Args)]
pub struct FeatureArgs {
    #[command(subcommand)]
    pub sub: FeatureSub,
}

#[derive(Debug, Subcommand)]
pub enum FeatureSub {
    #[command(about = "Create a new feature unit")]
    New { name: String },
    #[command(about = "Mount a feature unit")]
    Mount { name: String },
    #[command(about = "Unmount a feature unit")]
    Unmount { name: String },
    #[command(about = "Analyze feature impact")]
    Impact { name: String },
    #[command(about = "List all feature units")]
    List {},
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
        #[arg(long)]
        fix: bool,
    },
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
    New { title: String },
    #[command(about = "List ADRs")]
    List {},
    #[command(about = "Show ADR details")]
    Show { id: String },
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
pub struct CompletionsArgs {
    #[arg(help = "Shell: bash, zsh, fish, powershell, elvish")]
    pub shell: clap_complete::Shell,
}
