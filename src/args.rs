use anyhow::{Result, bail, ensure};

#[derive(Debug)]
pub(crate) enum Args {
    GenerateKey,
    StartServer,
}

const USAGE: &str = r#"Usage:

minippa --generate-key
minippa --start-server
"#;

impl Args {
    pub(crate) fn parse() -> Self {
        match Self::try_parse() {
            Ok(args) => args,
            Err(_) => {
                eprintln!("{USAGE}");
                std::process::exit(1)
            }
        }
    }

    fn try_parse() -> Result<Self> {
        let args = std::env::args().skip(1).collect::<Vec<_>>();

        match args.first().map(|arg| arg.as_str()) {
            Some("--generate-key") => {
                ensure!(args.len() == 1);
                Ok(Self::GenerateKey)
            }
            Some("--start-server") => {
                ensure!(args.len() == 1);
                Ok(Self::StartServer)
            }
            other => bail!("invalid arguments: {other:?}"),
        }
    }
}
