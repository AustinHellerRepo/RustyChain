use std::sync::{Mutex, Arc};

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

    let mut mapper = GetParentById::new(GetParentByIdInitializer { connection_string: String::from("get from settings") });
    
    // one thread is queueing up parent models to be received on the other end
    mapper.receive(Arc::new(Mutex::new(GetParentByIdInput::new(1)))).await;
    
    // background thread is polling mapper and pulling out parent models on an interval
    mapper.poll().await;
    let model = mapper.send().await;
    
    match model {
        Some(model) => {
            let locked_model = model.lock().unwrap();
            assert_eq!(1, locked_model.parent_id);
            assert_eq!("Some name", locked_model.name.as_str());
            assert_eq!(2, locked_model.children_image_bytes.len());
        },
        None => {
            panic!("Unexpected None result.");
        }
    }

    println!("Successful!");
}