use std::io::Write;
use tempfile::NamedTempFile;
use rusty_chain::framework::ChainLink;

use crate::etl::{etl_process::{EtlProcess, EtlProcessInitializer}, read_file::ReadFromFileInitializer, parse::ParseStringToCustomerInitializer, database::{InsertCustomerIntoDatabaseInitializer, DatabaseRepository}};

// each module could exist in a separate file for better maintainability
mod etl {

    // example filename: "read_file.rs"
    pub mod read_file {
        use std::{io::{BufReader, SeekFrom, Seek, BufRead}, fs::File};
        use rusty_chain::chain_link;

        chain_link!(ReadFromFile => (buffer: Option<BufReader<File>>), input: String => String, {
            if let Some(file_path) = input.received {
                // store the file buffer in the initializer
                let mut file = File::open(file_path.read().await.clone()).expect("The file should open.");
                file.seek(SeekFrom::Start(0)).expect("The file should return to the front.");
                let mut read_buffer = BufReader::new(file);
                read_buffer.seek(SeekFrom::Start(0)).expect("The read buffer should return to the front.");
                let _ = input.initializer.write().await.buffer.replace(read_buffer);
            }

            let mut locked_initializer = input.initializer.write().await;
            if let Some(buffer) = locked_initializer.buffer.as_mut() {
                // read the next line from the file
                let mut output: String = String::default();
                let read_bytes_count = buffer
                    .read_line(&mut output)
                    .expect("The buffer should return a line.");
                if read_bytes_count == 0 {
                    // returning None informs the process that the next file path should be supplied
                    None
                }
                else {
                    // return the file line
                    Some(output)
                }
            }
            else {
                // return None if the file path hasn't been provided yet
                None
            }
        });
    }

    // example filename: "models.rs"
    pub mod models {

        // this is in a separate module so that it can be conveniently shared between dependent modules
        #[derive(Debug)]
        pub struct Customer {
            pub customer_name: String,
            pub age: u8
        }
    }

    // example filename: "parse.rs"
    pub mod parse {
        use nom::{bytes::complete::tag, IResult};
        use rusty_chain::chain_link;
        use super::models::Customer;

        fn parse_name_part(input: &str) -> IResult<&str, &str> {
            let (input, name_part) = nom::character::complete::alpha1(input)?;
            Ok((input, name_part))
        }

        fn parse_full_name(input: &str) -> IResult<&str, String> {
            let (input, first_name) = parse_name_part(input)?;
            let (input, _) = tag(" ")(input)?;
            let (input, last_name) = parse_name_part(input)?;
            Ok((input, format!("{} {}", first_name, last_name)))
        }

        fn parse_age(input: &str) -> IResult<&str, u8> {
            let (input, age) = nom::character::complete::u8(input)?;
            Ok((input, age))
        }

        fn parse_customer(input: &str) -> IResult<&str, Customer> {
            let (input, name) = parse_full_name(input)?;
            let (input, _) = tag(",")(input)?;
            let (input, age) = parse_age(input)?;
            Ok((input, Customer {
                customer_name: name,
                age
            }))
        }

        chain_link!(ParseStringToCustomer, input: String => Customer, {
            match input.received {
                Some(received) => {

                    // parse the file line using nom
                    let (_, parsed_customer) = parse_customer(received.read().await.as_str()).unwrap();

                    // return the parsed Customer instance
                    Some(parsed_customer)
                },
                None => None
            }
        });
    }

    // example filename: "database.rs"
    pub mod database {
        use rusty_chain::chain_link;
        use super::models::Customer;

        // this represents an abstract layer over the database interactions
        pub struct DatabaseRepository { }

        impl DatabaseRepository {
            pub fn insert_customer(&self, customer: &Customer) {
                // pretends that it inserts into database
                println!("DatabaseRepository: inserted customer {} with age {}", customer.customer_name, customer.age);
            }
        }

        chain_link!(InsertCustomerIntoDatabase => (repository: DatabaseRepository), input: Customer => bool, {
            match input.received {
                Some(received) => {
                    // when the Customer is supplied, perform the insert into the database
                    input.initializer.read().await.repository.insert_customer(&*received.read().await);
                    Some(true)
                },
                None => None
            }
        });
    }

    // example filename: "etl_process.rs"
    pub mod etl_process {
        use rusty_chain::chain;
        use super::{read_file::{ReadFromFile, ReadFromFileInitializer}, parse::{ParseStringToCustomer, ParseStringToCustomerInitializer}, database::{InsertCustomerIntoDatabase, InsertCustomerIntoDatabaseInitializer}};

        // this single line creates the EtlProcess chain
        chain!(EtlProcess,
            String => bool,
            [
                ReadFromFile => ParseStringToCustomer => InsertCustomerIntoDatabase
            ]: (all join)
        );
    }
}

#[tokio::main]
async fn main() {

    // create a couple test files
    let mut first_file = NamedTempFile::new().unwrap();
    let mut second_file = NamedTempFile::new().unwrap();

    writeln!(first_file, "John Smith,28").unwrap();
    writeln!(first_file, "Jane Jackson,59").unwrap();
    writeln!(second_file, "Adam Allison,31").unwrap();
    writeln!(second_file, "Brady Brickly,32").unwrap();
    writeln!(second_file, "Charlie Chucks,43").unwrap();

    // setup chain
    let etl_process = EtlProcess::new_raw(
        EtlProcessInitializer::new(
            ReadFromFileInitializer {
                buffer: None
            },
            ParseStringToCustomerInitializer { },
            InsertCustomerIntoDatabaseInitializer { 
                repository: DatabaseRepository { }
            }
        )
    ).await;

    // pass in files
    fn get_path_as_string(path: &std::path::Path) -> String {
        (*path).as_os_str().to_str().unwrap().to_string()
    }
    etl_process.push_raw(get_path_as_string(first_file.path())).await;
    etl_process.push_raw(get_path_as_string(second_file.path())).await;

    // run ETL process until completed
    let mut is_successful = true;
    while is_successful {
        etl_process.process().await;
        match etl_process.try_pop().await {
            Some(popped_is_successful) => {
                is_successful = *popped_is_successful.read().await;
            },
            None => {
                is_successful = false;
            }
        }
    }

    first_file.close().expect("The first file should close.");
    second_file.close().expect("The second file should close.");
}