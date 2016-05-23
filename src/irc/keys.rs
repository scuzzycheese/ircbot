use message::Message;
use std::str;

pub enum Keys
{
   Prefix,
   Command,
   Params,
   Trailing,
}

//IRC specific keys
impl Keys 
{
   pub fn string(&self) -> &'static str
   {
      match *self 
      {
         Keys::Prefix => "Prefix",
         Keys::Command => "Command",
         Keys::Params => "Params",
         Keys::Trailing => "Trailing",
      }
   }
}

//IRC specific methods
pub trait IRCFields 
{
   fn get_prefix(&self) -> Option<&str>;
   fn get_command(&self) -> Option<&str>;
   fn get_params(&self) -> Option<&str>;
   fn get_trailing(&self) -> Option<&str>;
}

impl IRCFields for Message
{
   fn get_prefix(&self) -> Option<&str>
   {
      match self.type_specific_keys.get(Keys::Prefix.string())
      {
         Some(&(start, end)) =>
         {
            let original_message: &str = str::from_utf8(&self.original_data).unwrap();
            Some(&original_message[start .. end])
         },
         _ => { None }
      }
   }

   fn get_command(&self) -> Option<&str>
   {
      match self.type_specific_keys.get(Keys::Command.string())
      {
         Some(&(start, end)) =>
         {
            let original_message: &str = str::from_utf8(&self.original_data).unwrap();
            Some(&original_message[start .. end])
         },
         _ => { None }
      }
   }

   fn get_params(&self) -> Option<&str>
   {
      match self.type_specific_keys.get(Keys::Params.string())
      {
         Some(&(start, end)) =>
         {
            let original_message: &str = str::from_utf8(&self.original_data).unwrap();
            Some(&original_message[start .. end])
         },
         _ => { None }
      }
   }

   fn get_trailing(&self) -> Option<&str>
   {
      match self.type_specific_keys.get(Keys::Trailing.string())
      {
         Some(&(start, end)) =>
         {
            let original_message: &str = str::from_utf8(&self.original_data).unwrap();
            Some(&original_message[start .. end])
         },
         _ => { None }
      }
   }

}
