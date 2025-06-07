pub struct Args {
    pub name: String,
    pub disable: bool,
    pub auto: bool,
    pub on_login: bool,
}

impl Args {
    pub fn parse() -> Self {
        let mut name = "Defender-rs".to_string();
        let mut disable = false;
        let mut auto = false;
        let mut on_login = false;
        let mut args = std::env::args().skip(1); // 跳过程序名

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--name" => {
                    if let Some(val) = args.next() {
                        name = val;
                    }
                }
                "--disable" => {
                    disable = true;
                }
                "--auto" => {
                    auto = true;
                }
                "--on-login" => {
                    on_login = true;
                }
                _ => {}
            }
        }

        Args {
            name,
            disable,
            auto,
            on_login,
        }
    }
}
