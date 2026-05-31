use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::{fs, thread};

pub(crate) struct FilePoller {
    filepath: String,
    poll_interval: Duration,
    current_file_content: Arc<Mutex<String>>,
    last_update: Arc<Mutex<Option<Instant>>>,
}

impl FilePoller {
    pub fn new(filepath: String, poll_interval: Duration) -> Self {
        Self {
            filepath,
            poll_interval,
            current_file_content: Arc::new(Mutex::new(String::new())),
            last_update: Arc::new(Mutex::new(None)),
        }
    }

    pub fn start(&mut self, is_running: Arc<AtomicBool>) {
        let poll_interval = self.poll_interval.clone();
        let filepath = self.filepath.clone();
        let current_file_content = self.current_file_content.clone();
        let last_update = self.last_update.clone();

        // TODO: deal with thread handler
        let _ = thread::spawn(move || {
            while is_running.load(Ordering::Relaxed) {
                // TODO: error handling
                match current_file_content.lock() {
                    Ok(mut content) => {
                        *content = fs::read_to_string(&filepath).unwrap();
                        match last_update.lock() {
                            Ok(mut last_update) => {
                                *last_update = Some(Instant::now());
                            }
                            Err(_) => {}
                        }
                    }
                    Err(e) => {
                        println!("File Poller Error: {:?}", e);
                    }
                }
                thread::sleep(poll_interval);
            }
        });
    }

    pub fn get_current_file_content(&self) -> String {
        // TODO: error handling
        self.current_file_content.lock().unwrap().clone()
    }
}
