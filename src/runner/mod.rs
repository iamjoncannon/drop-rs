use std::{collections::HashMap, sync::{Arc, Mutex}};

use hcl::Value;

pub mod drop_runner;
pub mod drop_run;
pub mod run_pool;

type RunPoolOutputMap = HashMap<String,Value>;
type RunPoolMutex = Arc<Mutex<RunPoolOutputMap>>;