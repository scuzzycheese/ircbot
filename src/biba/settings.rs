use sqlite;


pub struct Settings
{
   connection: sqlite::Connection
}

pub fn new() -> Settings
{
   let connection = sqlite::open("biba.db").unwrap();
   build_biba_settings(&connection);
   build_biba_cache(&connection);

   Settings
   {
      connection: connection
   } 

}


impl Settings
{


   pub fn add_key_to_db(&mut self, key: &str, value: &str)
   {
      let key_insert_result = self.connection.execute(format!("INSERT INTO biba_cache(key, value) VALUES ('{}', '{}');", key, value));
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

   pub fn get_key_from_db(&mut self, key: &str) -> Result<(String, i64), String>
   {
      let mut statement = self.connection.prepare("SELECT * FROM biba_cache WHERE key = ?").unwrap();      
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



   pub fn get_setting_value(&mut self, key: &str) -> Result<String, String>
   {
      let mut statement = self.connection.prepare("SELECT * FROM biba_settings WHERE key = ?").unwrap();      
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

pub fn build_biba_settings(connection: &sqlite::Connection)
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

fn build_biba_cache(connection: &sqlite::Connection)
{
   let create_table_result = connection.execute("CREATE TABLE biba_cache(key TEXT PRIMARY KEY, value TEXT, expiry INTEGER);");
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


