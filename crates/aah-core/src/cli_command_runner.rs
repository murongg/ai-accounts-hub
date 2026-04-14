use std::path::Path;
use std::process::{Command, ExitStatus};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CliCommandPlan {
    pub program: String,
    pub args: Vec<String>,
}

pub(crate) fn run_cli_status(
    binary: &Path,
    args: &[&str],
    envs: &[(&str, &Path)],
) -> Result<ExitStatus, std::io::Error> {
    let mut command = cli_command(binary, args);
    for (key, value) in envs {
        command.env(key, value);
    }
    command.status()
}

pub(crate) fn cli_command_plan(binary: &Path, args: &[&str]) -> CliCommandPlan {
    cli_command_plan_for_os(binary, args, cfg!(windows))
}

fn cli_command(binary: &Path, args: &[&str]) -> Command {
    let plan = cli_command_plan(binary, args);
    let mut command = Command::new(plan.program);
    command.args(plan.args);
    command
}

fn cli_command_plan_for_os(binary: &Path, args: &[&str], is_windows: bool) -> CliCommandPlan {
    if is_windows && is_windows_cmd_wrapper(binary) {
        return CliCommandPlan {
            program: "cmd.exe".to_string(),
            args: vec![
                "/C".to_string(),
                build_windows_cmd_command_line(binary, args),
            ],
        };
    }

    CliCommandPlan {
        program: binary.to_string_lossy().to_string(),
        args: args.iter().map(|arg| (*arg).to_string()).collect(),
    }
}

fn is_windows_cmd_wrapper(binary: &Path) -> bool {
    binary
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            extension.eq_ignore_ascii_case("cmd") || extension.eq_ignore_ascii_case("bat")
        })
}

fn build_windows_cmd_command_line(binary: &Path, args: &[&str]) -> String {
    std::iter::once(cmd_quote(&binary.to_string_lossy()))
        .chain(args.iter().map(|arg| cmd_quote(arg)))
        .collect::<Vec<_>>()
        .join(" ")
}

fn cmd_quote(arg: &str) -> String {
    let mut escaped = arg.replace('"', "").replace('%', "%%");
    let trailing_backslashes = escaped
        .chars()
        .rev()
        .take_while(|char| *char == '\\')
        .count();
    if trailing_backslashes > 0 {
        escaped.push_str(&"\\".repeat(trailing_backslashes));
    }
    format!("\"{escaped}\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_cmd_wrappers_run_through_cmd_exe() {
        let plan = cli_command_plan_for_os(
            Path::new(r"C:\Users\murong\AppData\Roaming\npm\claude.cmd"),
            &["auth", "login"],
            true,
        );

        assert_eq!(plan.program, "cmd.exe");
        assert_eq!(
            plan.args,
            vec![
                "/C".to_string(),
                r#""C:\Users\murong\AppData\Roaming\npm\claude.cmd" "auth" "login""#.to_string(),
            ]
        );
    }

    #[test]
    fn native_binaries_run_directly_on_windows() {
        let plan = cli_command_plan_for_os(
            Path::new(r"C:\Users\murong\scoop\shims\codex.exe"),
            &["login"],
            true,
        );

        assert_eq!(plan.program, r"C:\Users\murong\scoop\shims\codex.exe");
        assert_eq!(plan.args, vec!["login".to_string()]);
    }

    #[test]
    fn cmd_quoting_escapes_percent_expansion() {
        assert_eq!(
            cmd_quote(r"C:\Users\100%\claude.cmd"),
            r#""C:\Users\100%%\claude.cmd""#
        );
    }
}
