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
    pub depends_on: Vec<String>,
    pub result_mutex: Arc<Mutex<RunPoolOutputMap>>,
    pub tx: Sender<i32>,
    pub rx: Receiver<i32>,
}

impl DropRunner {

    #[log_attributes::log(debug, "{fn}")]
    pub async fn run(&mut self) -> i32 {

        log::trace!("init DropRunner run: {self:?}");

        // let mut mutex = self.result_mutex;

        self.wait_for_dependencies().await;

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
        let send_res = self.tx.send(self.id);

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
        &mut self,
    )  {

        let drop_id = self.drop_run.call_drop_container.drop_id.as_ref().unwrap().drop_id().unwrap();

        if self.depends_on.is_empty() {
            log::trace!("DropRunner task {drop_id} dependencies resolved, starting." );
            return;
        }

        loop {
            match self.rx.recv().await {
                // the message itself is simply
                // to trigger an observation of the result hash
                Ok(_recvd) => {
                    let unlocked_hash_map = self.result_mutex.lock().unwrap();

                    let keys:Vec<String> = unlocked_hash_map.keys().map(|key| key.to_string()).collect();

                    let mut completed = true;

                    for dependency in self.depends_on.iter() {
                        if!keys.contains(&dependency.to_string()) {
                            completed = false;
                        }
                    }

                    if completed {
                        log::trace!(
                            "DropRunner task complete id: {drop_id:?} dependencies: {:?} completed: {:?}",
                            self.depends_on,
                            unlocked_hash_map
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

    }

}
