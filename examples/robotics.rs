use std::time::Duration;
use robotics::{automated_robot::{AutomatedRobot, AutomatedRobotInitializer}, sensory_split::SensorySplitInitializer, camera_sensor::CameraSensorInitializer, sensor_processor::SensorProcessorInitializer, robot_interface::RobotInterfaceInitializer, dependency::{Robot, Controller, Camera}, controller_sensor::ControllerSensorInitializer};
use rusty_chain::chain::ChainLink;

mod robotics {

    pub mod model {
        use rand::Rng;
        
        pub enum SensorData {
            Camera(Direction),
            Controller(KeyPress)
        }

        pub enum Direction {
            Left,
            Straight,
            Right
        }

        impl Direction {
            pub fn choose<R: Rng + ?Sized>(rng: &mut R) -> Self {
                match rng.gen_range(0..=2) {
                    0 => Direction::Left,
                    1 => Direction::Straight,
                    _ => Direction::Right
                }
            }
        }

        pub enum KeyPress {
            Stop,
            Go
        }

        pub enum RobotAction {
            Shutdown,
            Startup,
            MoveLeft,
            MoveStraight,
            MoveRight
        }

        pub enum Facing {
            North,
            South,
            East,
            West
        }
    }

    pub mod dependency {
        use std::{time::Duration, sync::Arc};
        use tokio::sync::Mutex;
        use super::model::{Direction, KeyPress, Facing};
        
        // This struct represents the physical sensor of the robot
        pub struct Camera { }

        impl Camera {
            pub fn new() -> Self {
                Camera { }
            }
            pub async fn read_instruction_under_robot(&self) -> Direction {

                // pretend that reading from the camera is somewhat slow
                tokio::time::sleep(Duration::from_millis(1000)).await;

                // actual camera read would occur here
                Direction::choose(&mut rand::thread_rng())
            }
        }

        // This struct represents a physical control pad for fast interrupts
        pub struct Controller {
            read_attempts: Arc<Mutex<u32>>,
            last_key_press: KeyPress
        }

        impl Controller {
            pub fn new() -> Self {
                Controller {
                    read_attempts: Arc::new(Mutex::new(0)),
                    last_key_press: KeyPress::Go
                }
            }
            pub async fn read_last_keypress(&mut self) -> Option<KeyPress> {
                
                // toggling stop and go every X reads
                let mut locked_read_attempts = self.read_attempts.lock().await;
                let read_attempts: u32 = *locked_read_attempts;
                if read_attempts == 8 {
                    *locked_read_attempts = 0;
                    match self.last_key_press {
                        KeyPress::Go => {
                            self.last_key_press = KeyPress::Stop;
                            Some(KeyPress::Stop)
                        },
                        KeyPress::Stop => {
                            self.last_key_press = KeyPress::Go;
                            Some(KeyPress::Go)
                        }
                    }
                }
                else {
                    *locked_read_attempts += 1;
                    None
                }
            }
        }

        pub struct Robot {
            is_active: bool,
            location: (i8, i8),
            facing: Facing
        }

        impl Robot {
            pub fn new() -> Self {
                Robot {
                    is_active: true,
                    location: (0, 0),
                    facing: Facing::North
                }
            }
            pub fn shutdown(&mut self) {
                println!("{}: Robot: shutting down...", chrono::Utc::now().timestamp());
                self.is_active = false;
            }
            pub fn startup(&mut self) {
                println!("{}: Robot: starting up...", chrono::Utc::now().timestamp());
                self.is_active = true;
            }
            pub fn move_left(&mut self) {
                if self.is_active {
                    println!("{}: Robot: moving left...", chrono::Utc::now().timestamp());
                    // turn left and move straight
                    match self.facing {
                        Facing::North => {
                            self.facing = Facing::West;
                            self.location = (self.location.0 - 1, self.location.1);
                        },
                        Facing::South => {
                            self.facing = Facing::East;
                            self.location = (self.location.0 + 1, self.location.1);
                        },
                        Facing::East => {
                            self.facing = Facing::North;
                            self.location = (self.location.0, self.location.1 + 1);
                        },
                        Facing::West => {
                            self.facing = Facing::South;
                            self.location = (self.location.0, self.location.1 - 1);
                        }
                    }
                }
            }
            pub fn move_right(&mut self) {
                if self.is_active {
                    println!("{}: Robot: moving right...", chrono::Utc::now().timestamp());
                    // turn right and move forward
                    match self.facing {
                        Facing::North => {
                            self.facing = Facing::East;
                            self.location = (self.location.0 + 1, self.location.1);
                        },
                        Facing::South => {
                            self.facing = Facing::West;
                            self.location = (self.location.0 - 1, self.location.1);
                        },
                        Facing::East => {
                            self.facing = Facing::South;
                            self.location = (self.location.0, self.location.1 - 1);
                        },
                        Facing::West => {
                            self.facing = Facing::North;
                            self.location = (self.location.0, self.location.1 + 1);
                        }
                    }
                }
            }
            pub fn move_straight(&mut self) {
                if self.is_active {
                    println!("{}: Robot: moving straight...", chrono::Utc::now().timestamp());
                    // move forward
                    match self.facing {
                        Facing::North => {
                            self.location = (self.location.0, self.location.1 + 1);
                        },
                        Facing::South => {
                            self.location = (self.location.0, self.location.1 - 1);
                        },
                        Facing::East => {
                            self.location = (self.location.0 + 1, self.location.1);
                        },
                        Facing::West => {
                            self.location = (self.location.0 - 1, self.location.1);
                        }
                    }
                }
            }
        }
    }

    pub mod camera_sensor {
        use rusty_chain::chain_link;
        use crate::robotics::model::SensorData;
        use super::dependency::Camera;

        chain_link!(CameraSensor => (camera: Camera), input: () => SensorData, {
            match input.received {
                Some(_) => {
                    println!("{}: CameraSensor", chrono::Utc::now().timestamp());
                    let direction = input.initializer.read().await.camera.read_instruction_under_robot().await;
                    Some(SensorData::Camera(direction))
                },
                None => None
            }
        });
    }

    pub mod controller_sensor {
        use rusty_chain::chain_link;
        use super::{model::SensorData, dependency::Controller};

        chain_link!(ControllerSensor => (controller: Controller), input: () => SensorData, {
            match input.received {
                Some(_) => {
                    println!("{}: ControllerSensor", chrono::Utc::now().timestamp());
                    if let Some(key_press) = input.initializer.write().await.controller.read_last_keypress().await {
                        Some(SensorData::Controller(key_press))
                    }
                    else {
                        None
                    }
                },
                None => None
            }
        });
    }

    pub mod sensor_processor {
        use rusty_chain::chain_link;
        use crate::robotics::model::RobotAction;
        use super::model::{SensorData, Direction, KeyPress};

        chain_link!(SensorProcessor, input: SensorData => RobotAction, {
            match input.received {
                Some(sensor_data) => {
                    println!("{}: SensorProcessor", chrono::Utc::now().timestamp());
                    match &*sensor_data.read().await {
                        SensorData::Camera(direction) => {
                            match direction {
                                Direction::Left => {
                                    Some(RobotAction::MoveLeft)
                                },
                                Direction::Straight => {
                                    Some(RobotAction::MoveStraight)
                                },
                                Direction::Right => {
                                    Some(RobotAction::MoveRight)
                                }
                            }
                        },
                        SensorData::Controller(key_press) => {
                            match key_press {
                                KeyPress::Go => {
                                    Some(RobotAction::Startup)
                                },
                                KeyPress::Stop => {
                                    Some(RobotAction::Shutdown)
                                }
                            }
                        }
                    }
                },
                None => None
            }
        });
    }

    pub mod robot_interface {
        use rusty_chain::chain_link;
        use super::{dependency::Robot, model::RobotAction};

        chain_link!(RobotInterface => (robot: Robot), input: RobotAction => bool, {
            match input.received {
                Some(robot_action) => {
                    println!("{}: RobotInterface", chrono::Utc::now().timestamp());
                    let robot = &mut input.initializer.write().await.robot;
                    match &*robot_action.read().await {
                        RobotAction::MoveLeft => {
                            robot.move_left();
                        },
                        RobotAction::MoveStraight => {
                            robot.move_straight();
                        },
                        RobotAction::MoveRight => {
                            robot.move_right();
                        },
                        RobotAction::Shutdown => {
                            robot.shutdown();
                        },
                        RobotAction::Startup => {
                            robot.startup();
                        }
                    }
                    Some(true)
                },
                None => None
            }
        });
    }

    pub mod sensory_split {
        use rusty_chain::split_merge;
        use super::{model::SensorData, controller_sensor::{ControllerSensor, ControllerSensorInitializer}, camera_sensor::{CameraSensor, CameraSensorInitializer}};

        // the split_merge is not joined, so this runs each sensor in separate threads
        split_merge!(SensorySplit, () => SensorData, (CameraSensor, ControllerSensor));
    }

    pub mod automated_robot {
        use rusty_chain::chain;
        use super::{sensory_split::{SensorySplit, SensorySplitInitializer}, sensor_processor::{SensorProcessor, SensorProcessorInitializer}, robot_interface::{RobotInterface, RobotInterfaceInitializer}};

        chain!(AutomatedRobot, () => bool, SensorySplit => SensorProcessor => RobotInterface);
    }
}

fn main() {

    // ensure that tokio will use multiple threads, permitting the split_merge async to run the ControllerSensor while waiting for the CameraSensor
    let tokio_runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_time()
        .build()
        .unwrap();

    tokio_runtime.block_on(async {

        let automated_robot = AutomatedRobot::new(AutomatedRobotInitializer {
            x_sensory_split: SensorySplitInitializer {
                x_camera_sensor_initializer: CameraSensorInitializer {
                    camera: Camera::new()
                },
                xx_controller_sensor_initializer: ControllerSensorInitializer {
                    controller: Controller::new()
                }
            },
            xx_sensor_processor: SensorProcessorInitializer { },
            xxx_robot_interface: RobotInterfaceInitializer {
                robot: Robot::new()
            }
        });

        for _ in 0..30 {
            println!("{}: loop start", chrono::Utc::now().timestamp());
            automated_robot.push_raw_if_empty(()).await;
            automated_robot.process().await;
            automated_robot.try_pop().await;
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    });
}