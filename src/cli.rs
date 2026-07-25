use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about = "outrrr - Stream stdout to Discord via Pipe based on Shoutrrr")]
pub struct Args {
    /// 사용할 YAML 설정 파일 경로
    #[arg(short, long, default_value = "config.yaml")]
    pub config: PathBuf,
}
