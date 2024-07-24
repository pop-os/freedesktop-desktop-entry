

need:

- option to select the GpuPreference:
pub enum GpuPreference {
    Default,
    NonDefault,
    SpecificIdx(u32),
}


- request a token:
    params:
    - app_id
    - window id
























- launch:

    if token exist:
        envs.push(("XDG_ACTIVATION_TOKEN".to_string(), token.clone()));
        envs.push(("DESKTOP_STARTUP_ID".to_string(), token));

    
    env for gpu:
        async fn try_get_gpu_envs(gpu: GpuPreference) -> Option<HashMap<String, String>> {
            let connection = zbus::Connection::system().await.ok()?;
            let proxy = switcheroo_control::SwitcherooControlProxy::new(&connection)
                .await
                .ok()?;
            let gpus = proxy.get_gpus().await.ok()?;
            match gpu {
                GpuPreference::Default => gpus.into_iter().find(|gpu| gpu.default),
                GpuPreference::NonDefault => gpus.into_iter().find(|gpu| !gpu.default),
                GpuPreference::SpecificIdx(idx) => gpus.into_iter().nth(idx as usize),
            }
            .map(|gpu| gpu.environment)
        }

















spawn

pub fn spawn_desktop_exec<S, I, K, V>(exec: S, env_vars: I)
where
    S: AsRef<str>,
    I: IntoIterator<Item = (K, V)>,
    K: AsRef<OsStr>,
    V: AsRef<OsStr>,
{
    let mut exec = shlex::Shlex::new(exec.as_ref());
    let mut cmd = match exec.next() {
        Some(cmd) if !cmd.contains('=') => std::process::Command::new(cmd),
        _ => return,
    };

    for arg in exec {
        // TODO handle "%" args here if necessary?
        if !arg.starts_with('%') {
            cmd.arg(arg);
        }
    }

    cmd.envs(env_vars);

    crate::process::spawn(cmd);
}


/// Performs a double fork with setsid to spawn and detach a command.
pub fn spawn(mut command: Command) {
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    unsafe {
        match fork() {
            Ok(ForkResult::Parent { child }) => {
                let _res = waitpid(Some(child), None);
            }

            Ok(ForkResult::Child) => {
                let _res = nix::unistd::setsid();
                let _res = command.spawn();

                exit(0);
            }

            Err(why) => {
                println!("failed to fork and spawn command: {}", why.desc());
            }
        }
    }
}
