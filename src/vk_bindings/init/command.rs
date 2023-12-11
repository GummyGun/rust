use ash::{
    vk,
    prelude::VkResult,
};

use super::{
    DeviceDrop,
    device::Device,
    p_device::PhysicalDevice,
    swapchain::Swapchain,
    render_pass::RenderPass,
    pipeline::Pipeline,
    buffers::Buffer,
};

use crate::{
    State,
    constants,
    graphics::VERTEX_INDEX,
};

use std::{
    slice::from_raw_parts,
    ptr::addr_of,
};

pub struct CommandControl{
    pub pool: vk::CommandPool,
    pub buffer: [vk::CommandBuffer; constants::FIF],
    pub staging_buffer: vk::CommandBuffer,
}


impl CommandControl {
    pub fn create(state:&State, p_device:&PhysicalDevice, device:&Device) -> VkResult<Self> {
        if  state.v_exp() {
            println!("\nCREATING:\tCOMMAND CONTROL STRUCTURES");
        }
        let create_info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(p_device.queues.graphics_family);
        let command_pool = unsafe{device.create_command_pool(&create_info, None)}?;
        
        if  state.v_exp() {
            println!("\nALLOCATING:\tCOMMAND BUFFERS");
            println!("allocating graphics command buffers");
        }
        let create_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(constants::FIF_U32);
        
        let buffer_vec = unsafe{device.allocate_command_buffers(&create_info)}?;
        
        if  state.v_exp() {
            println!("allocating staging command buffer");
        }
        let sb_create_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);
        
        let staging_buffer = unsafe{device.allocate_command_buffers(&sb_create_info)}?;
        
        let mut buffer_arr = [vk::CommandBuffer::null(); constants::FIF];
        for (index, buffer) in buffer_vec.into_iter().enumerate() {
            buffer_arr[index] = buffer;
        }
        
        Ok(Self{
            pool: command_pool,
            buffer: buffer_arr,
            staging_buffer: staging_buffer[0],
        })
    }
    
    pub fn record_command_buffer(
        &mut self, 
        state:&State, 
        device:&Device, 
        swapchain:&Swapchain, 
        render_pass:&RenderPass, 
        pipeline:&Pipeline, 
        vertex_buffer:&Buffer, 
        index_buffer:&Buffer,
        image_index:u32, 
        frame_index:usize,
    ) {
        if  state.v_dmp() {
            println!("\nFILLING:\tCOMMAND BUFFER");
        }
        let image_index_usize = usize::try_from(image_index).unwrap();
        
        let command_buffer_begin = vk::CommandBufferBeginInfo::builder();
        
        unsafe{device.begin_command_buffer(self.buffer[frame_index], &command_buffer_begin)}.unwrap();
        
        let scissor = [
            vk::Rect2D::builder()
                .offset(*vk::Offset2D::builder().x(0).y(0))
                .extent(swapchain.extent)
                .build()
        ];
        
        let viewport = [
            ash::vk::Viewport::builder()
                .x(0f32)
                .y(0f32)
                .width(swapchain.extent.width as f32)
                .height(swapchain.extent.height as f32)
                .min_depth(0f32)
                .max_depth(0f32)
                .build()
        ];
        
        let clear_color = [
            vk::ClearValue{
                color: vk::ClearColorValue{float32:[0.0f32; 4]},
            }
        ];
        
        let render_pass_begin = vk::RenderPassBeginInfo::builder()
            .render_pass(render_pass.as_inner())
            .framebuffer(swapchain.framebuffers[image_index_usize])
            .render_area(scissor[0])
            .clear_values(&clear_color[..]);
        
        let device_size:[vk::DeviceSize;1] = [0];
        let vertex_buffer_slice = unsafe{from_raw_parts(addr_of!(vertex_buffer.buffer), 1)};
        
        //initialize the command buffer
        unsafe{device.cmd_begin_render_pass(self.buffer[frame_index], &render_pass_begin, vk::SubpassContents::INLINE)};
        //bind command buffer to graphics pipeline
        
        unsafe{device.cmd_bind_pipeline(self.buffer[frame_index], vk::PipelineBindPoint::GRAPHICS, pipeline.as_inner())};
        
        
        unsafe{device.cmd_set_viewport(self.buffer[frame_index], 0, &viewport[..])};
        unsafe{device.cmd_set_scissor(self.buffer[frame_index], 0, &scissor[..])};
        
        unsafe{device.cmd_bind_vertex_buffers(self.buffer[frame_index], 0, vertex_buffer_slice, &device_size)};
        unsafe{device.cmd_bind_index_buffer(self.buffer[frame_index], index_buffer.buffer, 0, vk::IndexType::UINT16)};
        
        unsafe{device.cmd_draw_indexed(self.buffer[frame_index], u32::try_from(VERTEX_INDEX.len()).expect("vertex to draw should fit in u32"), 1, 0, 0, 0)};
        unsafe{device.cmd_end_render_pass(self.buffer[frame_index])};
        
        unsafe{device.end_command_buffer(self.buffer[frame_index])}.unwrap();
    }
    
}

impl DeviceDrop for CommandControl {
    fn device_drop(&mut self, state:&State, device:&Device) {
        if state.v_nor() {
            println!("[0]:deallocating command buffer");
        }
        if state.v_nor() {
            println!("[0]:deleting command pool");
        }
        unsafe{device.destroy_command_pool(self.pool, None)};
    }
}
