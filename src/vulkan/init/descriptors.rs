use crate::AAError;
use crate::macros;
use crate::logger;
use crate::errors::messages::STANDARD_CONV;
use crate::errors::messages::GRANTED;
use crate::errors::messages::SIMPLE_VK_FN;

use super::VkDestructor;
use super::VkDestructorArguments;
use super::Device;
use super::Image;

use std::slice::from_ref;
use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Mul;
use std::ops::MulAssign;

use ash::vk;
use arrayvec::ArrayVec;

const DESCRIPTOR_TYPE_COUNT:usize = 17;

pub struct DescriptorLayoutBuilder {
    bindings: Vec<vk::DescriptorSetLayoutBinding>,
    type_count: DescriptorPoolCount,
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct DescriptorLayout {
    set_layout: vk::DescriptorSetLayout,
}

macros::impl_deref!(DescriptorLayout, vk::DescriptorSetLayout, set_layout);


#[derive(Default, Debug, Clone)]
pub struct DescriptorPoolCount {
    count: [u32; DESCRIPTOR_TYPE_COUNT],
}

pub struct DescriptorPoolAllocator {
    pool: vk::DescriptorPool,
}

pub fn init_descriptors(device:&mut Device, ds_layout_builder:&mut DescriptorLayoutBuilder, render_image:&Image) -> (DescriptorLayout, DescriptorPoolAllocator, vk::DescriptorSet) {
    //logger::init();
    ds_layout_builder.add_binding(0, vk::DescriptorType::STORAGE_IMAGE);
    let (ds_layout, mut types_in_layout) = ds_layout_builder.build(device, vk::ShaderStageFlags::COMPUTE).unwrap();
    
    let gds_pool: GDescriptorAllocator = GDescriptorAllocator::create(device, types_in_layout.clone()).unwrap();
    gds_pool.destruct(VkDestructorArguments::Dev(device));
    
    types_in_layout *= 10;//allocate 10 DS
    let mut ds_pool = DescriptorPoolAllocator::create(device, types_in_layout).unwrap();
    let ds_set = ds_pool.allocate(device, ds_layout).unwrap();
    
    let mut descriptor_image_info = vk::DescriptorImageInfo::default();
    descriptor_image_info.image_layout = vk::ImageLayout::GENERAL;
    descriptor_image_info.image_view = render_image.view;
    
    let write_descriptor_set = vk::WriteDescriptorSet::builder()
        .dst_binding(0)
        .dst_set(ds_set)
        .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
        .image_info(from_ref(&descriptor_image_info));
    
    unsafe{device.update_descriptor_sets(from_ref(&write_descriptor_set), &[])};
    (ds_layout, ds_pool, ds_set)
}

impl DescriptorLayoutBuilder {
    
    pub fn create() -> Result<Self, ()> {
        //logger::dlb::create();
        Ok(Self{
            bindings: Vec::new(),
            type_count: DescriptorPoolCount::default(),
        })
    }
    
    pub fn add_binding(&mut self, binding:u32, d_type:vk::DescriptorType) {
        let mut holder = vk::DescriptorSetLayoutBinding::default();
        holder.binding = binding;
        holder.descriptor_count = 1;
        holder.descriptor_type = d_type;
        self.bindings.push(holder);
        self.type_count.add_type_count(d_type, 1);
    }
    
    pub fn reset(&mut self) {
        self.bindings.clear();
    }
    
    pub fn build(&mut self, device:&mut Device, shader_stage:vk::ShaderStageFlags) -> Result<(DescriptorLayout, DescriptorPoolCount), AAError> {
        
        //logger::dl::create();
        for binding in self.bindings.iter_mut(){
            binding.stage_flags |= shader_stage;
        }
        
        let create_info = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(&self.bindings[..]);
        
        let holder_layout = DescriptorLayout{
            set_layout: unsafe{device.create_descriptor_set_layout(&create_info, None)}?,
        };
        let holder_count = self.type_count.clone();
        
        self.bindings.clear();
        self.type_count = DescriptorPoolCount::default();
        
        Ok((holder_layout, holder_count))
    }
    
}


impl VkDestructor for DescriptorLayoutBuilder {
    fn destruct(self, mut args:VkDestructorArguments) {
        //logger::dlb::destruct();
        args.unwrap_none();
    }
}

impl VkDestructor for DescriptorLayout {
    fn destruct(self, mut args:VkDestructorArguments) {
        //logger::dl::destruct();
        let device = args.unwrap_dev();
        unsafe{device.destroy_descriptor_set_layout(self.set_layout, None)};
    }
}


impl DescriptorPoolCount {
    #[allow(dead_code)]
    pub fn set_type_count(&mut self, d_type:vk::DescriptorType, amount:u32) {
        let index = Self::descriptor_type_to_index(d_type);
        self.count[index] = amount;
    }
    
    #[allow(dead_code)]
    pub fn add_type_count(&mut self, d_type:vk::DescriptorType, amount:u32) {
        self.count[usize::try_from(d_type.as_raw()).expect(STANDARD_CONV)] += amount;
    }
    
    #[allow(dead_code)]
    pub fn update(&mut self, magnitud:u32, other:&Self) {
        *self += other.clone()*magnitud;
    }
    
    pub fn max_sets(&self) -> u32 {
        self.count.iter().fold(0u32, |last, current|last+current)
    }
    
    fn descriptor_type_to_index(d_type:vk::DescriptorType) -> usize {
        use vk::DescriptorType as DT;
        match d_type {
            regular @ (DT::SAMPLER | DT::COMBINED_IMAGE_SAMPLER | DT::SAMPLED_IMAGE | DT::STORAGE_IMAGE | DT::UNIFORM_TEXEL_BUFFER | 
                DT::STORAGE_TEXEL_BUFFER | DT::UNIFORM_BUFFER | DT::STORAGE_BUFFER | DT::UNIFORM_BUFFER_DYNAMIC | 
                DT::STORAGE_BUFFER_DYNAMIC | DT::INPUT_ATTACHMENT) => {
                usize::try_from(regular.as_raw()).expect(STANDARD_CONV)
            }
            DT::INLINE_UNIFORM_BLOCK => 11usize,
            DT::ACCELERATION_STRUCTURE_KHR => 12usize,
            DT::ACCELERATION_STRUCTURE_NV => 13usize,
            DT::SAMPLE_WEIGHT_IMAGE_QCOM => 14usize,
            DT::BLOCK_MATCH_IMAGE_QCOM => 15usize,
            DT::MUTABLE_EXT => 16usize,
            /*
            DT::INLINE_UNIFORM_BLOCK_EXT => 17usize,
            DT::MUTABLE_VALVE => 18usize,
            */
            _ => {panic!("descriptor not supported");}
        }
    } 
    
    fn index_to_descriptor_type(index:usize) -> vk::DescriptorType {
        use vk::DescriptorType as DT;
        let index_i32 = i32::try_from(index).expect(GRANTED);
        match index_i32 {
            0..=10 => {
                DT::from_raw(index_i32)
            }
            11 => DT::INLINE_UNIFORM_BLOCK,
            12 => DT::ACCELERATION_STRUCTURE_KHR,
            13 => DT::ACCELERATION_STRUCTURE_NV,
            14 => DT::SAMPLE_WEIGHT_IMAGE_QCOM,
            15 => DT::BLOCK_MATCH_IMAGE_QCOM,
            16 => DT::MUTABLE_EXT,
            /*
            17 => DT::INLINE_UNIFORM_BLOCK_EXT,
            18 => DT::MUTABLE_VALVE,
            */
            _ => {panic!("bad index");}
        }
        
    } 
    
    fn fill_pool_sizes(&self, target:&mut [vk::DescriptorPoolSize; DESCRIPTOR_TYPE_COUNT]) -> usize {
        let mut count = 0usize;
        let mut target_iter = target.iter_mut();
        for (index, value) in self.count.iter().enumerate(){
            if value > &0 {
                let target = target_iter.next().expect(GRANTED);//it is running in a array big enought 
                target.ty = Self::index_to_descriptor_type(index);
                target.descriptor_count = *value;
                count += 1;
            }
        }
        count
    }
    
    fn fill_pool_sizes_array_vec(&self, target:&mut ArrayVec<vk::DescriptorPoolSize, DESCRIPTOR_TYPE_COUNT>, coheficient:u32) -> Result<u32, ()> {
        if !target.is_empty() {
            return Err(());
        }
        
        let mut count = 0u32;
        let mut target_iter = target.iter_mut();
        
        for (index, value) in self.count.into_iter().enumerate(){
            if value > 0 {
                
                let mut holder = vk::DescriptorPoolSize::default();
                holder.ty = DescriptorPoolCount::index_to_descriptor_type(index);
                holder.descriptor_count = coheficient*value;//descriptor_type_count*descriptor_type_count;
                count += holder.descriptor_count;
                target.push(holder);
            }
        }
        Ok(count)
    }
    
}

impl Mul<u32> for DescriptorPoolCount {
    type Output = Self;
    fn mul(mut self, magnitude:u32) -> Self::Output {
        self *= magnitude;
        self
    }
}

impl MulAssign<u32> for DescriptorPoolCount {
    fn mul_assign(&mut self, magnitude:u32) {
        for iter in 0..self.count.len() {
            self.count[iter] *= magnitude;
        }
    }
}

impl Add<Self> for DescriptorPoolCount {
    type Output = Self;
    fn add(mut self, other:Self) -> Self::Output {
        self += &other;
        self
    }
}

impl AddAssign<Self> for DescriptorPoolCount {
    fn add_assign(&mut self, other:Self) {
        *self += &other;
    }
}

impl AddAssign<&Self> for DescriptorPoolCount {
    fn add_assign(&mut self, other:&Self) {
        for iter in 0..self.count.len() {
            self.count[iter] += other.count[iter];
        }
    }
}


impl DescriptorPoolAllocator {
    
    pub fn create(device:&mut Device, count:DescriptorPoolCount) -> Result<Self, AAError> {
        //logger::dpa::create();
        
        let mut pool_sizes:[vk::DescriptorPoolSize; DESCRIPTOR_TYPE_COUNT] = Default::default();
        let max_sets = count.max_sets();
        
        let count = count.fill_pool_sizes(&mut pool_sizes);
        let pool_sizes_slice = &pool_sizes[..count];
        
        let create_info = vk::DescriptorPoolCreateInfo::builder()
            .max_sets(max_sets)
            .pool_sizes(pool_sizes_slice);
        
        let holder = unsafe{device.create_descriptor_pool(&create_info, None)}?;
        Ok(Self{
            pool: holder,
        })
    }
    
    pub fn reset(&mut self, device:&mut Device) {
        
        unsafe{device.reset_descriptor_pool(self.pool, vk::DescriptorPoolResetFlags::empty())}.expect(SIMPLE_VK_FN);
    }
    
    pub fn allocate(&mut self, device:&mut Device, layout:DescriptorLayout) -> Result<vk::DescriptorSet, AAError> {
        
        let allocate_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(self.pool)
            .set_layouts(from_ref(&layout.set_layout));
        
        let holder = unsafe{device.allocate_descriptor_sets(&allocate_info)}?;
        
        Ok(holder[0])
    }
}

impl VkDestructor for DescriptorPoolAllocator {
    fn destruct(self, mut args:VkDestructorArguments) {
        //logger::dpa::destruct();
        let device = args.unwrap_dev();
        unsafe{device.destroy_descriptor_pool(self.pool, None)};
    }
}



pub struct GDescriptorAllocator<const UPPER_LIMIT_PER_POOL:u32 = 4092, const INITIAL_GROUPS:u32 = 256> {
    ratios: DescriptorPoolCount,
    full_pools: Vec<vk::DescriptorPool>,
    ready_pools: Vec<vk::DescriptorPool>,
    max_descriptors_groups: u32,
}



impl<const UPPER_LIMIT_PER_POOL:u32, const INITIAL_GROUPS:u32> GDescriptorAllocator<UPPER_LIMIT_PER_POOL, INITIAL_GROUPS> {
    
    pub fn create(device:&mut Device, ratios:DescriptorPoolCount, ) -> Result<Self, AAError> {
        
        logger::create!("descriptor allocator");
        
        let full_pools = Vec::new();
        let ready_pools = Vec::new();
        
        
        let mut holder = Self{
            ratios,
            full_pools,
            ready_pools,
            max_descriptors_groups: INITIAL_GROUPS,
        };
        let initial_pool = holder.create_pool(device);
        holder.ready_pools.push(initial_pool?);
        Ok(holder)
    }
    
    fn get_pool(&mut self, device:&mut Device) -> Result<vk::DescriptorPool, AAError> {
        if self.ready_pools.is_empty() {
            self.max_descriptors_groups *=2;
            let holder = self.create_pool(device);
            holder
        } else {
            Ok(self.ready_pools.pop().unwrap())
        }
    }
    
    fn create_pool(&mut self, device:&mut Device) -> Result<vk::DescriptorPool, AAError> {
        let mut ratios = ArrayVec::new();
        let count = self.ratios.fill_pool_sizes_array_vec(&mut ratios, 1).expect(GRANTED);
        
        let descriptor_pool_ci = vk::DescriptorPoolCreateInfo::builder()
            .max_sets(count)
            .pool_sizes(&ratios[..]);
            
        
        unsafe{device.create_descriptor_pool(&descriptor_pool_ci, None)}.map_err(|error|error.into())
    } 
    
    pub fn clear_pools(&mut self, device:&Device) {
        for pool in &self.ready_pools {
            unsafe{device.reset_descriptor_pool(*pool, vk::DescriptorPoolResetFlags::empty())}.expect(SIMPLE_VK_FN);
        }
        for pool in self.full_pools.iter() {
            unsafe{device.reset_descriptor_pool(*pool, vk::DescriptorPoolResetFlags::empty())}.expect(SIMPLE_VK_FN);
            self.ready_pools.push(*pool);
        }
        self.full_pools.clear();
    }
    
    pub fn allocate(&mut self, device:&mut Device, layout:&vk::DescriptorSetLayout) -> Result<vk::DescriptorSet, AAError> {
        let mut pool_to_use = self.get_pool(device)?;
        
        let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(pool_to_use)
            .set_layouts(from_ref(layout));
        
        let holder = unsafe{device.allocate_descriptor_sets(&descriptor_set_allocate_info)};
        
        let set = match holder {
            Ok(mut descriptor_set) => {
                let holder = descriptor_set.pop().expect(GRANTED);
                holder
            }
            Err(retryable_error) if (retryable_error == vk::Result::ERROR_OUT_OF_POOL_MEMORY) || (retryable_error == vk::Result::ERROR_FRAGMENTED_POOL) => {
                self.full_pools.push(pool_to_use);
                
                pool_to_use = self.get_pool(device)?;
                let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo::builder()
                    .descriptor_pool(pool_to_use)
                    .set_layouts(from_ref(layout));
                
                match unsafe{device.allocate_descriptor_sets(&descriptor_set_allocate_info)} {
                    Ok(mut descriptor_set) => {
                        let holder = descriptor_set.pop().expect(GRANTED);
                        holder
                    }
                    Err(err) => {
                        panic!("descriptors can't be created {:?}", err);
                    }
                }
            }
            Err(error) => {return Err(error.into());}
        };
        self.ready_pools.push(pool_to_use);
        return Ok(set);
    }
    
}

impl VkDestructor for GDescriptorAllocator {
    fn destruct(self, mut args:VkDestructorArguments) {
        //logger::dpa::destruct();
        let device = args.unwrap_dev();
        for pool in &self.ready_pools {
            unsafe{device.destroy_descriptor_pool(*pool, None)};
        }
        for pool in self.full_pools.iter() {
            unsafe{device.destroy_descriptor_pool(*pool, None)};
        }
    }
}




