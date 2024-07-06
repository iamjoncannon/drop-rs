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
    pub async fn runner_pool(mut drop_runs: Vec<DropRun>) {
        let result_mutex = Arc::new(Mutex::new(RunPoolOutputMap::new()));

        let mut i = 0;

        let mut run_list: Vec<DropRunner> = drop_runs
            .drain(..)
            .map(|drop_run| {

                i += 1;
                
                DropRunner {
                    id: i,
                    drop_run,
                    result_mutex: Arc::clone(&result_mutex),
                    tx: None,
                    rx: None,
                    depends_on: vec![],
                }
            })
            .collect();

        let (tx, _) = tokio::sync::broadcast::channel::<i32>(run_list.capacity());

        // give each run task a broadcast channel and receiver

        run_list.iter_mut().for_each(move |run| {
            run.tx = Some(tx.clone());
            run.rx = Some(tx.clone().subscribe());
        });

        let mut set = JoinSet::new();

        for call_runner in run_list.drain(..) {
            let future_to_run = async move { call_runner.run().await };
            let _abort_handle = set.spawn(future_to_run);
        }

        while let Some(res) = set.join_next().await {
            if res.is_err() {
                log::warn!("join error: {:?}", res.unwrap_err());
            } else {
                let completed_id: i32 = res.unwrap();
                log::trace!("RunPool completed task with id: {completed_id}");
            }
        }
    }
}
