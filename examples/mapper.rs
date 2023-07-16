use std::{sync::Arc, time::Duration};
use tokio::sync::Mutex;

mod mapper_example {

    use rusty_chain::chain_link;

    pub struct ChildRecord {
        parent_id: i32,
        image_bytes: Vec<u8>
    }

    pub struct DatabaseConnection {}

    impl DatabaseConnection {
        pub fn new(_connection_string: String) -> Self {
            DatabaseConnection { }
        }
    }

    impl DatabaseConnection {
        pub fn get_parent_by_parent_id(&self, parent_id: i32) -> ParentRecord {
            ParentRecord {
                parent_id,
                name: String::from("Some name")
            }
        }
        pub fn get_child_records_by_parent_id(&self, parent_id: i32) -> Vec<ChildRecord> {
            // return two child records
            vec![
                ChildRecord {
                    parent_id,
                    image_bytes: vec![0, 1, 2, 3]
                }, ChildRecord {
                    parent_id,
                    image_bytes: vec![4, 5, 6, 7]
                }
            ]
        }
    }

    pub struct ParentRecord {
        parent_id: i32,
        name: String
    }

    #[derive(Debug)]
    pub struct ParentModel {
        pub parent_id: i32,
        pub name: String,
        pub children_image_bytes: Vec<Vec<u8>>
    }

    pub struct GetParentByIdInput {
        parent_id: i32
    }

    impl GetParentByIdInput {
        pub fn new(parent_id: i32) -> Self {
            GetParentByIdInput {
                parent_id
            }
        }
    }

    chain_link!(GetParentById => (connection_string: String), input: GetParentByIdInput => ParentModel, {
        // the connection string was part of the initializer, so we can create our database connection on demand
        let database_connection = DatabaseConnection::new(input.initializer.connection_string.clone());
        let parent_record = database_connection.get_parent_by_parent_id(input.received.parent_id);
        let child_records = database_connection.get_child_records_by_parent_id(input.received.parent_id);

        // just checking that the data matches expectations
        assert_eq!(input.received.parent_id, parent_record.parent_id);

        ParentModel {
            parent_id: parent_record.parent_id,
            name: parent_record.name,
            children_image_bytes: child_records
                .into_iter()
                .map(|cr| {
                    // another check that data matches expectations
                    assert_eq!(parent_record.parent_id, cr.parent_id);
                    cr.image_bytes
                })
                .collect()
        }
    });
}


#[tokio::main]
async fn main() {
    use mapper_example::*;
    use rusty_chain::chain::*;

    let mapper = Arc::new(Mutex::new(GetParentById::new(GetParentByIdInitializer { connection_string: String::from("get from settings") })));
    
    // one thread is queueing up parent models to be received on the other end
    let receive_mapper = mapper.clone();
    let receive_task = tokio::task::spawn(async {
        let mapper = receive_mapper;
        for index in 0..10 {
            tokio::time::sleep(Duration::from_secs(1)).await;
            println!("receiving {}...", index);
            mapper.lock().await.receive(Arc::new(Mutex::new(GetParentByIdInput::new(index)))).await;
            println!("received {}.", index);
        }
    });

    // another thread polls faster than it may receive data
    // this allows chainlink data migration to be adjusted dynamically at runtime
    // this is not a requirement - you could just poll before each send
    let join_mapper = mapper.clone();
    let poll_task = tokio::task::spawn(async {
        let mapper = join_mapper;
        for _ in 0..20 {
            tokio::time::sleep(Duration::from_millis(500)).await;
            println!("polling...");
            mapper.lock().await.poll().await;
            println!("polled.");
        }
    });

    // this thread is pulling out parent models on an interval
    let send_mapper = mapper.clone();
    let send_task = tokio::task::spawn(async {
        let mapper = send_mapper;
        for _ in 0..10 {
            tokio::time::sleep(Duration::from_millis(1200)).await;
            println!("sending...");
            let model = mapper.lock().await.send().await;
            match model {
                Some(model) => {
                    let locked_model = model.lock().await;
                    println!("sent {:?}", locked_model);
                },
                None => {
                    panic!("Unexpected None result.");
                }
            }
        }
    });

    let result = tokio::join!(receive_task, poll_task, send_task);
    result.0.expect("The receive task should join properly.");
    result.1.expect("The receive task should join properly.");
    result.2.expect("The receive task should join properly.");

    println!("Successful!");
}