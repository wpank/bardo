/// Batch-stop controller
/// Tracks how many plans have completed since last pause
#[derive(Debug)]
pub struct BatchController {
    batch_size: Option<usize>,
    completed_since_pause: usize,
}

impl BatchController {
    pub fn new(batch_size: Option<usize>) -> Self {
        Self {
            batch_size,
            completed_since_pause: 0,
        }
    }

    /// Record a plan completion. Returns true if batch is full and should pause.
    pub fn plan_completed(&mut self) -> bool {
        self.completed_since_pause += 1;
        if let Some(size) = self.batch_size {
            self.completed_since_pause >= size
        } else {
            false
        }
    }

    /// Reset the batch counter (called after user continues from pause)
    pub fn reset(&mut self) {
        self.completed_since_pause = 0;
    }

    pub fn batch_size(&self) -> Option<usize> {
        self.batch_size
    }
}
