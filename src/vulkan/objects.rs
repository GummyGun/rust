use crate::State;
use crate::errors::messanges::BAD_DESTRUCTOR;

use super::Device;
use super::Allocator;

use std::ops::Deref;
use std::ops::DerefMut;

const DROPED_ERR_TEXT:&'static str = "can't run methods on destroyed objects";
const NON_DEV_DROPED_ERR_TEXT:&'static str = "dropping non-destroyed object use device_drop";
const NON_ACT_DROPED_ERR_TEXT:&'static str = "dropping non-destroyed object use active_drop";


pub enum DestructorArguments<'a> {
    None,
    Dev(&'a mut Device),
    DevAll(&'a mut Device,&'a mut Allocator),
}

#[allow(dead_code)]
pub enum DestructorType {
    None,
    Dev,
    DevAll,
}

pub trait VkDestructor {
    fn destruct(self, args:DestructorArguments);
}

pub trait VkDeferedDestructor {
    fn defered_destruct() -> (Box<dyn FnOnce(DestructorArguments)>, DestructorType);
}


/* 
dynamic dispached 
should implement 
*/


pub struct VkWraper<T:VkDestructor>(Option<T>);

impl<T:VkDestructor> VkWraper<T> {
    pub fn new(new:T) -> Self {
        Self(Some(new))
    }
    
    
    pub fn destruct(&mut self, args:DestructorArguments) {
        self.0.take().expect(DROPED_ERR_TEXT).destruct(args);
    }
    
    pub fn take(&mut self) -> T {
        self.0.take().expect(DROPED_ERR_TEXT)
    }
}


impl<T:VkDestructor> Drop for VkWraper<T> {
    fn drop(&mut self) {
        match self.0.as_mut() {
            Some(_) => {eprintln!("{}", NON_ACT_DROPED_ERR_TEXT)}
            None => {}
        }
    }
}

impl<T:VkDestructor> Deref for VkWraper<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.0.as_ref().expect(DROPED_ERR_TEXT)
    }
}

impl<T:VkDestructor> DerefMut for VkWraper<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut().expect(DROPED_ERR_TEXT)
    }
}



impl DestructorArguments<'_> {
    
    pub fn unwrap_none(&mut self) {
        if let DestructorArguments::None = self {
            
        } else {
            panic!("{}", BAD_DESTRUCTOR);
        }
    }
    
    pub fn unwrap_dev(&mut self) -> &Device {
        if let DestructorArguments::Dev(device) = self {
            device
        } else {
            panic!("{}", BAD_DESTRUCTOR);
        }
    }
    
    pub fn unwrap_dev_all(&mut self) -> (&mut Device, &mut Allocator) {
        if let DestructorArguments::DevAll(device, allocator) = self {
            (device, allocator)
        } else {
            panic!("{}", BAD_DESTRUCTOR);
        }
    }
    
}










pub trait DeviceDestroy {
    fn device_destroy(&mut self, state:&State, device:&Device);
}


pub trait ActiveDestroy {
    fn active_drop(&mut self, state:&State);
}


pub struct VkObjDevDep<T:DeviceDestroy>(Option<T>);


impl<T:DeviceDestroy> Drop for VkObjDevDep<T> {
    fn drop(&mut self) {
        match self.0.as_mut() {
            Some(_) => {eprintln!("{}", NON_DEV_DROPED_ERR_TEXT)}
            None => {}
        }
    }
}

impl<T:DeviceDestroy> DeviceDestroy for VkObjDevDep<T> {
    fn device_destroy(&mut self, state:&State, device:&Device) {
        self.0.as_mut().unwrap().device_destroy(state, device);
        self.0 = None;
    }
}

impl<T:DeviceDestroy> Deref for VkObjDevDep<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.0.as_ref().expect(DROPED_ERR_TEXT)
    }
}

impl<T:DeviceDestroy> DerefMut for VkObjDevDep<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut().expect(DROPED_ERR_TEXT)
    }
}

impl<T:DeviceDestroy> VkObjDevDep<T> {
    pub fn new(new:T) -> Self {
        Self(Some(new))
    }
    
    pub fn take(&mut self) -> T {
        self.0.take().unwrap()
    }
}

