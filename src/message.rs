

pub enum MessageType
{
   Unknown,
   PrivateMessage,
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
            Some(&self.message_string[start .. end].trim())
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
            Some(&self.message_string[start .. end].trim())
         },
         _ => { None }
      }
   }

}
