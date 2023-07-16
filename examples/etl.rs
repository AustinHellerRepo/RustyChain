use std::io::Write;

use etl::{EtlProcess, EtlProcessInitializer, ReadFromFileInitializer, ParseStringToCustomerInitializer, InsertCustomerIntoDatabaseInitializer, DatabaseRepository};
use rusty_chain::chain::ChainLink;
use tempfile::NamedTempFile;

mod etl {
    use std::{io::{BufRead, BufReader, SeekFrom, Seek}, fs::File};
    use nom::{bytes::complete::tag, sequence::tuple, IResult};
    use nom::sequence::delimited;

    use rusty_chain::{chain_link, chain};

    chain_link!(ReadFromFile => (buffer: Option<BufReader<File>>), input: String => String, {
        println!("ReadFromFile processing: {:?}", input.received);
        if let Some(file_path) = input.received {
            // store the file buffer in the initializer
            let mut file = File::open(file_path.clone()).expect("The file should open.");
            file.seek(SeekFrom::Start(0)).expect("The file should return to the front.");
            let mut read_buffer = BufReader::new(file);
            read_buffer.seek(SeekFrom::Start(0)).expect("The read buffer should return to the front.");
            let _ = input.initializer.lock().await.buffer.replace(read_buffer);
        }

        let mut locked_initializer = input.initializer.lock().await;
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

    #[derive(Debug)]
    pub struct Customer {
        pub customer_name: String,
        pub age: u8
    }

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
        println!("ParseStringToCustomer processing: {:?}", input.received);
        match input.received {
            Some(received) => {
                let customer: Customer;
                {
                    //let mut parser = delimited(tag(""), tag(","), tag("\n"));
                    //let mut parser = tuple(fold_many1(alpha, space, alpha), tag_s!(","), digit);
                    
                    // this helps nom determine which error to return
                    assert!(matches!(parse_customer("1,2"), Err(_)));

                    // parse the file line
                    {
                        let (input, parsed_customer) = parse_customer(received.as_str()).unwrap();
                        assert_eq!("\n", input);
                        customer = parsed_customer;
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
            println!("DatabaseRepository: inserted customer {} with age {}", customer.customer_name, customer.age);
        }
    }

    chain_link!(InsertCustomerIntoDatabase => (repository: DatabaseRepository), input: Customer => bool, {
        println!("InsertCustomerIntoDatabase processing: {:?}", input.received);
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
    let mut first_file = NamedTempFile::new().unwrap();
    let mut second_file = NamedTempFile::new().unwrap();

    writeln!(first_file, "John Smith,28").unwrap();
    writeln!(first_file, "Jane Jackson,59").unwrap();
    writeln!(second_file, "Adam Allison,31").unwrap();
    writeln!(second_file, "Brady Brickly,32").unwrap();
    writeln!(second_file, "Charlie Chucks,43").unwrap();

    println!("first file size: {}", first_file.as_file().metadata().unwrap().len());
    println!("second file size: {}", second_file.as_file().metadata().unwrap().len());

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

    first_file.close().expect("The first file should close.");
    second_file.close().expect("The second file should close.");
}