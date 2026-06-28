use cell_domain::errors::CellResult;
use cell_interfaces::cli::{Cli, Command};
use cell_interfaces::commands::{init_cmd, generate_cmd, entropy_cmd, feature_cmd, diagnose_cmd, event_cmd, lifecycle_cmd, evolve_cmd, arch_cmd, adr_cmd, decision_cmd, dev_cmd, quality_cmd, dev_workflow_cmd, template_cmd, git_cmd, project_cmd, bench_cmd, review_cmd, workflow_cmd, ws_cmd, agent_cmd, canary_cmd, docs_cmd, env_cmd, db_cmd, enforcement_cmd, task_cmd, self_verify_cmd, decide_cmd, tool_policy_cmd, audit_cmd, agent_profile_cmd, self_heal_cmd, mesh_cmd, sandbox_cmd, plugin_cmd, plugin_validator_cmd, ab_cmd, pattern_cmd, rca_cmd, rule_cmd, refactor_cmd, saga_cmd, contract_cmd, entropy_bank_cmd, complexity_quota_cmd};
use clap::Parser;

fn main() -> CellResult<()> {
    let cli = Cli::parse();
    init_tracing(cli.verbose);
    dispatch(cli)
}

fn dispatch(cli: Cli) -> CellResult<()> {
    match cli.command {
        Command::Init(args) => init_cmd::cmd_init(args),
        Command::Generate(args) => generate_cmd::cmd_generate(args),
        Command::Validate(args) => init_cmd::cmd_validate(args),
        Command::Entropy(args) => entropy_cmd::cmd_entropy(args, cli.format),
        Command::Feature(args) => feature_cmd::cmd_feature(args),
        Command::Diagnose(args) => diagnose_cmd::cmd_diagnose(args),
        Command::Event(args) => event_cmd::cmd_event(args),
        Command::Handoff(args) => lifecycle_cmd::cmd_handoff(args),
        Command::Progress(args) => lifecycle_cmd::cmd_progress(args),
        Command::Evolve(args) => evolve_cmd::cmd_evolve(args),
        Command::Arch(args) => arch_cmd::cmd_arch(args),
        Command::Adr(args) => adr_cmd::cmd_adr(args),
        Command::Decision(args) => decision_cmd::cmd_decision(args, cli.format),
        Command::Dashboard(args) => dev_cmd::cmd_dashboard(args),
        Command::Test(args) => quality_cmd::cmd_test(args),
        Command::Verify(args) => quality_cmd::cmd_verify(args),
        Command::Config(args) => quality_cmd::cmd_config(args),
        Command::Lint(args) => quality_cmd::cmd_lint(args),
        Command::Dev(args) => dev_workflow_cmd::cmd_dev(args),
        Command::Template(args) => template_cmd::cmd_template(args),
        Command::Git(args) => git_cmd::cmd_git(args),
        Command::Project(args) => project_cmd::cmd_project(args),
        Command::Bench(args) => bench_cmd::cmd_bench(args),
        Command::Review(args) => review_cmd::cmd_review(args),
        Command::Workflow(args) => workflow_cmd::cmd_workflow(args),
        Command::Ws(args) => ws_cmd::cmd_ws(args),
        Command::Agent(args) => agent_cmd::cmd_agent(args),
        Command::Canary(args) => canary_cmd::cmd_canary(args),
        Command::Docs(args) => docs_cmd::cmd_docs(args),
        Command::Env(args) => env_cmd::cmd_env(args),
        Command::Db(args) => db_cmd::cmd_db(args),
        Command::Enforcement(args) => enforcement_cmd::cmd_enforcement(args),
        Command::Task(args) => task_cmd::cmd_task(args),
        Command::SelfVerify(args) => self_verify_cmd::cmd_self_verify(args),
        Command::Decide(args) => decide_cmd::cmd_decide(args),
        Command::ToolPolicy(args) => tool_policy_cmd::cmd_tool_policy(args),
        Command::Audit(args) => audit_cmd::cmd_audit(args),
        Command::AgentProfile(args) => agent_profile_cmd::cmd_agent_profile(args),
        Command::SelfHeal(args) => self_heal_cmd::cmd_self_heal(args),
        Command::Mesh(args) => mesh_cmd::cmd_mesh(args),
        Command::Sandbox(args) => sandbox_cmd::cmd_sandbox(args),
        Command::Tools(args) => dev_cmd::cmd_tools(args),
        Command::Completions(args) => dev_cmd::cmd_completions(args),
        Command::Plugin(args) => plugin_cmd::cmd_plugin(args),
        Command::PluginValidate(args) => plugin_validator_cmd::cmd_plugin_validate(args),
        Command::Ab(args) => ab_cmd::cmd_ab(args),
        Command::Pattern(args) => pattern_cmd::cmd_pattern(args),
        Command::Rca(args) => rca_cmd::cmd_rca(args),
        Command::Rule(args) => rule_cmd::cmd_rule(args),
        Command::Refactor(args) => refactor_cmd::cmd_refactor(args),
        Command::Saga(args) => saga_cmd::cmd_saga(args),
        Command::Contract(args) => contract_cmd::cmd_contract(args),
        Command::EntropyBank(args) => entropy_bank_cmd::cmd_entropy_bank(args),
        Command::ComplexityQuota(args) => complexity_quota_cmd::cmd_complexity_quota(args),
    }
}

fn init_tracing(verbose: u8) {
    let level = match verbose {
        0 => "info",
        1 => "debug",
        _ => "trace",
    };
    let _ = tracing_subscriber::fmt()
        .with_env_filter(format!("cell_cli={level},cell_core={level}"))
        .with_target(false)
        .try_init();
}
