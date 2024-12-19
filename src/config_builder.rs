use std::io::Read;
use std::time::Duration;
use serde::{Serialize, Deserialize};
use crate::bot_core::{BotConfig};


#[derive(Deserialize,Serialize,Debug)]
pub struct Config{
    bot: BotConfig
}
impl Config{
    pub fn config(self) -> (BotConfig) {
        self.bot
    }
}
pub fn get_config() -> Config{
    let mut file = std::fs::File::open("config.toml").unwrap();
    let mut string = String::new();
    file.read_to_string(&mut string).unwrap();
    toml::from_str(string.as_str()).unwrap()
}


#[cfg(test)]
mod test {
    use super::*;
    use std::fs::File;
    use std::io::Read;

    #[test]
    fn test_bot_config(){
        let mut file = File::open("config.toml").unwrap();
        let mut string = String::new();
        file.read_to_string(&mut string).unwrap();
        let config: Config = toml::from_str(string.as_str()).unwrap();
        println!("{:?}",config);
    }
}