use core::fmt::Debug;

use vc_os::utils::ListQueue;

use super::CommandObject;

pub struct CommandQueue {
    queue: ListQueue<CommandObject>,
}

impl Debug for CommandQueue {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("CommmandQueue")
            .field("len", &self.queue.len())
            .finish()
    }
}

impl CommandQueue {
    pub(crate) fn new() -> Self {
        Self {
            queue: ListQueue::default(),
        }
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub fn push(&self, command: CommandObject) {
        self.queue.push(command);
    }

    pub fn pop(&self) -> Option<CommandObject> {
        self.queue.pop()
    }

    pub fn extend(&self, iter: impl IntoIterator<Item = CommandObject>) {
        let iter = iter.into_iter();
        let mut guard = self.queue.lock_push();
        iter.for_each(|command| {
            self.queue.push_with_lock(&mut guard, command);
        });
    }
}
