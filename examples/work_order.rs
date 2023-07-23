use rusty_chain::chain::ChainLink;
use work_order::{work_assignment_manager::WorkAssignmentManagerInitializer, model::{WorkSystemCache, OrderEvent, Order, WorkType, Customer, WorkerAvailability, Worker}, work_processor::{WorkProcessor, WorkProcessorInitializer}, unit_of_work_manager::UnitOfWorkManagerInitializer};

mod work_order {

    // the shared types between other modules
    pub mod model {
        use std::fmt::Display;
        use dashmap::DashMap;

        #[derive(PartialEq, Eq, Hash, Clone, Debug)]
        pub enum WorkType {
            InvestigateAccount,
            CallCustomer
        }

        impl Display for WorkType {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{:?}", self)
            }
        }

        #[derive(Clone)]
        pub struct Customer {
            pub name: String,
        }

        #[derive(Clone)]
        pub struct Worker {
            pub name: String
        }

        #[derive(Clone)]
        pub struct Order {
            pub work_type: WorkType,
            pub customer: Customer
        }

        #[derive(Clone)]
        pub struct WorkerAvailability {
            pub work_type: WorkType,
            pub worker: Worker
        }

        pub enum OrderEvent {
            AddOrder(Order),
            AddWorkerAvailability(WorkerAvailability)
        }

        pub struct AssignedOrder {
            pub work_type: WorkType,
            pub customer: Customer,
            pub worker: Worker
        }

        pub enum EventOutcome {
            OrderAssignedToWorker(AssignedOrder),
            NothingAssigned
        }

        pub struct WorkSystemCache {
            pub customers_per_work_type: DashMap<WorkType, tokio::sync::Mutex<Vec<Customer>>>,
            pub workers_per_work_type: DashMap<WorkType, tokio::sync::Mutex<Vec<Worker>>>
        }

        impl WorkSystemCache {
            pub fn new() -> Self {
                WorkSystemCache {
                    customers_per_work_type: DashMap::new(),
                    workers_per_work_type: DashMap::new()
                }
            }
        }
    }

    pub mod work_assignment_manager {
        use rusty_chain::chain_link;
        use crate::work_order::model::{EventOutcome, AssignedOrder};
        use super::model::{WorkSystemCache, OrderEvent};

        // reacts to an order event, potentially assigning a customer's work to a worker
        chain_link!(WorkAssignmentManager => (work_system_cache: WorkSystemCache), input: OrderEvent => EventOutcome, {
            match input.received {
                Some(order_event) => {
                    match order_event {
                        OrderEvent::AddOrder(order) => {

                            // ensure that the work system contains the work type
                            {
                                let locked_work_system_cache = &input.initializer.write().await.work_system_cache;
                                if !locked_work_system_cache.customers_per_work_type.contains_key(&order.work_type) {
                                    locked_work_system_cache.customers_per_work_type.insert(order.work_type.clone(), tokio::sync::Mutex::new(vec![]));
                                }
                                if !locked_work_system_cache.workers_per_work_type.contains_key(&order.work_type) {
                                    locked_work_system_cache.workers_per_work_type.insert(order.work_type.clone(), tokio::sync::Mutex::new(vec![]));
                                }
                            }

                            // check for an available worker

                            if let Some(popped_worker) = &input.initializer
                                .read()
                                .await
                                .work_system_cache
                                .workers_per_work_type
                                .get(&order.work_type)
                                .expect(&format!("The work type {} should exist in the workers_per_work_type.", order.work_type))
                                .lock()
                                .await
                                .pop() {
                                
                                    // there is a worker available for this customer's work order
                                    Some(EventOutcome::OrderAssignedToWorker(AssignedOrder {
                                        work_type: order.work_type.clone(),
                                        worker: popped_worker.clone(),
                                        customer: order.customer.clone()
                                    }))
                            }
                            else {

                                // cache the customer's work for when a worker is available
                                input.initializer
                                    .read()
                                    .await
                                    .work_system_cache
                                    .customers_per_work_type
                                    .get(&order.work_type)
                                    .unwrap()
                                    .lock()
                                    .await
                                    .push(order.customer.clone());

                                Some(EventOutcome::NothingAssigned)
                            }

                        },
                        OrderEvent::AddWorkerAvailability(worker_availability) => {

                            // ensure that the work system contains the work type
                            {
                                let locked_work_system_cache = &input.initializer.write().await.work_system_cache;
                                if !locked_work_system_cache.customers_per_work_type.contains_key(&worker_availability.work_type) {
                                    locked_work_system_cache.customers_per_work_type.insert(worker_availability.work_type.clone(), tokio::sync::Mutex::new(vec![]));
                                }
                                if !locked_work_system_cache.workers_per_work_type.contains_key(&worker_availability.work_type) {
                                    locked_work_system_cache.workers_per_work_type.insert(worker_availability.work_type.clone(), tokio::sync::Mutex::new(vec![]));
                                }
                            }

                            // check for an applicable order

                            if let Some(popped_customer) = &input.initializer
                                .read()
                                .await
                                .work_system_cache
                                .customers_per_work_type
                                .get(&worker_availability.work_type)
                                .expect(&format!("The work type {} should exist in the workers_per_work_type.", worker_availability.work_type))
                                .lock()
                                .await
                                .pop() {
                                
                                    // there is a customer's work order ready for this worker
                                    Some(EventOutcome::OrderAssignedToWorker(AssignedOrder {
                                        work_type: worker_availability.work_type.clone(),
                                        worker: worker_availability.worker.clone(),
                                        customer: popped_customer.clone()
                                    }))
                            }
                            else {

                                // cache the worker until a customer's work order is processed
                                input.initializer
                                    .read()
                                    .await
                                    .work_system_cache
                                    .workers_per_work_type
                                    .get(&worker_availability.work_type)
                                    .unwrap()
                                    .lock()
                                    .await
                                    .push(worker_availability.worker.clone());

                                Some(EventOutcome::NothingAssigned)
                            }
                        }
                    }
                },
                None => None
            }
        });
    }

    pub mod unit_of_work_manager {
        use rusty_chain::chain_link;
        use super::model::{EventOutcome, WorkType};

        // processes the assigned order, if applicable
        chain_link!(UnitOfWorkManager, input: EventOutcome => bool, {
            match input.received {
                Some(event_outcome) => {
                    match event_outcome {
                        EventOutcome::OrderAssignedToWorker(assigned_order) => {
                            match assigned_order.work_type {
                                WorkType::InvestigateAccount => {
                                    println!("The worker {} investigated the account of {}.", assigned_order.worker.name, assigned_order.customer.name);
                                    Some(true)
                                },
                                WorkType::CallCustomer => {
                                    println!("The worker {} called the customer {}.", assigned_order.worker.name, assigned_order.customer.name);
                                    Some(true)
                                }
                            }
                        },
                        EventOutcome::NothingAssigned => None
                    }
                },
                None => None
            }
        });
    }

    pub mod work_processor {
        use rusty_chain::chain;
        use super::{unit_of_work_manager::{UnitOfWorkManager, UnitOfWorkManagerInitializer}, work_assignment_manager::{WorkAssignmentManager, WorkAssignmentManagerInitializer}, model::OrderEvent};

        // the chain of processing from an order event to a processed work order
        chain!(WorkProcessor, OrderEvent => bool, WorkAssignmentManager => UnitOfWorkManager);
    }
}

#[tokio::main]
async fn main() {
    
    // accept in customer orders and available workers, pairing them up as they become available

    let work_processor = WorkProcessor::new(WorkProcessorInitializer {
        x_work_assignment_manager: WorkAssignmentManagerInitializer {
            work_system_cache: WorkSystemCache::new()
        },
        xx_unit_of_work_manager: UnitOfWorkManagerInitializer { }
    });

    work_processor.push_raw(OrderEvent::AddOrder(Order {
        work_type: WorkType::CallCustomer,
        customer: Customer {
            name: String::from("John")
        }
    })).await;

    // there are no pairs yet
    assert!(!work_processor.process().await);

    work_processor.push_raw(OrderEvent::AddWorkerAvailability(WorkerAvailability {
        work_type: WorkType::InvestigateAccount,
        worker: Worker {
            name: String::from("Bob")
        }
    })).await;

    // there are no pairs yet
    assert!(!work_processor.process().await);

    work_processor.push_raw(OrderEvent::AddWorkerAvailability(WorkerAvailability {
        work_type: WorkType::CallCustomer,
        worker: Worker {
            name: String::from("Bill")
        }
    })).await;

    // a customer needing a call and a worker who can call are now present
    assert!(work_processor.process().await);

    work_processor.push_raw(OrderEvent::AddOrder(Order {
        work_type: WorkType::InvestigateAccount,
        customer: Customer {
            name: String::from("Jane")
        }
    })).await;

    // a customer needing their account investigated and a worker who investigates are now present
    assert!(work_processor.process().await);
}