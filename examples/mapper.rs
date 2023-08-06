use std::{sync::Arc, time::Duration};

mod mapper_example {

    use std::time::Duration;

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
        pub async fn get_parent_by_parent_id(&self, parent_id: i32) -> ParentRecord {
            tokio::time::sleep(Duration::from_millis(500)).await;
            ParentRecord {
                parent_id,
                name: String::from("Some name")
            }
        }
        pub async fn get_child_records_by_parent_id(&self, parent_id: i32) -> Vec<ChildRecord> {
            // return two child records
            tokio::time::sleep(Duration::from_millis(500)).await;
            vec![
                ChildRecord {
                    parent_id,
                    image_bytes: vec![0, 1]
                }, ChildRecord {
                    parent_id,
                    image_bytes: vec![4, 5]
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
        match input.received {
            Some(parent_id_container) => {
                // the connection string was part of the initializer, so we can create our database connection on demand
                let database_connection = DatabaseConnection::new(input.initializer.read().await.connection_string.clone());
                let parent_id = parent_id_container.read().await.parent_id;
                let parent_record = database_connection.get_parent_by_parent_id(parent_id).await;
                let child_records = database_connection.get_child_records_by_parent_id(parent_id).await;

                // just checking that the data matches expectations
                assert_eq!(parent_id, parent_record.parent_id);

                Some(ParentModel {
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
                })
            },
            None => None
        }
    });
}


#[tokio::main]
async fn main() {
    use mapper_example::*;
    use rusty_chain::macros::*;

    let mapper = Arc::new(GetParentById::new_raw(
        GetParentByIdInitializer {
            connection_string: String::from("get from settings")
        }
    ).await);
    
    // this thread is queueing up parent models to be received on the other end faster than they can be pulled out
    let receive_mapper = mapper.clone();
    let receive_task = tokio::task::spawn(async {
        let mapper = receive_mapper;
        for index in 0..10 {
            tokio::time::sleep(Duration::from_millis(150)).await;
            println!("receiving {}...", index);
            mapper.push_raw(GetParentByIdInput::new(index)).await;
            println!("received {}.", index);
        }
    });

    // this thread is pulling out parent models on an interval slower than they are being pushed in
    let send_mapper = mapper.clone();
    let send_task = tokio::task::spawn(async {
        let mapper = send_mapper;
        tokio::time::sleep(Duration::from_millis(175)).await;
        for _ in 0..10 {
            println!("processing...");
            mapper.process().await;
            println!("processed.");
            println!("popping...");
            let model = mapper.try_pop().await;
            match model {
                Some(model) => {
                    let locked_model = model.read().await;
                    println!("popped {:?}", locked_model);
                },
                None => {
                    panic!("Unexpected None result.");
                }
            }
        }
    });

    let result = tokio::join!(receive_task, send_task);
    result.0.expect("The 0th receive task should join properly.");
    result.1.expect("The 1th receive task should join properly.");

    println!("Successful!");
}