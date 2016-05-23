use std::str;
use std::io::prelude::*;
use std::net::TcpStream;
use std::io;
use std::collections::HashMap;

use irc::keys::IRCFields;
use message::*;
pub mod keys;

pub struct Connector<'a>
{
   sock: &'a mut TcpStream,
   nick: &'a str,
   buf: [u8; 512],
   need_to_read: bool,
   start: usize,
   end: usize,
}


impl<'a> Iterator for Connector<'a>
{
   type Item = Message;

   //This function needs to return one message at a time (ie: one line at a time)
   fn next(&mut self) -> Option<Message> 
   {
      let mut message_vec: Vec<u8>; 
      let mut message: Message;

      loop
      {
         let mut must_break: bool = true;
         message_vec = Vec::with_capacity(512);

         let mut found_cr: bool = false;
         let mut found_lf: bool = false;

         while !found_cr && !found_lf
         {
            if self.need_to_read
            {
               //println!("Reading from socket...");
               let read_result = self.sock.read(&mut self.buf);

               self.end = match read_result
               {
                  Ok(read_length) => read_length,
                  _ => { return None; },
               }
            }


            for chr in (&self.buf[self.start..self.end]).iter()
            {
               self.start += 1;

               message_vec.push(*chr);
               match *chr as char
               {
                  //TODO: make sure these are sequential
                  '\r' => { found_cr = true; },
                  '\n' => 
                  {
                     found_lf = true;
                     if found_cr 
                     {
                        self.need_to_read = false;
                        break;
                     }
                  },
                  _ => (),
               }
            }

            if !found_cr && !found_lf 
            { 
               self.buf = [0; 512];
               self.start = 0;
               self.end = 0;
               self.need_to_read = true; 
            }
         }

         
         message = self.parse_message(message_vec).unwrap();

         println!("Message:");
         println!("  prefix: {}", match message.get_prefix() {Some(m) => m, None => "None"});
         println!("  command: {}", match message.get_command(){Some(m) => m, None => "None"});
         println!("  params: {}", match message.get_params(){Some(m) => m, None => "None"});
         println!("  trailing: {}", match message.get_trailing(){Some(m) => m, None => "None"});
         
         match message.get_command().unwrap()
         {
				"PING" =>
            {
               match self.ping_pong(&message) 
               {
                  Ok(_) => { must_break = false; },
                  Err(e) => { println!("Error sending PONG to server: {}", e); return None },
               }
            },
            _ => {}
         }
         if must_break { break; }
      }


      Some(message)
   }

}

impl<'a> Connector<'a>
{
   pub fn new(sock: &'a mut TcpStream, nick: &'a str) -> Connector<'a>
   {
      Connector
      {
         sock: sock,
         nick: nick,
         buf: [0; 512],
         need_to_read: true,
         start: 0,
         end: 0,
      }   
   }

   pub fn privmsg(&mut self, message: &str, dest: &str) -> io::Result<usize>
   {
      let send_string = format!("PRIVMSG {} :{}\r\n", dest, message);
      println!("SENDING ----------> {}", send_string);
      try!(self.sock.write(send_string.as_bytes()));
      Ok(0)
   }

   pub fn connect(&mut self) -> io::Result<usize>
   {
      try!(self.sock.write(format!("NICK {}\r\n", self.nick).as_bytes()));
      try!(self.sock.write(format!("USER {} 2 * : {}\r\n", self.nick, self.nick).as_bytes()));
      Ok(0)
   }

   pub fn join_channel(&mut self, channel_name: &str) -> io::Result<usize>
   {
      try!(self.sock.write(format!("JOIN {}\r\n", channel_name).as_bytes()));
      Ok(0)
   }


   fn ping_pong(&mut self, message: &Message) -> io::Result<usize>
   {
      let pong_resp = format!("PONG {}", message.get_trailing().unwrap());
      println!("Send -> {}", pong_resp);
      try!(self.sock.write(pong_resp.as_bytes()));
      Ok(0)
   }

   fn parse_message(&mut self, message_vec: Vec<u8>) -> Result<Message, &'static str>
   {

      let message_vec_clone = message_vec.clone();
		let message_slice: &[u8] = message_vec_clone.as_slice();

      let original_data: &str = str::from_utf8(message_slice).unwrap();
      print!("SRV_MESSAGE: {}", original_data);

      let mut message_chars = original_data.char_indices().peekable();

      let mut prefix: Option<(usize, usize)> = None;
      let mut command: Option<(usize, usize)> = None;
      let mut params: Option<(usize, usize)> = None;
      let mut trailing: Option<(usize, usize)> = None;

      let mut start_index = 0;
      let mut word_counter = 0;
      while let Some(message_char) = message_chars.next()
      {
         let (index, char) = message_char;

         if ' ' == char 
         {
            let mut message_chars = original_data.chars();
            match word_counter
            {
               0 => 
               {
                  
                  if message_chars.next() == Some(':')
                  {
                     prefix = Some((start_index, index));
                  }
                  else
                  {
                     command = Some((start_index, index));
                     word_counter = word_counter + 1;
                  } 
                  
               },
               1 => 
               {
                  command = Some((start_index, index));
               },
               _ => 
               {
                  break;
               },
            }
            word_counter = word_counter + 1;
            start_index = index + 1;
         }

      }
      //look for the next ':'
      let trailing_index = original_data[start_index .. original_data.len()].find(':');

      //lastly we parse out the params and trailing
      params = match trailing_index
      {
         Some(x) => 
         {
            trailing = Some((start_index + x, original_data.len()));
            Some((start_index, start_index + x))

         },
         _ =>
         {
            Some((start_index, original_data.len()))
         } 
      };

      //Add all the irc specific keys to a hash
      let mut key_hash: HashMap<&'static str, (usize, usize)> = HashMap::new();
      match prefix { Some(x) => { key_hash.insert(keys::Keys::Prefix.string(), x); } _ => {}};
      match command { Some(x) => { key_hash.insert(keys::Keys::Command.string(), x); } _ => {}};
      match params { Some(x) => { key_hash.insert(keys::Keys::Params.string(), x); } _ => {}};
      match trailing { Some(x) => { key_hash.insert(keys::Keys::Trailing.string(), x); } _ => {}};

      //TODO: I need to understand why I have to make a reference below since it's already an &str
      let message_struct: Message = match command
      {
         Some((command_start, command_end)) => 
         {
            match &original_data[command_start .. command_end]
            {
               "PRIVMSG" =>
               {

                  let trailing_data = &original_data[trailing.unwrap().0 .. trailing.unwrap().1].split_at(1).1;

                  let from_nickname = parse_nickname(original_data);
                  println!("Private Message from {}: {}", from_nickname.unwrap(), trailing_data);


//                  if original_data.starts_with(nickname)
//                  {
//                     println!("Message Directed at me!!!");
//                  }
//                  else if original_data.contains(nickname)
//                  {
//
//                     let from_nickname = match parse_nickname(out_prefix)
//                     {
//                        Some(x) => x,
//                        None =>
//                        {
//                           println!("Error parsing Nickname!");
//                           "IMPROBABLENICKNAME"
//                        }
//                     };
//
//
//
//                     let destination;
//                     if out_params == nickname
//                     {
//                        destination = from_nickname;
//                     }
//                     else
//                     {
//                        destination = out_params;
//                     }
//
//
//                     if original_data.to_uppercase().starts_with("HI")
//                     {
//                        irc.privmsg(&format!("Hi {}", from_nickname), destination);
//                     }
//                   }


                  Message
                  {
                     original_data: message_vec,
                     system: System::IRC,
                     message_type: MessageType::PrivateMessage,

                     origin_who: Some((0, 0)),
                     origin_channel: Some((0, 0)),
                     message: Some((0, 0)),

                     type_specific_keys: key_hash
                  }
               },
               _ => 
               {
                  Message
                  {
                     original_data: message_vec,
                     system: System::IRC,
                     message_type: MessageType::Unknown,

                     origin_who: Some((0, 0)),
                     origin_channel: Some((0, 0)),
                     message: Some((0, 0)),

                     type_specific_keys: key_hash
                  }
               }
            }
         },
         _ =>
         {
            Message
            {
               original_data: message_vec,
               system: System::IRC,
               message_type: MessageType::Unknown,

               origin_who: Some((0, 0)),
               origin_channel: Some((0, 0)),
               message: Some((0, 0)),

               type_specific_keys: key_hash
            }
         }
      };

      Ok(message_struct)
   }
}


fn parse_nickname(params: &str) -> Option<&str>
{
   println!("PARAMS: {}", params);
   params.split(|c| c == ':' || c == '!').nth(1)
}





