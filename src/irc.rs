use std::io::{TcpStream, IoResult};
use std::str;

pub struct Connector<'a>
{
   sock: &'a mut TcpStream,
   nick: &'a str,
   buf: [u8, ..512],
   need_to_read: bool,
   start: uint,
   end: uint,
}


impl<'a> Iterator<Message> for Connector<'a>
{
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
               println!("Reading from socket...");
               let read_result = self.sock.read(&mut self.buf);

               self.end = match read_result
               {
                  Ok(read_length) => read_length,
                  _ => { return None; },
               }
            }

            for chr in self.buf.slice(self.start, self.end).iter()
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
               self.buf = [0, ..512];
               self.start = 0u;
               self.end = 0u;
               self.need_to_read = true; 
            }
         }

         
         message = self.parse_message(&message_vec).unwrap();
         
         match message.command.as_ref()
         {
            Some(x) => 
            {
               if x.as_slice() == "PING"
               {
                  match self.ping_pong(&message) 
                  {
                     Ok(_) => { must_break = false; },
                     Err(e) => { println!("Error sending PONG to server: {}", e); return None },
                  }
               }
            },
            None => {}
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
         buf: [0, ..512],
         need_to_read: true,
         start: 0u,
         end: 0u,
      }   
   }

   pub fn connect(&mut self) -> IoResult<uint>
   {
      try!(self.sock.write(format!("NICK {}\r\n", self.nick).as_bytes()));
      try!(self.sock.write(format!("USER {} 2 * : {}\r\n", self.nick, self.nick).as_bytes()));
      Ok(0)
   }

   pub fn join_channel(&mut self, channel_name: &str) -> IoResult<uint>
   {
      try!(self.sock.write(format!("JOIN {}\r\n", channel_name).as_bytes()));
      Ok(0)
   }


   fn ping_pong(&mut self, message: &Message) -> IoResult<uint>
   {
      let pong_resp = format!("PONG {}", message.trailing);
      println!("Send -> {}", pong_resp);
      try!(self.sock.write(pong_resp.as_bytes()));
      Ok(0)
   }

   fn parse_message(&mut self, message: &Vec<u8>) -> Result<Message, &'static str>
   {
      let message_string: &str = str::from_utf8(message.as_slice()).unwrap();

      let mut message_parts = message_string.splitn(3, ' ');

      let mut prefix: Option<String> = None;
      let mut command: Option<String> = None;
      let mut params: Option<String> = None;
      let mut trailing: Option<String> = None;

      for message_part in message_parts
      {
         if message_part.starts_with(":") && prefix == None
         {
            prefix = Some(String::from_str(message_part));
            continue;
         }

         if message_part.starts_with(":") && prefix != None
         {
            trailing = Some(String::from_str(message_part));
            continue;
         }

         if command == None 
         {
            command = Some(String::from_str(message_part));
            continue;
         }
         else
         {
            params = Some(String::from_str(message_part));
            continue;
         }
      }

      let message_struct = match command.as_ref().unwrap().as_slice()
      {
         "PRIVMSG" =>
         {
            Message
            {
               message_type: MessageType::PrivateMessage,
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

   pub prefix: Option<String>,
   pub command: Option<String>,
   pub params: Option<String>,
   pub trailing: Option<String>,
}
