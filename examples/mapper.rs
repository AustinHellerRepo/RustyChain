use rusty_chain::chain_link;

struct ChildRecord {
    parent_id: i32,
    image_bytes: Vec<u8>
}

struct DatabaseConnection {}

impl DatabaseConnection {
    pub fn get_child_records_by_parent_id(parent_id: i32) -> Vec<ChildRecord> {
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

struct ParentRecord {
    parent_id: i32,
    image_bytes: Vec<u8>
}

struct ParentModel {
    parent_id: i32,
    image_bytes: Vec<u8>,
    children_image_bytes: Vec<Vec<u8>>
}

fn get_database_connection() -> DatabaseConnection {
    DatabaseConnection { }
}

chain_link!(DatabaseRecordToDataModelMapper, parent_record: ParentRecord => ParentModel, {
    let database_connection = get_database_connection();
    todo!();
});

fn main() {
}