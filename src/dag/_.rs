
pub fn dag_observer_thread(timeout_mutex: Arc<Mutex<HashMap<i32, u128>>>, tx: Sender<String>) {
    loop {
        // for the items in the deadline hash
        // observe if they've timed out

        let mut timeout_hash_guard = timeout_mutex.lock().unwrap();

        let mut jobs_to_remove = Vec::<i32>::new();

        for (job_id, timeout_deadline) in timeout_hash_guard.iter() {
            if current_time() > *timeout_deadline {
                println!(
                    "{job_id} exceeded deadline: {} {}",
                    current_time(),
                    timeout_deadline
                );
                jobs_to_remove.push(*job_id);
            }
        }

        if !jobs_to_remove.is_empty() {
            for job in jobs_to_remove {
                timeout_hash_guard.remove(&job);
            }

            println!("timeout_hash {timeout_hash_guard:?}");

            if timeout_hash_guard.is_empty() {
                tx.send("fini".to_string());
            }
        }

        drop(timeout_hash_guard);

        // observe if state is completed-
        // no pending jobs

        sleep(50);
    }
}