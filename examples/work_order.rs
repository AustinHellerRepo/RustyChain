
mod work_order {

    pub mod model {
        use dashmap::DashMap;


        pub enum WorkType {
            InvestigateAccount,
            CallCustomer
        }

        pub struct Customer {
            id: u64,
            name: String,
            phone_number: String,
            account_number: String
        }

        pub struct Worker {
            id: u64,
            name: String
        }

        pub struct Order {
            work_type: WorkType,
            customer: Customer
        }

        pub struct WorkerAvailability {
            work_type: WorkType,
            worker: Worker
        }

        pub enum OrderEvent {
            AddOrder(Order),
            AddWorkerAvailability(WorkerAvailability)
        }

        pub enum EventOutcome {
            OrderAssignedToWorker(Order, Worker),
            NothingAssigned
        }

        pub struct WorkSystem {
            customer_per_work_type: DashMap<WorkType, Customer>,
            worker_per_work_type: DashMap<WorkType, Worker>
        }
    }


}

#[tokio::main]
async fn main() {

}