use sp_std::vec::{Vec};
use codec::{Encode, Decode};
use scale_info::TypeInfo;

#[derive(Encode, Decode, Default, Clone, PartialEq, TypeInfo)]
pub struct Queue<T> {
  max_size: u32,
	queue: Vec<T>,
}

impl<T: PartialEq> Queue<T> {
  
  pub fn new(size: u32) -> Self {
    Queue { 
      max_size: size,
      queue: Vec::new() 
    }
  }
  
  pub fn enqueue(&mut self, item: T) -> bool {

    if self.queue.len() < self.max_size as usize {
      self.queue.push(item);
      return true;
    }
    return false;
  }

  pub fn dequeue(&mut self) -> T {
      self.queue.remove(0)
  }

  pub fn length(&self) -> u32 {
    self.queue.len() as u32
  }

  pub fn is_empty(&self) -> bool {
    self.queue.is_empty()
  }

  pub fn peek(&self) -> Option<&T> {
    self.queue.first()
  }

  pub fn contains(&self, item: T) -> bool {
    self.queue.contains(&item)
  }

  pub fn remove(&mut self, item: T) {
    self.queue.retain(|x| x != &item)
  }

}