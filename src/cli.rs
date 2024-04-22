use clap::Parser;

#[derive(Parser)]
#[command(author, about, long_about = None, next_line_help = false)]
struct Cli {
    ///执行配置文件的路径，格式为toml
    profile_path: String,

    /// 输出结果文件的路径，输出格式为toml格式
    /// 默认值为./output.toml
    output_path: Option<String>,
}

pub fn get_args() -> (String, String) {
    let args = Cli::parse();
    let profile_path = args.profile_path;

    let output_path = if let Some(p) = args.output_path {
        p
    } else {
        String::from("./output.toml")
    };

    (profile_path, output_path)
}
