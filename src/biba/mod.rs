use websocket::*;
use websocket::client::request::Url;
use openssl::ssl::*;
use curl::easy::Easy;
use std::str;
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
      build_biba_settings(&connection);

      let ssl_context: SslContext = SslContext::new(SslMethod::Sslv23).unwrap();

      let url = Url::parse("wss://ec2-54-174-239-169.compute-1.amazonaws.com").unwrap(); // Get the URL
      let request = Client::connect_ssl_context(url, &ssl_context).unwrap(); // Connect to the server


      let response = request.send().unwrap(); // Send the request
      let client = response.begin();
      
      let (mut sender, mut receiver) = client.split();

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
   pub fn login(&mut self) -> Result<(), &'static str>
   {
      let username = try!(self.get_setting_value("username"));
      let password = try!(self.get_setting_value("password"));


      let mut handle: Easy = Easy::new();

      self.build_biba_cache();

      match self.get_key_from_db("_relay_session")
      {
         Ok((value_string, expiry)) => 
         {
            self.curl_handle = Some(handle);
            self._relay_session = Some(value_string);
            return Ok(());
         },
         Err(error) => 
         {
            info!("Unable to fetch _relay_session key from cache: {}", error);
         }
      }

      let mut _relay_session: Option<String> = None;

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
            let relay_session_string: String = _relay_session.unwrap();

            self.add_key_to_db("_relay_session", relay_session_string.as_str());

            self.curl_handle = Some(handle);
            self._relay_session = Some(relay_session_string);

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
      let key_insert_result = self.sqlite_connection.execute(format!("INSERT INTO biba_cache(key, value) VALUES ('{}', '{}');", key, value));
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

   fn get_key_from_db(&mut self, key: &str) -> Result<(String, i64), String>
   {
      let mut statement = self.sqlite_connection.prepare("SELECT * FROM biba_cache WHERE key = ?").unwrap();      
      statement.bind(1, key).unwrap();

      //Read the first row
      match statement.next()
      {
         Ok(_) => 
         {
            let string_read_column = statement.read::<String>(1);
            let integer_read_column = statement.read::<i64>(2);
            match (string_read_column, integer_read_column)
            {
               (Ok(result_string), Ok(result_int)) => 
               {
                  Ok((result_string, result_int))
               },
               (Err(error_string), Err(error_int)) =>
               {
                  let error_message = format!("Errors fetching results from DB: {} - {}", error_string.message.unwrap(), error_int.message.unwrap());
                  error!("Unable to read key from biba_cache table: {}", error_message);
                  Err(error_message)
               },
               _ => 
               {
                  error!("Unable to read key from biba_cache table");
                  Err("Error fetching results from DB".to_string())
               }
            }
         },
         Err(error) =>
         {
            let error_message = error.message.unwrap();
            error!("Unable to read key from biba_cache table: {}", error_message);
            Err(error_message)
         }
      }
   }

   fn build_biba_cache(&mut self)
   {
      let create_table_result = self.sqlite_connection.execute("CREATE TABLE biba_cache(key TEXT PRIMARY KEY, value TEXT, expiry INTEGER);");
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

   fn get_setting_value(&mut self, key: &str) -> Result<String, String>
   {
      let mut statement = self.sqlite_connection.prepare("SELECT * FROM biba_settings WHERE key = ?").unwrap();      
      statement.bind(1, key).unwrap();

      //Read the first row
      match statement.next()
      {
         Ok(_) => 
         {
            let string_read_column = statement.read::<String>(1);
            match string_read_column
            {
               Ok(result_string) => 
               {
                  Ok(result_string)
               },
               Err(error_string) =>
               {
                  let error_message = format!("Errors fetching results from DB: {}", error_string.message.unwrap());
                  error!("Unable to read key from biba_cache table: {}", error_message);
                  Err(error_message)
               }
            }
         },
         Err(error) =>
         {
            let error_message = error.message.unwrap();
            error!("Unable to read key from biba_cache table: {}", error_message);
            Err(error_message)
         }
      }
   }

}

fn build_biba_settings(connection: &sqlite::Connection)
{
   let create_table_result = connection.execute
   ("
      CREATE TABLE biba_settings(key TEXT PRIMARY KEY, value TEXT);
      INSERT INTO biba_settings(key, value) VALUES(\"name\", \"TwoDelta\");
      INSERT INTO biba_settings(key, value) VALUES(\"username\", \"someusername\");
      INSERT INTO biba_settings(key, value) VALUES(\"password\", \"somepassword\");
   ");
   match create_table_result
   {
      Err(error) => 
      {
         warn!("biba_settings table already exists: {}", error.message.unwrap());
      },
      Ok(_) =>
      {
         info!("biba_settings table created.");
      }
   }
}




