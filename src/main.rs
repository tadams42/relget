use anyhow::Result;
use relget::{create_cli, execute_cli};

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format(|buf, record| {
            use std::io::Write;
            writeln!(buf, "lvl={} {}", record.level(), record.args())
        })
        .init();
    let cli = create_cli()?;
    execute_cli(&cli)
}
