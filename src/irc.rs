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


impl<'a> Iterator<Vec<u8>> for Connector<'a>
{
   //This function needs to return one message at a time (ie: one line at a time)
   fn next(&mut self) -> Option<Vec<u8>> 
   {
      let mut message: Vec<u8>; 

      loop
      {
         let mut must_break: bool = true;
         message = Vec::with_capacity(512);

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

               message.push(*chr);
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

         let message_string = str::from_utf8(message.as_slice()).unwrap();
         if message_string.starts_with("PING")
         {
            match self.ping_pong(message_string) 
            {
               Ok(_) => { must_break = false; },
               Err(e) => { println!("Error sending PONG to server: {}", e); },
            }
         }
         //print!("Start: {} Message: {}", self.start, str::from_utf8(message.as_slice()).unwrap());
         //print!("Start: {} Message: {}", self.start, message);
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


   fn ping_pong(&mut self, read_string: &str) -> IoResult<uint>
   {
      //println!("Initial string len: {}", read_string.len());
      let mut ping_parts = read_string.splitn(1, ' ');
      //println!("First String len: {}", ping_parts.next().unwrap().len());
      let pong_resp = ping_parts.next().unwrap();
      //println!("pong_resp length: {}", pong_resp.len());
      println!("Send -> PONG {}", pong_resp);
      try!(self.sock.write(format!("PONG {}", pong_resp).as_bytes()));
      Ok(0)
   }

   fn parse_message(&mut self, message: &Vec<u8>)
   {
   }
}


pub enum MessageType
{
   PrivateMessage,
   ChannelMessage,
}

pub struct Message<'a>
{
   message_type: MessageType,
   message: &'a Vec<u8>,
}
