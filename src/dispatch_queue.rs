use std::thread;
use std::sync::Arc;
use std::sync::mpsc;
use rand::thread_rng;
use rand::seq::SliceRandom;

pub struct DispatchQueue<T>
where
  T: Send + Sync + Clone + 'static,
{
  thread_limit: usize, // usize in case we want 2^64 threads
  global_queue: Vec<(usize, T)>,
  current_task: usize,
}

fn shuffle<T>(input: &mut Vec<T>) {
  let mut rng = thread_rng();
  input.shuffle(&mut rng);
}

impl<T> Default for DispatchQueue<T>
where
  T: Send + Sync + Clone + 'static,
{
  fn default() -> Self {
    return DispatchQueue::new(num_cpus::get());
  }
}

impl<T> DispatchQueue<T>
where
  T: Send + Sync + Clone + 'static,
{
  pub fn new(thread_limit: usize) -> Self {
    DispatchQueue {
      thread_limit: thread_limit,
      global_queue: vec![],
      current_task: 0,
    }
  }
  pub fn add_task(&mut self, task: &T) {
    self.global_queue.push((self.current_task, task.clone()));
  }

  #[allow(dead_code, unused_variables)]
  pub fn consume_tasks<F, R>(&mut self, callback: &F) -> Vec<R>
  where
    R: Send + Sync + Clone + 'static,
    F: Fn(&T) -> R + Send + Clone + 'static,
  {
    let mut local_tasks = {
      let mut temp = vec![];
      temp.append(&mut self.global_queue);
      shuffle(&mut temp);
      temp
    };

    let mut thread_tasks = vec![vec![]];
    let max_tasks_per_thread = (self.thread_limit - 1 + local_tasks.len()) / self.thread_limit;
    while let Some(task) = local_tasks.pop() {
      let last_length = {
        let last = thread_tasks.last_mut().unwrap();
        last.push(task);
        last.len()
      };
      if last_length == max_tasks_per_thread {
        thread_tasks.push(vec![]);
      }
    }
    assert!(local_tasks.len() == 0);

    if thread_tasks.last().unwrap().is_empty() {
      thread_tasks.pop();
    }

    assert!(thread_tasks.len() <= self.thread_limit);

    let mut threads = vec![];
    let mut channels = vec![];
    for i in 0..self.thread_limit.min(thread_tasks.len()) {
      let thread_local_tasks = Arc::new(thread_tasks.pop().unwrap());
      let callback = callback.clone();
      let (tx, rx) = mpsc::channel();
      channels.push(rx);
      threads.push(thread::spawn(move || {
        let mut result = vec![];
        for task in thread_local_tasks.iter() {
          result.push((task.0, callback(&task.1)));
        }

        tx.send(Arc::new(result))
      }));
    }
    let mut results = vec![];
    for channel in channels {
      for result in channel.recv().unwrap().iter() {
        results.push(result.clone());
      }
    }
    results.sort_by(|(a, _), (b, _)| a.cmp(b));
    assert!(results[0].0 == 0);
    return results.iter().map(|(_, r)| r.clone()).collect();
  }
}
