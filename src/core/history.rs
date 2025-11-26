use crate::core::message::Message;
use std::collections::VecDeque;

pub struct ChatHistory {
    messages: VecDeque<Message>,
    max_size: usize,
}

impl ChatHistory {
    pub fn new(max_size: usize) -> Self {
        Self {
            messages: VecDeque::with_capacity(max_size),
            max_size,
        }
    }

    pub fn add_message(&mut self, message: Message) {
        if self.messages.len() == self.max_size {
            self.messages.pop_front();
        }
        self.messages.push_back(message);
    }

    pub fn get_messages(&self) -> &VecDeque<Message> {
        &self.messages
    }

    pub fn clear(&mut self) {
        self.messages.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
}
