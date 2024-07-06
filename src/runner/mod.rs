use std::sync::{Arc, Mutex};

use crate::s;
use drop_runner::DropRunner;
use tokio::task::JoinSet;

pub mod drop_runner;

pub struct RunPool {}

impl RunPool {
    pub async fn runner_pool() {
        let result_mutex = Arc::new(Mutex::new(Vec::new()));

        let mut run_list: Vec<DropRunner> = Vec::<DropRunner>::new();

        run_list.push(DropRunner {
            message: s!("uno"),
            id: 1,
            depends_on: vec![],
            result_mutex: Arc::clone(&result_mutex),
            tx: None,
            rx: None,
        });

        // run_list.push(DropRunner {
        //     message: s!("dose"),
        //     id: 2,
        //     depends_on: vec![1],
        //     result_mutex: Arc::clone(&result_mutex),
        //     tx: None,
        //     rx: None,
        // });

        // run_list.push(DropRunner {
        //     message: s!("tres"),
        //     id: 3,
        //     depends_on: vec![1, 2],
        //     result_mutex: Arc::clone(&result_mutex),
        //     tx: None,
        //     rx: None,
        // });

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
