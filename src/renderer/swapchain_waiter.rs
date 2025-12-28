use crate::renderer::util::Dev;
use crate::renderer::{Synchronization, FRAMES_IN_FLIGHT};
use ash::vk;
use std::sync::mpsc;
use std::thread::JoinHandle;
use winit::event_loop::EventLoopProxy;

pub struct SwapchainWaiter {
    sender: Option<mpsc::SyncSender<vk::SwapchainKHR>>,
    join_handle: Option<JoinHandle<()>>,
}

#[derive(Debug)]
pub struct SwapchainEvent {
    pub image_index: Option<usize>,
}

impl SwapchainWaiter {
    pub(super) fn new(
        sync: Synchronization,
        dev: &Dev,
        event_loop_proxy: EventLoopProxy<SwapchainEvent>,
    ) -> SwapchainWaiter {
        let (sender, receiver) = mpsc::sync_channel(1);
        let dev = dev.clone();
        let join_handle = std::thread::Builder::new()
            .name("swapchain-waiter".to_owned())
            .spawn(move || thread(sync, receiver, dev, event_loop_proxy))
            .unwrap();
        SwapchainWaiter {
            sender: Some(sender),
            join_handle: Some(join_handle),
        }
    }

    pub fn send(&self, swapchain: vk::SwapchainKHR) {
        self.sender.as_ref().unwrap().send(swapchain).unwrap();
    }

    pub fn shutdown(&mut self) {
        let _ = self.sender.take().unwrap();
        self.join_handle.take().unwrap().join().unwrap();
    }
}

fn thread(
    sync: Synchronization,
    receiver: mpsc::Receiver<vk::SwapchainKHR>,
    dev: Dev,
    event_loop: EventLoopProxy<SwapchainEvent>,
) {
    let mut flight_index = 0;
    while let Ok(swapchain) = receiver.recv() {
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

        let image_index = if acquire_result == Err(vk::Result::ERROR_OUT_OF_DATE_KHR) {
            None
        } else {
            Some(acquire_result.unwrap().0 as usize)
        };
        event_loop
            .send_event(SwapchainEvent { image_index })
            .unwrap();

        flight_index = (flight_index + 1) % FRAMES_IN_FLIGHT;
    }
}
