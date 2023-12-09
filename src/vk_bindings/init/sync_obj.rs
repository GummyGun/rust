use ash::{
    vk,
    prelude::VkResult,
};

use super::{
    DeviceDrop,
    Device,
};

use crate::{
    State,
    constants,
};


pub struct SyncObjects {
    pub image_available_semaphore: [vk::Semaphore; constants::FIF],
    pub render_finished_semaphore: [vk::Semaphore; constants::FIF],
    pub in_flight_fence: [vk::Fence; constants::FIF],
}

impl SyncObjects {
    pub fn create(state:&State, device:&Device) -> VkResult<Self> {
        if  state.v_exp() {
            println!("\nCREATING:\tSYNC OBJECTS");
        }
        
        let semaphore_create_info = vk::SemaphoreCreateInfo::builder();
        let fence_create_info = vk::FenceCreateInfo::builder()
            .flags(vk::FenceCreateFlags::SIGNALED);
        
        let mut image_available_semaphore = [vk::Semaphore::null(); constants::FIF];
        let mut render_finished_semaphore = [vk::Semaphore::null(); constants::FIF];
        let mut in_flight_fence = [vk::Fence::null(); constants::FIF];
            
        for index in 0..constants::FIF {
            image_available_semaphore[index] = unsafe{device.create_semaphore(&semaphore_create_info, None)}?;
            render_finished_semaphore[index] = unsafe{device.create_semaphore(&semaphore_create_info, None)}?;
            in_flight_fence[index] =  unsafe{device.create_fence(&fence_create_info, None)}?;
        }
        
        Ok(Self{
            image_available_semaphore: image_available_semaphore,
            render_finished_semaphore: render_finished_semaphore,
            in_flight_fence: in_flight_fence,
        })
    }
}

impl DeviceDrop for SyncObjects {
    fn device_drop(&mut self, state:&State, device:&Device) {
        if state.v_nor() {
            println!("[0]:deleting semaphores");
            println!("[0]:deleting fence");
        }
        
        for index in 0..constants::FIF {
            unsafe{device.destroy_semaphore(self.image_available_semaphore[index], None)};
            unsafe{device.destroy_semaphore(self.render_finished_semaphore[index], None)};
            unsafe{device.destroy_fence(self.in_flight_fence[index], None)};
        }
        
    }
}