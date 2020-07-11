use anyhow::{bail, Context, Result};

use zoxide::subcommand::init::shell;
use std::io::Write;
use std::process::{Command, Output, Stdio};
use std::str;

macro_rules! make_tests {
    ($shell:ident, $config:expr) => {
        paste::item! {
            #[test]
            fn [< $shell _z >] () -> Result<()> {
                let z = ($config.z)("z");
                [< $shell _test >](z.trim())
            }

            #[test]
            fn [< $shell _alias >] () -> Result<()> {
                let alias = ($config.alias)("z");
                [< $shell _test >](alias.trim())
            }

            #[test]
            fn [< $shell _hook_prompt >] () -> Result<()> {
                let prompt = $config.hook.prompt;
                [< $shell _test >](prompt.trim())
            }

            #[test]
            fn [< $shell _hook_pwd >] () -> Result<()> {
                let pwd = ($config.hook.pwd)()?;
                [< $shell _test >](pwd.trim())
            }
        }
    };
}

make_tests!(bash, shell::bash::CONFIG);
make_tests!(bash_posix, shell::posix::CONFIG);
make_tests!(dash, shell::posix::CONFIG);
make_tests!(ksh, shell::posix::CONFIG);
make_tests!(fish, shell::fish::CONFIG);
make_tests!(pwsh, shell::powershell::CONFIG);
make_tests!(shellcheck_bash, shell::bash::CONFIG);
make_tests!(shellcheck_sh, shell::posix::CONFIG);
make_tests!(zsh, shell::zsh::CONFIG);

fn bash_test(command: &str) -> Result<()> {
    generic_init("bash", command);
    generic_version(Command::new("bash").arg("--version"))?;
    generic_command(Command::new("bash").args(&["-c", command]))
}

fn bash_posix_test(command: &str) -> Result<()> {
    generic_init("bash-posix", command);
    generic_version(Command::new("bash").arg("--version"))?;
    generic_command(Command::new("bash").args(&["--posix", "-c", command]))
}

fn dash_test(command: &str) -> Result<()> {
    generic_init("dash", command);
    generic_command(Command::new("dash").args(&["-c", command]))
}

fn ksh_test(command: &str) -> Result<()> {
    generic_init("sh", command);
    generic_version(Command::new("ksh").args(&[
        "-c",
        r#"
        case "$KSH_VERSION" in
            (*MIRBSD*|*PD*|*LEGACY*) printf '%s\n' "$KSH_VERSION" ;;
            (*) [ -z "$ERRNO" ] && printf '%s\n' "${.sh.version}" || echo ksh88/86 ;;
        esac
    "#,
    ]))?;
    generic_command(Command::new("ksh").args(&["-c", command]))
}

fn fish_test(command: &str) -> Result<()> {
    generic_init("fish", command);
    generic_version(Command::new("fish").arg("--version"))?;
    generic_command(Command::new("fish").args(&["--command", command]))
}

fn pwsh_test(command: &str) -> Result<()> {
    generic_init("pwsh", command);
    generic_version(Command::new("pwsh").arg("-Version"))?;
    generic_command(Command::new("pwsh").args(&["-Command", command]))
}

fn shellcheck_bash_test(command: &str) -> Result<()> {
    shellcheck_test("bash", command)
}

fn shellcheck_sh_test(command: &str) -> Result<()> {
    shellcheck_test("sh", command)
}

fn shellcheck_test(shell: &str, command: &str) -> Result<()> {
    generic_init(&format!("shellcheck-{}", shell), command);
    generic_version(Command::new("shellcheck").arg("--version"))?;

    let mut shellcheck = Command::new("shellcheck")
        .args(&["--shell", shell, "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("could not start shellcheck")?;

    write!(shellcheck.stdin.as_mut().unwrap(), "{}", command)
        .context("could not write to shellcheck")?;

    let output = shellcheck
        .wait_with_output()
        .context("could not run shellcheck")?;
    generic_command_helper(output)
}

fn zsh_test(command: &str) -> Result<()> {
    generic_init("zsh", command);
    generic_version(Command::new("zsh").arg("--version"))?;
    generic_command(Command::new("zsh").args(&["-c", command]))
}

fn generic_init(shell: &str, command: &str) {
    println!("shell: {}", shell);
    print!("command:\n{}", indent(command));
}

fn generic_version(command: &mut Command) -> Result<()> {
    let output = command
        .output()
        .with_context(|| format!("version check failed"))?;

    let stdout = str::from_utf8(&output.stdout)?;
    let stderr = str::from_utf8(&output.stderr)?;
    print!("version:\n{}{}", indent(stdout), indent(stderr));

    if !output.status.success() {
        bail!("version check failed with {}", output.status);
    }

    Ok(())
}

fn generic_command(command: &mut Command) -> Result<()> {
    let output = command
        .output()
        .with_context(|| format!("could not run command"))?;

    generic_command_helper(output)
}

fn generic_command_helper(output: Output) -> Result<()> {
    let stdout = str::from_utf8(&output.stdout)?;
    print!("stdout:\n{}", indent(stdout));

    let stderr = str::from_utf8(&output.stderr)?.trim();
    print!("stderr:\n{}", indent(stderr));

    if !output.status.success() {
        bail!("command failed with {}", output.status);
    }

    if !stdout.is_empty() || !stderr.is_empty() {
        bail!("command exited with output");
    }

    Ok(())
}

fn indent(text: &str) -> String {
    textwrap::indent(text.trim(), "    ")
}
