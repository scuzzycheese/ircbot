//use websocket::*;
//use websocket::client::request::Url;
//use openssl::ssl::*;
use curl::easy::Easy;
use std::str;
use std::io::{Read};

pub mod settings;





pub struct Connector<'a>
{
   //sender: client::Sender<stream::WebSocketStream>,
   //receiver: client::Receiver<stream::WebSocketStream>,
   curl_handle: Option<Easy>,
   _relay_session: Option<String>,
   nick: &'a str,
   settings: settings::Settings,
}

const BIBA_BASE_ADDRESS: &'static str = "https://app.biba.com";


impl<'a> Connector<'a>
{
   pub fn new(nick: &'a str) -> Connector<'a>
   {
      let mut settings = settings::new();

      let bot_nick = settings.get_setting_value("name");

      //let ssl_context: SslContext = SslContext::new(SslMethod::Sslv23).unwrap();
      //let url = Url::parse("wss://ec2-54-174-239-169.compute-1.amazonaws.com").unwrap(); // Get the URL
      //let request = Client::connect_ssl_context(url, &ssl_context).unwrap(); // Connect to the server
      //let response = request.send().unwrap(); // Send the request
      //let client = response.begin();
      //let (sender, receiver) = client.split();

      Connector
      {
         //sender: sender,
         //receiver: receiver,
         curl_handle: None,
         _relay_session: None,
         nick: nick,
         settings: settings,
      }   
   }


   /**
   * This method logs into Biba, and returns the curl Easy session, and the 
   * _realy_session cookie that needs to be used for follor up requests
   */
   pub fn login(&mut self) -> Result<(), String>
   {
      let username = try!(self.settings.get_setting_value("username"));
      let password = try!(self.settings.get_setting_value("password"));

      //info!("username: \"{}\"", username);
      //info!("password: \"{}\"", password);

      let mut handle: Easy = Easy::new();

      match self.settings.get_key_from_db("_relay_session")
      {
         //TODO: Implement expiry when reading the relay_session
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
      handle.username(&username).unwrap();
      handle.password(&password).unwrap();

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
                  let end_index: Option<usize> = utf8_header.find(';');
                  
                  
                  _relay_session = match end_index
                  {
                     Some(index) =>
                     {
                        Some(utf8_header[27 .. index].to_string())
                     },
                     None => 
                     {
                        Some(utf8_header[27 ..].to_string())
                     }
                  };
               }

               info!("header: {}", str::from_utf8(header).unwrap());
               true
            }
         ).unwrap();

         transfer.perform().unwrap();
      }

      info!("BIBA RESPONSE CODE: {}", handle.response_code().unwrap());
      match handle.response_code().unwrap()
      {
         201 => 
         {
            let relay_session_string: String = _relay_session.unwrap();

            self.settings.add_key_to_db("_relay_session", relay_session_string.as_str());

            self.curl_handle = Some(handle);
            self._relay_session = Some(relay_session_string);

            Ok(())
         },
         _ =>
         {
            error!("There was an error logging in to Biba");
            Err("There was an error logging in to Biba".to_string())
         }
      }
   }

   pub fn send_chat_room_message(&mut self, room_id: u64, message: &str) -> Result<(), String>
   {
      //we need our string as bytes
      let mut message_data: &[u8] = message.as_bytes();


      let mut curl_handle = match self.curl_handle
      {
         Some(ref mut curl_handle) => 
         {
            curl_handle
         },
         _ =>
         {
            error!("No curl handle available.");
            return Err("No curl handle available.".to_string());
         }
      };

      //u := fmt.Sprintf("/chat_rooms/%d/chat_room_messages", req.RoomId)
      curl_handle.url(format!("{}/chat_rooms/{}/chat_room_messages", BIBA_BASE_ADDRESS, room_id).as_str()).unwrap();
      curl_handle.post(true).unwrap();
      curl_handle.post_field_size(message_data.len() as u64).unwrap();

      let mut transfer = curl_handle.transfer();
      transfer.read_function
      (
         |buf|
         {
            Ok(message_data.read(buf).unwrap_or(0))
         }
      ).unwrap();

      try!
      (
         transfer.perform().map_err(|err| err.to_string())
      );

      Ok(())
   }

}


impl<'a> Drop for Connector<'a>
{
   fn drop(&mut self)
   {
      info!("DROP CALLED!");
   }
}

