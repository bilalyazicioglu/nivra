pub fn init() -> anyhow::Result<String> {
    let exe = std::env::current_exe()?;
    let quoted = format!("'{}'", exe.to_string_lossy().replace('\'', "'\\''"));
    Ok(include_str!("../scripts/nivra.zsh")
        .replace("__NIVRA_BINARY__", &quoted)
        .replace(
            "__NIVRA_SESSION__",
            &format!("zsh-{}", uuid::Uuid::new_v4()),
        ))
}
