use websocket::*;
use websocket::client::request::Url;
use openssl::ssl::*;
//use std::time::Duration;
//use std::thread::sleep;
//use std::io::{stdout, Write};
use curl::easy::Easy;
use std::str;
//use std;
use log::LogLevel;
use sqlite;



pub struct Connector<'a>
{
   sender: client::Sender<stream::WebSocketStream>,
   receiver: client::Receiver<stream::WebSocketStream>,
   curl_handle: Option<Easy>,
   _relay_session: Option<String>,
   nick: &'a str,
   sqlite_connection: sqlite::Connection,
}

const BIBA_BASE_ADDRESS: &'static str = "https://app.biba.com";


impl<'a> Connector<'a>
{
   pub fn new(nick: &'a str) -> Connector<'a>
   {
      let connection = sqlite::open("biba.db").unwrap();

      let ssl_context: SslContext = SslContext::new(SslMethod::Sslv23).unwrap();

      let url = Url::parse("wss://ec2-54-174-239-169.compute-1.amazonaws.com").unwrap(); // Get the URL
      let request = Client::connect_ssl_context(url, &ssl_context).unwrap(); // Connect to the server


      let response = request.send().unwrap(); // Send the request
      let client = response.begin();
      
      let (mut sender, mut receiver) = client.split();
//
//      for message in receiver.incoming_messages()
//      {
//         info!("Going to receive now...");
//
//         let new_message: Message;
//
//         match message
//         {
//            Ok(_) => 
//            {
//               new_message = message.unwrap();
//            },
//            Err(error) => 
//            {
//               error!("Error: {}", error);
//            }
//         }
//
//         sleep(Duration::new(1, 0));


         //if message.unwrap() == Err
         //{
         //}


         //let message: Message = message.unwrap();
         //println!("Recv: {:?}", message);
      //}


      Connector
      {
         sender: sender,
         receiver: receiver,
         curl_handle: None,
         _relay_session: None,
         nick: nick,
         sqlite_connection: connection,
      }   
   }


   /**
   * This method logs into Biba, and returns the curl Easy session, and the 
   * _realy_session cookie that needs to be used for follor up requests
   */
   pub fn login(&mut self, username: &str, password: &str) -> Result<(), &'static str>
   {

      self.build_sqlite_db();

      let mut _relay_session: Option<String> = None;

      let mut handle: Easy = Easy::new();
      handle.url(format!("{}/v2/sessions", BIBA_BASE_ADDRESS).as_str()).unwrap();
      handle.post(true).unwrap();
      handle.username(username).unwrap();
      handle.password(password).unwrap();

      {
         let mut transfer = handle.transfer();
         transfer.header_function
         (
            |header|
            {
               let utf8_header = str::from_utf8(header).unwrap();
               if utf8_header.starts_with("Set-Cookie: _relay_session=")
               {
                  info!("found _relay_session");
                  _relay_session = Some(utf8_header[12..].to_string());
               }

               info!("header: {}", str::from_utf8(header).unwrap());
               true
            }
         ).unwrap();

         transfer.perform().unwrap();
      }

      info!("{}", handle.response_code().unwrap());
      match handle.response_code().unwrap()
      {
         201 => 
         {

            let relay_session_ref = _relay_session.unwrap().as_str();

            self.add_key_to_db("_relay_session", relay_session_ref);

            self.curl_handle = Some(handle);
            self._relay_session = _relay_session;

            Ok(())
         },
         _ =>
         {
            Err("There was an error logging in to Biba")
         }
      }
   }

   fn add_key_to_db(&mut self, key: &str, value: &str)
   {
      let key_insert_result = self.sqlite_connection.execute("");
      match key_insert_result
      {
         Err(error) => 
         {
            error!("Error adding key to biba_cache table: {}", error.message.unwrap());
         },
         Ok(_) => 
         {
            info!("added key {} to biba_cache table.", key);
         }
      }

   }

   fn build_sqlite_db(&mut self)
   {
      let create_table_result = self.sqlite_connection.execute("CREATE TABLE biba_cache(key TEXT, value TEXT);");
      match create_table_result
      {
         Err(error) => 
         {
            warn!("biba_cache table already exists: {}", error.message.unwrap());
         },
         Ok(_) =>
         {
            info!("biba_cache table created.");
         }
      }
   }


}


