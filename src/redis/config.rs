pub struct Config {
    pub host: String,
    pub port: String,
}

impl Config {
    pub fn parse(args: Vec<String>) -> Result<Config, String> {
        let len = args.len();
        let mut cfg = Config::default();
        for i in (1..len-1).step_by(2) {
            match args[i].as_str() {
                "--port" => cfg.port = extract_arg(&args, i + 1)?,
                "--host" => cfg.host = extract_arg(&args, i + 1)?,
                _ => ()
            }
        }

        Ok(cfg)
    }

    pub fn addr(self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            host: String::from("127.0.0.1"),
            port: String::from("6379"),
        }
    }
}

fn extract_arg(args: &[String], i: usize) -> Result<String, String> {
    if let Some(value) = args.get(i) {
        Ok(value.clone())
    } else {
        Err(format!("No value for {}", args[i-1]))
    }
}