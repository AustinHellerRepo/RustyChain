use std::{io::Write, cell::RefCell};

use etl::{EtlProcess, EtlProcessInitializer, ReadFromFileInitializer, ParseStringToCustomerInitializer, InsertCustomerIntoDatabaseInitializer, DatabaseRepository};
use rusty_chain::chain::ChainLink;
use tempfile::NamedTempFile;

mod etl {
    use std::{io::{BufRead, BufReader}, fs::File};
    use nom::bytes::complete::tag;
    use nom::sequence::delimited;

    use rusty_chain::{chain_link, chain};
    use tokio::sync::Mutex;

    chain_link!(ReadFromFile => (buffer: Option<BufReader<File>>), input: String => String, {
        if let Some(file_path) = input.received {
            // store the file buffer in the initializer
            let file = File::open(file_path.clone()).unwrap();
            let read_buffer = BufReader::new(file);
            let _ = input.initializer.lock().await.buffer.replace(read_buffer);
        }

        let locked_initializer = input.initializer.lock().await;
        if let Some(buffer) = locked_initializer.buffer.as_ref() {
            // read the next line from the file
            let mut output: String = String::default();
            let read_bytes_count = buffer
                .buffer()
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

    pub struct Customer {
        pub customer_name: String,
        pub age: u8
    }

    chain_link!(ParseStringToCustomer, input: String => Customer, {
        match input.received {
            Some(received) => {
                let customer: Customer;
                {
                    let mut parser = delimited(tag(""), tag(","), tag(""));
                    
                    // this helps nom determine which error to return
                    assert!(matches!(parser("1.2"), Err(nom::Err::Error(nom::error::VerboseError {..}))));

                    // parse the file line
                    {
                        let (customer_name, age_string) = parser(received.as_str()).unwrap();
                        customer = Customer {
                            customer_name: String::from(customer_name),
                            age: age_string.parse::<u8>().unwrap()
                        };
                    }
                }
                Some(customer)
            },
            None => {
                None
            }
        }
    });

    pub struct DatabaseRepository { }

    impl DatabaseRepository {
        pub fn insert_customer(&self, customer: &Customer) {
            // inserts into database
            println!("inserted customer {} with age {}", customer.customer_name, customer.age);
        }
    }

    chain_link!(InsertCustomerIntoDatabase => (repository: DatabaseRepository), input: Customer => bool, {
        match input.received {
            Some(received) => {
                input.initializer.lock().await.repository.insert_customer(&*received);
                Some(true)
            },
            None => None
        }
    });

    chain!(EtlProcess, String => bool, ReadFromFile => ParseStringToCustomer => InsertCustomerIntoDatabase);
}

#[tokio::main]
async fn main() {

    // create a couple test files
    let first_file = NamedTempFile::new().unwrap();
    let second_file = NamedTempFile::new().unwrap();

    first_file.as_file().write_all(b"John Smith,28\nJane Jackson,59").unwrap();
    second_file.as_file().write_all(b"Adam Allison,31\nBrady Brickly,32\nCharlie Chucks,43").unwrap();

    // setup chain
    let mut etl_process = EtlProcess::new(EtlProcessInitializer { x_read_from_file: ReadFromFileInitializer { buffer: None }, xx_parse_string_to_customer: ParseStringToCustomerInitializer { }, xxx_insert_customer_into_database: InsertCustomerIntoDatabaseInitializer { repository: DatabaseRepository { }}});

    // pass in files
    async fn push_file(path: &std::path::Path, etl_process: &mut EtlProcess) {
        etl_process.push_raw((*path).as_os_str().to_str().unwrap().to_string()).await;
    }
    push_file(first_file.path(), &mut etl_process).await;
    push_file(second_file.path(), &mut etl_process).await;

    // run ETL process until completed
    let mut is_successful = true;
    while is_successful {
        etl_process.process().await;
        match etl_process.try_pop().await {
            Some(popped_is_successful) => {
                is_successful = *popped_is_successful.lock().await;
            },
            None => {
                is_successful = false;
            }
        }
    }
}