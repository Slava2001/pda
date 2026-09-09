use crate::core::{interface::IfMngr, module::Module};
use crate::modules::power_ctrl::PowerCtrlIf;
use anyhow::{Context, Result};
use async_trait::async_trait;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::fs::{File, OpenOptions};
use tokio::io::AsyncWriteExt;
use tokio::time::{interval, Duration, MissedTickBehavior};

const DEFAULT_CSV_PATH: &str = "/root/voltage_log.csv";
const LOG_PERIOD: Duration = Duration::from_secs(1);

pub struct VoltageLogger {
    path: PathBuf,
}

impl VoltageLogger {
    pub async fn build(_if_mngr: IfMngr) -> Result<Self> {
        Ok(Self {
            path: PathBuf::from(DEFAULT_CSV_PATH),
        })
    }
}

async fn open_csv(path: &PathBuf) -> Result<File> {
    let is_new = !tokio::fs::try_exists(path).await.unwrap_or(false);

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .await
        .with_context(|| format!("Failed to open csv file {path:?}"))?;

    if is_new {
        file.write_all(b"timestamp_ms,voltage_v,current_a\n")
            .await
            .context("Failed to write csv header")?;
        file.flush().await.context("Failed to flush csv header")?;
        file.sync_all()
            .await
            .context("Failed to fsync csv header")?;
    }

    Ok(file)
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

async fn read_measurement(power_ctrl: &mut PowerCtrlIf) -> Result<(f32, f32)> {
    let voltage = power_ctrl
        .get_voltage()
        .await
        .context("Failed to call voltage()")?
        .context("Failed to get voltage")?;

    let current = power_ctrl
        .get_current()
        .await
        .context("Failed to call current()")?
        .context("Failed to get current")?;

    Ok((voltage, current))
}

async fn write_and_sync(file: &mut File, line: &str) -> Result<()> {
    file.write_all(line.as_bytes())
        .await
        .context("write_all failed")?;
    file.flush().await.context("flush failed")?;
    file.sync_all().await.context("sync_all failed")?;
    Ok(())
}

#[async_trait]
impl Module for VoltageLogger {
    async fn run(&mut self, if_mngr: IfMngr) -> Result<()> {
        let mut power_ctrl: PowerCtrlIf = if_mngr
            .get("power_ctrl")
            .await
            .context("Failed to get power_ctrl if")?;

        let mut file = open_csv(&self.path).await?;

        let mut ticker = interval(LOG_PERIOD);
        ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);

        loop {
            ticker.tick().await;

            let (voltage, current) = match read_measurement(&mut power_ctrl).await {
                Ok(v) => v,
                Err(e) => {
                    println!("Failed to read voltage/current: {e:#}");
                    continue;
                }
            };

            let line = format!("{},{:.3},{:.3}\n", now_ms(), voltage, current);

            if let Err(e) = write_and_sync(&mut file, &line).await {
                println!("Failed to persist log line: {e:#}");
                match open_csv(&self.path).await {
                    Ok(f) => file = f,
                    Err(reopen_err) => {
                        println!("Failed to reopen csv file: {reopen_err:#}");
                    }
                }
            }
        }
    }
}
