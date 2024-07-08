use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::s;
use hcl::Value;
use tokio::task::JoinSet;

use super::{drop_run::DropRun, drop_runner::DropRunner, RunPoolOutputMap};

/// the pool that runs the entire
/// set of DropRuns for the procedure
pub struct RunPool {}

impl RunPool {
    #[log_attributes::log(debug, "{fn}")]
    pub async fn runner_pool(mut drop_runs: Vec<DropRun>) {

        log::trace!("RunPool init {drop_runs:?}");

        let result_mutex = Arc::new(Mutex::new(RunPoolOutputMap::new()));

        let mut i = 0;

        let (tx, _) = tokio::sync::broadcast::channel::<i32>(drop_runs.capacity());

        let mut run_list: Vec<DropRunner> = drop_runs
            .drain(..)
            .map(|mut drop_run| {

                i += 1;

                let depends_on = if drop_run.depends_on.is_none() {
                    drop_run.depends_on.take().unwrap()
                } else {
                    vec![]
                };
                
                DropRunner {
                    id: i,
                    drop_run,
                    result_mutex: Arc::clone(&result_mutex),
                    tx: tx.clone(),
                    rx: tx.clone().subscribe(),
                    depends_on,
                }
            })
            .collect();
   
        log::trace!("RunPool run_list {run_list:?}");

        let mut set = JoinSet::new();

        for mut call_runner in run_list.drain(..) {
            let future_to_run = async move { call_runner.run().await };
            let _abort_handle = set.spawn(future_to_run);
        }

        while let Some(res) = set.join_next().await {
            if res.is_err() {
                log::trace!("join error: {:?}", res.unwrap_err());
            } else {
                let completed_id: i32 = res.unwrap();
                log::trace!("RunPool completed task with id: {completed_id}");
            }
        }
    }
}
