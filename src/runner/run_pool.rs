use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::s;
use hcl::Value;
use tokio::{sync::broadcast::{self, Receiver}, task::JoinSet};

use super::{drop_run::DropRun, drop_runner::DropRunner, RunPoolOutputMap};

// https://stackoverflow.com/questions/53458755/how-do-i-gracefully-shutdown-the-tokio-runtime-in-response-to-a-sigterm

/// the pool that runs the entire
/// set of DropRuns for the procedure
pub struct RunPool {}

impl RunPool {
    // manages dag with jobs
    #[log_attributes::log(debug, "{fn}")]
    pub async fn run(mut drop_runs: Vec<DropRun>) {
        // run dag and generate adjacency list from
        // variable dependencies

        // handle dependency cycle

        // get initial nodes

        // create global broadcast channel

        // spawn dispatcher thread that listens to rx
        // dispatcher spawns job threads
        // and waits until either the entire list has
        // been processed or nodes have been released
        // due to dependencies failing
        // dispatcher thread takes results and passes
        // to DropRunner

        // transmit message to start inital nodes

        // join dispatcher thread


        

    }

     pub async fn dispatcher(mut this_rx: Receiver<String>) {
        loop {
            match this_rx.recv().await {
                Ok(_msg) => {

                    

                }
                Err(err) => match err {
                    broadcast::error::RecvError::Closed => {
                        // query adjacency list and see if completed
                    },
                    broadcast::error::RecvError::Lagged(err) => println!("err {err:?}"),
                },
            }
        }
    }

    // run jobs
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

                let depends_on = if drop_run.depends_on.is_some() {
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
