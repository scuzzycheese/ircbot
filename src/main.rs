extern crate stem;
extern crate openssl;
extern crate curl;
//use stem::*;
#[macro_use] extern crate log;
extern crate env_logger;
extern crate sqlite;

extern crate nlp;
use nlp::*;
extern crate websocket;

pub mod biba;

use log::LogLevel;


fn main() 
{
   info!("Connecting to Biba");
   env_logger::init().unwrap();

   let mut biba = biba::Connector::new("test");
   biba.login();




   let nickname = "TwoDelta";
   let nickname_soundex = phonetics::soundex::soundex(nickname);
   let nickname_jw_distance: f64 = 90.0;

}
