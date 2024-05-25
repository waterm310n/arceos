use alloc::{collections::VecDeque, sync::Arc};
use core::{ops::Deref, sync::atomic::AtomicUsize};
use core::sync::atomic::Ordering;

use linked_list::{Adapter, Links, List};

use crate::BaseScheduler;

/// A task wrapper for the [`FifoScheduler`].
///
/// It add extra states to use in [`linked_list::List`].
pub struct FifoTask<T> {
    inner: T,
    resched_cnt:AtomicUsize,//被重新调度计数器，如果为奇数则抢占式调度，否则协作式调度
    links: Links<Self>,
}

unsafe impl<T> Adapter for FifoTask<T> {
    type EntryType = Self;

    #[inline]
    fn to_links(t: &Self) -> &Links<Self> {
        &t.links
    }
}

impl<T> FifoTask<T> {
    /// Creates a new [`FifoTask`] from the inner task struct.
    pub const fn new(inner: T) -> Self {
        Self {
            inner,
            resched_cnt:AtomicUsize::new(0),
            links: Links::new(),
        }
    }

    fn resched_cnt(&self) -> usize {
        self.resched_cnt.load(Ordering::Acquire)
    }

    /// Returns a reference to the inner task struct.
    pub const fn inner(&self) -> &T {
        &self.inner
    }
}

impl<T> Deref for FifoTask<T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// A simple FIFO (First-In-First-Out) cooperative scheduler.
///
/// When a task is added to the scheduler, it's placed at the end of the ready
/// queue. When picking the next task to run, the head of the ready queue is
/// taken.
///
/// As it's a cooperative scheduler, it does nothing when the timer tick occurs.
///
/// It internally uses a linked list as the ready queue.
pub struct FifoScheduler<T> {
    ready_queue: VecDeque<Arc<FifoTask<T>>>,
}

impl<T> FifoScheduler<T> {
    /// Creates a new empty [`FifoScheduler`].
    pub const fn new() -> Self {
        Self {
            ready_queue: VecDeque::new(),
        }
    }
    /// get the name of scheduler
    pub fn scheduler_name() -> &'static str {
        "FIFO"
    }
}

impl<T> BaseScheduler for FifoScheduler<T> {
    type SchedItem = Arc<FifoTask<T>>;

    fn init(&mut self) {}

    fn add_task(&mut self, task: Self::SchedItem) {
        self.ready_queue.push_back(task);
    }

    fn remove_task(&mut self, task: &Self::SchedItem) -> Option<Self::SchedItem> {
        self.ready_queue
        .iter()
        .position(|t| Arc::ptr_eq(t, task))
        .and_then(|idx| self.ready_queue.remove(idx))
        // unsafe { self.ready_queue.remove(task) }
    }

    fn pick_next_task(&mut self) -> Option<Self::SchedItem> {
        self.ready_queue.pop_front()
    }

    fn put_prev_task(&mut self, prev: Self::SchedItem, preempt: bool) {
        // 因为始终坚持抢占式调度，因此通过prev.resched_cnt的奇偶性判断是否插队
        if prev.resched_cnt() % 2 == 0 && preempt {
            self.ready_queue.push_front(prev);
        }else{
            self.ready_queue.push_back(prev);
        }
        
    }

    fn task_tick(&mut self, current: &Self::SchedItem) -> bool {
        // 这个函数是在说因为时钟中断所以需要再次调度吗？如果是的话，那么该函数始终返回true。
        current.resched_cnt.fetch_add(1, Ordering::Release);
        true
        // if current.resched_cnt() % 2 == 0 {
        //     false
        // }else{
        //     true
        // }
    }

    fn set_priority(&mut self, _task: &Self::SchedItem, _prio: isize) -> bool {
        false
    }
}
