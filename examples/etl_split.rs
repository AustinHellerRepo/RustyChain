use std::io::Write;
use rusty_chain::chain::ChainLink;
use tempfile::NamedTempFile;

use crate::etl::{etl_process::{EtlProcess, EtlProcessInitializer}, read_file::ReadFromFileInitializer, parse::ParseStringToCustomerInitializer, separate_database::SeparateDatabaseSplitMergeInitializer, database::{InsertCustomerIntoDatabaseInitializer, DatabaseRepository}};

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
                    None
                }
                else {
                    Some(output)
                }
            }
            else {
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
        use std::time::Duration;

        use rusty_chain::chain_link;
        use super::models::Customer;

        pub struct DatabaseRepository {
            pub name: String
        }

        impl DatabaseRepository {
            pub async fn insert_customer(&self, customer: &Customer) {
                // make the mirror database take a little longer than the primary database
                if self.name.as_str() == "Primary" {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
                else {
                    tokio::time::sleep(Duration::from_millis(300)).await;
                }
                // inserts into database
                println!("DatabaseRepository: inserted customer {} with age {} into datababase {}", customer.customer_name, customer.age, self.name);
            }
        }

        chain_link!(InsertCustomerIntoDatabase => (repository: DatabaseRepository), input: Customer => bool, {
            match input.received {
                Some(received) => {
                    input.initializer.read().await.repository.insert_customer(&*received.read().await).await;
                    Some(true)
                },
                None => None
            }
        });
    }

    pub mod separate_database {
        use rusty_chain::split_merge;
        use super::{models::Customer, database::{InsertCustomerIntoDatabase, InsertCustomerIntoDatabaseInitializer}};

        split_merge!(SeparateDatabaseSplitMerge, Customer => bool, (InsertCustomerIntoDatabase, InsertCustomerIntoDatabase), join);
    }

    // example filename: "etl_process.rs"
    pub mod etl_process {
        use rusty_chain::chain;
        use super::{read_file::{ReadFromFile, ReadFromFileInitializer}, parse::{ParseStringToCustomer, ParseStringToCustomerInitializer}, separate_database::{SeparateDatabaseSplitMerge, SeparateDatabaseSplitMergeInitializer}};

        chain!(EtlProcess, String => bool, ReadFromFile => ParseStringToCustomer => SeparateDatabaseSplitMerge);
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
    let etl_process = EtlProcess::new(EtlProcessInitializer { x_read_from_file: ReadFromFileInitializer { buffer: None }, xx_parse_string_to_customer: ParseStringToCustomerInitializer { }, xxx_separate_database_split_merge: SeparateDatabaseSplitMergeInitializer { x_insert_customer_into_database_initializer: InsertCustomerIntoDatabaseInitializer { repository: DatabaseRepository { name: String::from("Primary")} }, xx_insert_customer_into_database_initializer: InsertCustomerIntoDatabaseInitializer { repository: DatabaseRepository { name: String::from("Mirror") } } } });

    // pass in files
    fn get_path_as_string(path: &std::path::Path) -> String {
        (*path).as_os_str().to_str().unwrap().to_string()
    }
    etl_process.push_raw(get_path_as_string(first_file.path())).await;
    etl_process.push_raw(get_path_as_string(second_file.path())).await;

    // run ETL process until completed
    // this methodology of checking for successful pops only works because the split_merge is joined
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