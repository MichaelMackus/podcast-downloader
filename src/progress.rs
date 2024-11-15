// Define a custom progress reporter:
pub struct SimpleReporterPrivate {
    last_update: std::time::Instant,
    max_progress: Option<u64>,
    message: String,
}
pub struct SimpleReporter {
    private: std::sync::Mutex<Option<SimpleReporterPrivate>>,
}

impl SimpleReporter {
    pub fn create() -> std::sync::Arc<Self> {
        std::sync::Arc::new(Self {
            private: std::sync::Mutex::new(None),
        })
    }
}

impl downloader::progress::Reporter for SimpleReporter {
    fn setup(&self, max_progress: Option<u64>, message: &str) {
        let private = SimpleReporterPrivate {
            last_update: std::time::Instant::now(),
            max_progress,
            message: message.to_owned(),
        };

        let mut guard = self.private.lock().unwrap();
        *guard = Some(private);
    }

    fn progress(&self, current: u64) {
        if let Some(p) = self.private.lock().unwrap().as_mut() {
            let max_bytes = match p.max_progress {
                Some(bytes) => format!("{:?}", bytes),
                None => "{unknown}".to_owned(),
            };
            if p.last_update.elapsed().as_millis() >= 1000 {
                println!(
                    "test file: {} of {} bytes. [{}]",
                    current, max_bytes, p.message
                );
                p.last_update = std::time::Instant::now();
            }
        }
    }

    fn set_message(&self, message: &str) {
        println!("test file: Message changed to: {}", message);
    }

    fn done(&self) {
        let mut guard = self.private.lock().unwrap();
        *guard = None;
        println!("test file: [DONE]");
    }
}
