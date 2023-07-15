use std::sync::{Mutex, Arc};

mod mapper_example {

    use rusty_chain::chain_link;

    struct ChildRecord {
        parent_id: i32,
        image_bytes: Vec<u8>
    }

    struct DatabaseConnection {}

    impl DatabaseConnection {
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
        image_bytes: Vec<u8>
    }

    pub struct ParentModel {
        parent_id: i32,
        children_image_bytes: Vec<Vec<u8>>
    }

    fn get_database_connection() -> DatabaseConnection {
        DatabaseConnection { }
    }

    chain_link!(GetParentById, parent_id: i32 => ParentModel, {
        let database_connection = get_database_connection();
        let child_records = database_connection.get_child_records_by_parent_id(*parent_id);
        ParentModel {
            parent_id: *parent_id,
            children_image_bytes: child_records
                .into_iter()
                .map(|cr| {
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
    let mut mapper = GetParentById::new(GetParentByIdInitializer { });
    mapper.receive(Arc::new(Mutex::new(1))).await;
}