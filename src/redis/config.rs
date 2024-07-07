use super::utils::Addr;

#[derive(Default)]
pub struct Config {
    pub addr: Addr,
    pub master_addr: Option<Addr>,
}

impl Config {
    pub fn parse(args: Vec<String>) -> Result<Config, String> {
        let len = args.len();
        let mut cfg = Config::default();

        for i in (1..len - 1).step_by(2) {
            match args[i].as_str() {
                "--host" => cfg.addr.host = extract_arg(&args, i + 1)?,
                "--port" => cfg.addr.port = extract_arg(&args, i + 1)?,
                "--replicaof" => cfg.master_addr = Some(
                    Config::parse_master_addr(
                        extract_arg(&args, i + 1)?
                    )?
                ),
                unknown => return Err(format!("Unknown param: {}", unknown))
            }
        }

        Ok(cfg)
    }

    fn parse_master_addr(addr_line: String) -> Result<Addr, String> {
        match addr_line.split_once(' ') {
            Some((host, port)) => Ok(Addr {
                host: host.to_string(),
                port: port.to_string(),
            }),
            None => Err("No master address provided".into())
        }
    }
}

fn extract_arg(args: &[String], i: usize) -> Result<String, String> {
    if let Some(value) = args.get(i) {
        Ok(value.clone())
    } else {
        Err(format!("No value for {}", args[i - 1]))
    }
}