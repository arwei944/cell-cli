use cell_core::domain::errors::CellResult;
use cell_core::interfaces::cli::*;
use cell_core::interfaces::commands::*;
use clap::Parser;

fn main() -> CellResult<()> {
    let cli = Cli::parse();
    init_tracing(cli.verbose);
    dispatch(cli)
}

fn dispatch(cli: Cli) -> CellResult<()> {
    match cli.command {
        Command::Init(args) => init_cmd::cmd_init(args),
        Command::Generate(args) => init_cmd::cmd_generate(args),
        Command::Validate(args) => init_cmd::cmd_validate(args),
        Command::Entropy(args) => entropy_cmd::cmd_entropy(args, cli.format),
        Command::Feature(args) => entropy_cmd::cmd_feature(args),
        Command::Handoff(args) => lifecycle_cmd::cmd_handoff(args),
        Command::Progress(args) => lifecycle_cmd::cmd_progress(args),
        Command::Evolve(args) => evolve_cmd::cmd_evolve(args),
        Command::Arch(args) => arch_cmd::cmd_arch(args),
        Command::Adr(args) => decision_cmd::cmd_adr(args),
        Command::Decision(args) => decision_cmd::cmd_decision(args, cli.format),
        Command::Dashboard(args) => dev_cmd::cmd_dashboard(args),
        Command::Test(args) => quality_cmd::cmd_test(args),
        Command::Verify(args) => quality_cmd::cmd_verify(args),
        Command::Config(args) => quality_cmd::cmd_config(args),
        Command::Lint(args) => quality_cmd::cmd_lint(args),
        Command::Tools(args) => dev_cmd::cmd_tools(args),
        Command::Completions(args) => dev_cmd::cmd_completions(args),
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
