use crate::renderer::util::Dev;
use crate::renderer::{Synchronization, FRAMES_IN_FLIGHT};
use ash::vk;
use std::sync::{Arc, Condvar, Mutex};
use std::thread::JoinHandle;
use winit::event_loop::EventLoopProxy;

pub struct SwapchainWaiter {
    shared: Arc<(Mutex<State>, Condvar)>,
    join_handle: Option<JoinHandle<()>>,
}

#[derive(Debug)]
pub struct SwapchainEvent {
    pub image_index: Option<usize>,
}

#[derive(PartialEq)]
enum State {
    Waiting(vk::SwapchainKHR),
    Idle,
    Shutdown,
}

impl SwapchainWaiter {
    pub(super) fn new(
        swapchain: vk::SwapchainKHR,
        sync: Synchronization,
        dev: &Dev,
        event_loop_proxy: EventLoopProxy<SwapchainEvent>,
    ) -> SwapchainWaiter {
        let shared = Arc::new((Mutex::new(State::Waiting(swapchain)), Condvar::new()));
        let shared2 = shared.clone();
        let dev = dev.clone();
        let join_handle = std::thread::Builder::new()
            .name("swapchain-waiter".to_owned())
            .spawn(move || thread(sync, shared2, dev, event_loop_proxy))
            .unwrap();
        SwapchainWaiter {
            shared,
            join_handle: Some(join_handle),
        }
    }

    pub fn send(&self, swapchain: vk::SwapchainKHR) {
        let (state, condvar) = self.shared.as_ref();
        *state.lock().unwrap() = State::Waiting(swapchain);
        condvar.notify_all();
    }

    pub fn shutdown(&mut self) {
        let (state, condvar) = self.shared.as_ref();
        *state.lock().unwrap() = State::Shutdown;
        condvar.notify_all();
        self.join_handle.take().unwrap().join().unwrap();
    }
}

fn thread(
    sync: Synchronization,
    swapchain: Arc<(Mutex<State>, Condvar)>,
    dev: Dev,
    event_loop: EventLoopProxy<SwapchainEvent>,
) {
    let (swapchain_mutex, swapchain_codvar) = swapchain.as_ref();
    let mut state = swapchain_mutex.lock().unwrap();
    let mut flight_index = 0;
    loop {
        let swapchain = match *state {
            State::Waiting(swapchain) => swapchain,
            State::Idle => unreachable!(),
            State::Shutdown => break,
        };

        let image_available = sync.image_available[flight_index];
        let in_flight = sync.in_flight[flight_index];

        unsafe { dev.wait_for_fences(&[in_flight], true, u64::MAX).unwrap() }
        unsafe { dev.reset_fences(&[in_flight]).unwrap() }

        let acquire_result = unsafe {
            dev.swapchain_ext.acquire_next_image(
                swapchain,
                u64::MAX,
                image_available,
                vk::Fence::null(),
            )
        };

        *state = State::Idle;
        let image_index = if acquire_result == Err(vk::Result::ERROR_OUT_OF_DATE_KHR) {
            None
        } else {
            Some(acquire_result.unwrap().0 as usize)
        };
        event_loop
            .send_event(SwapchainEvent { image_index })
            .unwrap();

        while *state == State::Idle {
            state = swapchain_codvar.wait(state).unwrap();
        }

        flight_index = (flight_index + 1) % FRAMES_IN_FLIGHT;
    }
}
