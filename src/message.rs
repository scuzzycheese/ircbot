use std::collections::HashMap;
use std::str;

pub enum MessageType
{
   Unknown,
   PrivateMessage,
   ChannelMessage,
}

pub enum System
{
   IRC,
}

pub struct Message
{
   pub original_data: Vec<u8>,
   pub system: System,
   pub message_type: MessageType,

   pub origin_who: Option<(usize, usize)>,
   pub origin_channel: Option<(usize, usize)>,
   pub message: Option<(usize, usize)>,

   pub type_specific_keys: HashMap<&'static str, (usize, usize)>,
}


//Generic message methods
pub trait MessageFields 
{
   fn get_origin_who(&self) -> Option<&str>;
   fn get_origin_channel(&self) -> Option<&str>;
   fn get_message(&self) -> Option<&str>;
}

impl MessageFields for Message
{
   fn get_origin_who(&self) -> Option<&str>
   {
      match self.origin_who
      {
         Some((start, end)) =>
         {
            let original_message: &str = str::from_utf8(&self.original_data).unwrap();
            Some(&original_message[start .. end])
         },
         _ => { None }
      }
   }


   fn get_origin_channel(&self) -> Option<&str>
   {
      match self.origin_channel
      {
         Some((start, end)) =>
         {
            let original_message: &str = str::from_utf8(&self.original_data).unwrap();
            Some(&original_message[start .. end])
         },
         _ => { None }
      }
   }

   fn get_message(&self) -> Option<&str>
   {
      match self.message
      {
         Some((start, end)) =>
         {
            let original_message: &str = str::from_utf8(&self.original_data).unwrap();
            Some(&original_message[start .. end])
         },
         _ => { None }
      }
   }


}
