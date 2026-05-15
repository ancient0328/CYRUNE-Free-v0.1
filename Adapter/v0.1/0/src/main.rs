use adapter_resolver::{load_binding, load_catalog, load_policy, resolve_configuration};
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "adapter-resolver")]
#[command(about = "Resolve adapter manifest + distro policy + binding into effective config")]
struct Cli {
    #[arg(long = "catalog-dir")]
    catalog_dir: PathBuf,
    #[arg(long)]
    policy: PathBuf,
    #[arg(long)]
    binding: PathBuf,
    #[arg(long)]
    output: Option<PathBuf>,
}

fn main() {
    let cli = Cli::parse();

    let result = (|| -> Result<(), String> {
        let catalog = load_catalog(&cli.catalog_dir).map_err(|e| e.to_string())?;
        let policy = load_policy(&cli.policy).map_err(|e| e.to_string())?;
        let binding = load_binding(&cli.binding).map_err(|e| e.to_string())?;
        let resolved =
            resolve_configuration(&catalog, &policy, &binding).map_err(|e| e.to_string())?;

        let json = serde_json::to_string_pretty(&resolved).map_err(|e| e.to_string())?;
        if let Some(path) = cli.output {
            std::fs::write(path, format!("{json}\n")).map_err(|e| e.to_string())?;
        } else {
            println!("{json}");
        }
        Ok(())
    })();

    if let Err(err) = result {
        eprintln!("resolver error: {err}");
        std::process::exit(2);
    }
}
