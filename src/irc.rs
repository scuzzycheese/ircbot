use std::str;
use std::io::prelude::*;
use std::net::TcpStream;
use std::io;

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

		let message_slice: &[u8] = message_vec.as_slice();

      let message_string: &str = str::from_utf8(message_slice).unwrap();
      print!("String: {}", message_string);

      let message_parts = message_string.match_indices(' ');

      let mut prefix: Option<(usize, usize)> = None;
      let mut command: Option<(usize, usize)> = None;
      let mut params: Option<(usize, usize)> = None;
      let mut trailing: Option<(usize, usize)> = None;

      let mut current_start: usize = 0;
      let mut current_count: usize = 0;
      let mut first_skip = false;
      //TODO: fix this parsing order
      for message_part in message_parts
      {
         if !first_skip 
         {
            first_skip = true;
            current_start = 0;
            continue;
         }

         if current_count > 2
         {
            break;
         }
         current_count = current_count + 1;

         let (index, match_str) = message_part;
         println!("current_start: {}, current_count: {}, index: {}", current_start, current_count, index);


         if match_str.starts_with(":") && prefix == None
         {
            prefix = Some((current_start, index));
            current_start = index;
            continue;
         }

         if match_str.starts_with(":") && prefix != None
         {
            trailing = Some((current_start, index));
            current_start = index;
            continue;
         }

         if command == None 
         {
            command = Some((current_start, index));
            current_start = index;
            continue;
         }
         else
         {
            params = Some((current_start, index));
            current_start = index;
            continue;
         }
      }

      println!("Looking for items to respond to");

      let (command_start, command_end) = command.unwrap();
      //TODO: I need to understand why I have to make a reference below since it's already an &str
      let message_struct = match &message_string[command_start .. command_end]
      {
         "PRIVMSG" =>
         {
            Message
            {
               message_type: MessageType::PrivateMessage,
               message_string: message_string.to_string(),
               prefix: prefix,
               command: command,
               params: params,
               trailing: trailing,
            }
         },
         _ => 
         {
            Message
            {
               message_type: MessageType::Unknown,
               message_string: message_string.to_string(),
               prefix: prefix,
               command: command,
               params: params,
               trailing: trailing,
            }
         },
      };

      Ok(message_struct)
   }
}


pub enum MessageType
{
   Unknown,
   PrivateMessage,
   ChannelMessage,
}

pub struct Message
{
   pub message_type: MessageType,

   pub message_string: String,

   pub prefix: Option<(usize, usize)>,
   pub command: Option<(usize, usize)>,
   pub params: Option<(usize, usize)>,
   pub trailing: Option<(usize, usize)>,
}


impl Message
{
   pub fn get_prefix(&self) -> Option<&str>
   {
      match self.prefix
      {
         Some((start, end)) =>
         {
            println!("prefix_start: {}, prefix_end: {}", start, end);
            Some(&self.message_string[start .. end])
         },
         _ => { None }
      }
   }

   pub fn get_command(&self) -> Option<&str>
   {
      match self.command
      {
         Some((start, end)) =>
         {
            println!("command_start: {}, command_end: {}", start, end);
            Some(&self.message_string[start .. end])
         },
         _ => { None }
      }
   }

   pub fn get_params(&self) -> Option<&str>
   {
      match self.params
      {
         Some((start, end)) =>
         {
            Some(&self.message_string[start .. end])
         },
         _ => { None }
      }
   }

   pub fn get_trailing(&self) -> Option<&str>
   {
      match self.trailing
      {
         Some((start, end)) =>
         {
            Some(&self.message_string[start .. end])
         },
         _ => { None }
      }
   }

}

