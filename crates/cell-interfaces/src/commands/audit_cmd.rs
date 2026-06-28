use cell_application::audit_service::{AuditService, AuditQuery, AuditActionType, AuditResult};
use cell_domain::errors::CellResult;
use crate::cli::{AuditArgs, AuditSub};

pub fn cmd_audit(args: AuditArgs) -> CellResult<()> {
    let service = AuditService::new();
    let project_path = ".";

    match args.sub {
        AuditSub::Log {} => {
            let query = AuditQuery {
                agent_id: args.agent.clone(),
                action_type: None,
                result: None,
                task_id: None,
                from_time: None,
                to_time: None,
                limit: args.limit.unwrap_or(50),
            };
            let report = service.query(project_path, query)?;
            println!("{}", service.format_report(&report));
        }
        AuditSub::Query { action, result } => {
            let action_type = action.as_deref().map(|a| match a {
                "tool_call" => AuditActionType::ToolCall,
                "file_read" => AuditActionType::FileRead,
                "file_write" => AuditActionType::FileWrite,
                "file_delete" => AuditActionType::FileDelete,
                "command_exec" => AuditActionType::CommandExec,
                "decision" => AuditActionType::Decision,
                "task_start" => AuditActionType::TaskStart,
                "task_complete" => AuditActionType::TaskComplete,
                "git_commit" => AuditActionType::GitCommit,
                "git_push" => AuditActionType::GitPush,
                _ => AuditActionType::Other,
            });

            let result_filter = result.as_deref().map(|r| match r {
                "success" => AuditResult::Success,
                "failure" => AuditResult::Failure,
                "blocked" => AuditResult::Blocked,
                "warning" => AuditResult::Warning,
                _ => AuditResult::Success,
            });

            let query = AuditQuery {
                agent_id: args.agent.clone(),
                action_type,
                result: result_filter,
                task_id: None,
                from_time: None,
                to_time: None,
                limit: args.limit.unwrap_or(50),
            };
            let report = service.query(project_path, query)?;
            println!("{}", service.format_report(&report));
        }
        AuditSub::Trace { file } => {
            let trace = service.trace_file(project_path, &file)?;
            println!("{}", service.format_trace(&trace));
        }
    }

    Ok(())
}
