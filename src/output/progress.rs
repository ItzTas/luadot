use std::io::stderr;
use std::sync::Arc;

use prodash::render::line::{JoinHandle, Options, StreamKind};
use prodash::tree::{Item, Root};

use super::constants::{PROGRESS_DELAY, PROGRESS_FRAME_RATE};

pub struct Progress {
    root: Arc<Root>,
    render: Option<JoinHandle>,
}

impl Progress {
    pub fn new() -> Self {
        let root = Root::new();
        let options = Options {
            initial_delay: Some(PROGRESS_DELAY),
            frames_per_second: PROGRESS_FRAME_RATE,
            throughput: true,
            ..Options::default()
        }
        .auto_configure(StreamKind::Stderr);

        let render = prodash::render::line(stderr(), Arc::downgrade(&root), options);

        Self {
            root,
            render: Some(render),
        }
    }

    pub fn task(&self, name: impl Into<String>) -> Item {
        self.root.add_child(name)
    }
}

impl Default for Progress {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Progress {
    fn drop(&mut self) {
        let Some(render) = self.render.take() else {
            return;
        };

        render.shutdown_and_wait();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_task_reports_to_the_tree() {
        let progress = Progress::new();

        let task = progress.task("fetch");
        task.init(Some(2), None);
        task.set(1);

        assert_eq!(progress.root.num_tasks(), 1);
    }
}
