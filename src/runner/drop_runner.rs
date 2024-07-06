use std::sync::{Arc, Mutex};

use rand::Rng;
use tokio::sync::broadcast::{self, error::SendError, Receiver, Sender};

/// DropRunner manages the exchange
/// between the underlying call
/// and the thread pool
///
/// DropRun manages evaluating the
/// final call block before execution
#[derive(Debug)]
pub struct DropRunner {
    pub message: String,
    pub id: i32,
    pub depends_on: Vec<i32>,
    pub result_mutex: Arc<Mutex<Vec<i32>>>,
    pub tx: Option<Sender<i32>>,
    pub rx: Option<Receiver<i32>>,
}

impl DropRunner {
    async fn wait_for_dependencies(
        mutex: Arc<Mutex<Vec<i32>>>,
        mut rx: Receiver<i32>,
        depends_on: Vec<i32>,
        id: i32,
    ) -> Arc<Mutex<Vec<i32>>> {
        if depends_on.is_empty() {
            log::trace!("DropRunner task complete id: {id:?} ");
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
                        if !unlocked.contains(dependency) {
                            completed = false;
                        }
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

    pub async fn run(self) -> i32 {
        let mut mutex = self.result_mutex;

        mutex =
            DropRunner::wait_for_dependencies(mutex, self.rx.unwrap(), self.depends_on, self.id)
                .await;

        DropRunner::api_call_placeholder();

        // report output to global output hash
        mutex.lock().unwrap().push(self.id);

        // broadcast the id of the completed task
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

    fn api_call_placeholder() {
        let mut rng = rand::thread_rng();

        let millis = rng.gen_range(1..2000);

        std::thread::sleep(std::time::Duration::from_millis(millis));
    }
}
