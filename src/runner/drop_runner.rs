use std::sync::{Arc, Mutex};

use indexmap::IndexMap;
use rand::Rng;
use tokio::sync::broadcast::{self, error::SendError, Receiver, Sender};

use crate::{action::PostAction, caller::Caller};

use super::{drop_run::DropRun, RunPoolMutex, RunPoolOutputMap};

/// DropRunner manages the exchange
/// between the DropRun
/// and the thread pool
#[derive(Debug)]
pub struct DropRunner {
    pub id: i32,
    pub drop_run: DropRun,
    pub depends_on: Vec<i32>,
    pub result_mutex: Arc<Mutex<RunPoolOutputMap>>,
    pub tx: Option<Sender<i32>>,
    pub rx: Option<Receiver<i32>>,
}

impl DropRunner {

    #[log_attributes::log(debug, "{fn}")]
    pub async fn run(mut self) -> i32 {

        log::trace!("init DropRunner run: {self:?}");

        let mut mutex = self.result_mutex;

        mutex =
            DropRunner::wait_for_dependencies(mutex, self.rx.unwrap(), self.depends_on, self.id)
                .await;

        // resolve run time call dependencies from previous chain

        let inputs_from_dependencies = IndexMap::<String, hcl::Value>::new();

        // evaluate 

        let drop_call = self.drop_run.get_drop_call(inputs_from_dependencies);
        
        log::trace!("DropRunner drop_call: {drop_call:?}");

        let caller = Caller {drop_call};

        let call_record_res = caller.call();

        log::debug!("call_record {call_record_res:?}");

        if call_record_res.is_err() {
            // report error to pool manager to 
            // cancel dependency calls
        } else {

            let call_record = call_record_res.unwrap();

            if !call_record.is_successful_call {
                // report failure to pool manager
            }

            PostAction::run_post_action_callbacks(call_record);
        }


        // report output to global output hash
        // mutex.lock().unwrap().push(self.id);

        // broadcast the id of the completed task
        // to trigger observation in remaining tasks
        let send_res = self.tx.unwrap().send(self.id);

        if send_res.is_err() {
            match send_res.unwrap_err() {
                SendError(e) => {
                    log::trace!(
                        "DropRunner::run task id {:?} send message response error: {:?}",
                        self.id,
                        e
                    );
                }
            }
        }

        self.id
    }

    #[log_attributes::log(debug, "{fn}")]
    async fn wait_for_dependencies(
        mutex: RunPoolMutex,
        mut rx: Receiver<i32>,
        depends_on: Vec<i32>,
        id: i32,
    ) -> RunPoolMutex {

        if depends_on.is_empty() {
            log::trace!("DropRunner task {id:?} dependencies resolved, starting.");
            return mutex;
        }

        loop {
            match rx.recv().await {
                // the message itself is simply
                // to trigger an observation of the result hash
                Ok(_recvd) => {
                    let unlocked = mutex.lock().unwrap();

                    let mut completed = true;

                    for dependency in depends_on.iter() {
                        
                        // logic to resolve dependencies
                        // from previous chain run 

                    }

                    if completed {
                        log::trace!(
                            "DropRunner task complete id: {id:?} dependencies: {:?} completed: {:?}",
                            depends_on,
                            unlocked
                        );

                        break;
                    }
                }
                Err(err) => match err {
                    broadcast::error::RecvError::Closed => break,
                    broadcast::error::RecvError::Lagged(err) => println!("err {err:?}"),
                },
            }
        }

        mutex
    }

}
