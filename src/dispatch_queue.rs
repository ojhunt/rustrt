#![feature(futures, async_await, await_macro)]
use std::thread;
use std::sync::Arc;
use std::sync::mpsc;

pub struct DispatchQueue<T>
where
  T: Send + Sync + Clone,
{
  thread_limit: usize, // usize in case we want 2^64 threads
  global_queue: Vec<T>,
}

impl<T> DispatchQueue<T>
where
  T: Send + Sync + Clone,
{
  pub fn new(thread_limit: usize) -> Self {
    DispatchQueue {
      thread_limit,
      global_queue: vec![],
    }
  }
  pub fn add_task(&mut self, task: &T) {
    self.global_queue.push(task.clone());
  }

  #[allow(dead_code, unused_variables)]
  pub fn consume_tasks<F, R>(&mut self, callback: &F) -> Vec<R>
  where
    R: Send + Clone,
    F: Fn(&T) -> R + Send + Clone,
  {
    let mut thread_tasks = vec![Arc::new(vec![])];
    let local_tasks = {
      let mut temp = vec![];
      temp.append(&mut self.global_queue);
      temp
    };

    let max_tasks_per_thread = (self.thread_limit - 1 + local_tasks.len()) / self.thread_limit;
    for task in local_tasks {
      {
        let last_length = {
          let last = thread_tasks.last_mut().unwrap();
          Arc::get_mut(last).unwrap().push(task);
          last.len()
        };
        if last_length == max_tasks_per_thread {
          thread_tasks.push(Arc::new(vec![]));
        }
      }
    }
    let mut threads = vec![];
    for i in 0..self.thread_limit {
      let thread_local_tasks = thread_tasks[i].clone();
      let callback = callback.clone();
      let (tx, rx) = mpsc::channel();
      threads.push(async {});
      thread::spawn(move || {
        tx.send(thread_local_tasks);
      })
      .join();
    }

    return vec![];
  }
}
