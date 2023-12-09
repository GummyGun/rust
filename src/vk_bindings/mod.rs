use ash::vk;
mod init;
pub use init::*;

mod objects;
use super::{
    window::{
        Window,
    },
    constants,
    State,
};

use objects::{
    VkObj,
    VkObjDevDep,
    DeviceDrop,
    ActiveDrop,
};

use std::{
    ptr::addr_of,
};

pub struct VInit {
    state: State,
    pub instance: VkObj<Instance>,
    pub messenger: Option<VkObj<DMessenger>>,
    pub surface: VkObj<Surface>,
    pub p_device: PhysicalDevice,
    pub device: VkObj<Device>,
    pub swapchain: VkObjDevDep<Swapchain>,
    pub render_pass: VkObjDevDep<RenderPass>,
    pub pipeline: VkObjDevDep<Pipeline>,
    pub sc_framebuffers: VkObjDevDep<SCFramebuffers>,
    pub command_control: VkObjDevDep<CommandControl>,
    pub sync_objects: VkObjDevDep<SyncObjects>,
}

impl VInit {
    pub fn init(state:State, window:&Window) -> VInit {
        
        let instance = vk_create_interpreter(state, Instance::create(&state, window), "instance"); 
        
        let messenger = if constants::VALIDATION {
            Some(match DMessenger::create(&state, &instance) {
                Ok(messenger) => {
                    if state.v_nor() {
                        println!("[0]:messenger");
                    }
                    messenger
                }
                Err(err) => {panic!("{:?}", err);}
            })
        } else {
            println!("[X]:messenger");
            None
        };
        
        let surface =  vk_create_interpreter(state, Surface::create(&state, &window, &instance), "surface"); 
        let p_device = vk_create_interpreter(state, PhysicalDevice::chose(&state, &instance, &surface), "p_device selected"); 
        let device = vk_create_interpreter(state, Device::create(&state, &instance, &p_device), "device"); 
        let swapchain = vk_create_interpreter(state, Swapchain::create(&state, &window, &instance, &surface, &p_device, &device), "swapchain");
        let render_pass = vk_create_interpreter(state, RenderPass::create(&state, &device, &swapchain), "render_pass");
        let pipeline = vk_create_interpreter(state, Pipeline::create(&state, &device, &render_pass), "pipeline");
        let framebuffers = vk_create_interpreter(state, SCFramebuffers::create(&state, &device, &swapchain, &render_pass), "framebuffer");
        let command_control = vk_create_interpreter(state, CommandControl::create(&state, &p_device, &device), "command_control");
        let sync_objects = vk_create_interpreter(state, SyncObjects::create(&state, &device), "sync_objects");
        
        VInit{
            state: state,
            instance: VkObj::new(instance),
            messenger: match messenger {
                Some(holder) => {Some(VkObj::new(holder))}
                None => None
            },
            p_device: p_device,
            surface: VkObj::new(surface),
            device: VkObj::new(device),
            swapchain: VkObjDevDep::new(swapchain),
            render_pass: VkObjDevDep::new(render_pass),
            pipeline: VkObjDevDep::new(pipeline),
            sc_framebuffers: VkObjDevDep::new(framebuffers),
            command_control: VkObjDevDep::new(command_control),
            sync_objects: VkObjDevDep::new(sync_objects),
        }
    }
    
    pub fn draw_frame(&mut self) {
        
        unsafe{self.device.wait_for_fences(&self.sync_objects.in_flight_fence[..], true, u64::MAX)}.expect("waiting for fence should not fail");
        unsafe{self.device.reset_fences(&self.sync_objects.in_flight_fence[..])}.expect("waiting for fence should not fail");
        let (image_index, _) = unsafe{self.swapchain.acquire_next_image(self.swapchain.swapchain, u64::MAX, self.sync_objects.image_available_semaphore[0], vk::Fence::null()).expect("next image should not fail")};
        
        unsafe{self.device.reset_command_buffer(self.command_control.buffer, vk::CommandBufferResetFlags::empty())}.expect("reseting command should not fail");
        self.command_control.record_command_buffer(&self.state, &self.device, &self.swapchain, &self.render_pass, &self.pipeline, &self.sc_framebuffers, image_index);
        
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffer_slice = unsafe{std::slice::from_raw_parts(addr_of!(self.command_control.buffer), 1)};
        
        let submit_info = [
            vk::SubmitInfo::builder()
                .wait_semaphores(&self.sync_objects.image_available_semaphore[..])
                .wait_dst_stage_mask(&wait_stages[..])
                .command_buffers(command_buffer_slice)
                .signal_semaphores(&self.sync_objects.render_finished_semaphore[..])
                .build()
        ];
        
        unsafe{self.device.queue_submit(self.device.queue_handles.graphics, &submit_info[..], self.sync_objects.in_flight_fence[0])}.expect("should not fail");
        
        let swapchain_slice = unsafe{std::slice::from_raw_parts(addr_of!(self.swapchain.swapchain), 1)};
        let image_index_slice = unsafe{std::slice::from_raw_parts(addr_of!(image_index), 1)};
        
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&self.sync_objects.render_finished_semaphore[..])
            .swapchains(swapchain_slice)
            .image_indices(image_index_slice);
        
        unsafe{self.swapchain.queue_present(self.device.queue_handles.presentation, &present_info)}.expect("present should not fail");
        
        
    }
    
    pub fn wait_idle(&self) {
        unsafe{self.device.device_wait_idle()}.expect("waiting for iddle should not fail");
    }
}


#[inline]
fn vk_create_interpreter<T, A:std::fmt::Debug>(state:State, result:Result<T, A>, name:&'static str) -> T {
    match result {
        Ok(device) => {
            if state.v_nor() {
                println!("[0]:{}", name);
            }
            device
        }
        Err(err) => {panic!("error in {} {:?}", name, err);}
    }
}

impl Drop for VInit {
    fn drop(&mut self) {
        
        self.sync_objects.device_drop(&self.state, &self.device);
        self.command_control.device_drop(&self.state, &self.device);
        self.sc_framebuffers.device_drop(&self.state, &self.device);
        self.pipeline.device_drop(&self.state, &self.device);
        self.render_pass.device_drop(&self.state, &self.device);
        self.swapchain.device_drop(&self.state, &self.device);
        self.device.active_drop(&self.state);
        self.surface.active_drop(&self.state);
        
        match &mut self.messenger {
            Some(ref mut messenger) => {
                messenger.active_drop(&self.state);
            }
            None => {
                if self.state.v_nor() {
                    println!("No Messenger to delete");
                }
            }
        }
        
        self.instance.active_drop(&self.state);
    }
}



