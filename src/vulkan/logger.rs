use crate::constants::LOGGING;
use ash::vk;

/*
mod base 
mod instance 
mod d_messenger 
mod surface 
mod device 
mod memory 
mod swapchain 
mod image 
*/
    
pub mod base {
    use super::*;
    
    pub fn create(name:&str) {
        if LOGGING {
            log::trace!("[0]:{}", name);
        }
    }
    
    pub fn debug_messenger_destruct() {
        if LOGGING {
            log::trace!("No Messenger to delete");
        }
    }
}

pub mod instance {
    use super::*;
    
    pub fn destruct() {
        if LOGGING {
            log::trace!("[0]:deleting instance");
        }
    }
}

pub mod d_messenger {
    use super::*;
    
    pub fn destruct() {
        if LOGGING {
            log::trace!("[0]:deleting debug messenger");
        }
    }
}

pub mod surface {
    use super::*;
    
    pub fn destruct() {
        if LOGGING {
            log::trace!("[0]:deleting surface");
        }
    }
}

pub mod device {
    use super::*;
    
    pub fn destruct() {
        if LOGGING {
            log::trace!("[0]:deleting device");
        }
    }
}

pub mod memory {
    use super::*;
    
    pub mod alloc {
        use super::*;
        
        pub fn create() {
            if LOGGING {
                log::trace!("\nCREATING:\tALLOCATOR");
            }
        }
        
        pub fn gpu_allocation(name:&str) {
            if LOGGING {
                log::trace!("allocating gpu memory for :\t{}", name);
            }
        }
        
        pub fn destruct() {
            if LOGGING {
                log::trace!("[0]:deleting allocator");
            }
        }
    }
    
}


pub mod swapchain {
    use super::*;
    
    pub fn create() {
        if LOGGING {
            log::trace!("\nCREATING:\tSWAPCHAIN");
        }
    }
    
    pub fn destruct(state:bool) {
        if LOGGING {
            if state {
                log::trace!("[0]:deleting images");
            } else {
                log::trace!("[0]:deleting swapchain");
            }
        }
    }
    
    pub fn format_chossing(surface_formats: &[vk::SurfaceFormatKHR]) {
        if LOGGING {
            log::trace!("{:#?}", surface_formats);
        }
    }
    
    pub fn found_format(found: bool, format: vk::SurfaceFormatKHR) {
        if LOGGING {
            if found {
                log::trace!("found target {:#?}", format);
            } else {
                log::trace!("didn't found target settling for {:#?}", format);
            }
        }
    }
    
    pub fn present_chossing(present: &[vk::PresentModeKHR]) {
        if LOGGING {
            log::trace!("{:#?}", present);
        }
    }
    
    pub fn found_present(found: bool) {
        if LOGGING {
            if found {
                log::trace!("found target Mailbox");
            } else {
                log::trace!("MAILBOX not available settling for FIFO");
            }
        }
    }
    
    pub fn sc_image_view_creates(index: usize) {
        if LOGGING {
            log::trace!("creating swapchain image {index}");
        }
    }
    
    pub fn extent_chossing(extent: vk::Extent2D) {
        if LOGGING {
            println!("normal display width:{} height:{}", extent.width, extent.height);
        }
    }
    
}

pub mod image {
    use super::*;
    pub fn create(name:Option<&'static str>) {
        if LOGGING {
            match name {
                Some(d_name) => {
                    log::trace!("\nCREATING:\tIMAGE\nType: \t{}",d_name);
                }
                None => {
                    log::trace!("\nCREATING:\tIMAGE");
                }
            }
        }
    }
    
    
    pub fn transitioning_image(old: vk::ImageLayout, new: vk::ImageLayout) {
        if LOGGING {
            log::info!("transitioning image from old:{:?} to new:{:?}", old, new);
        }
    }
    
    pub fn destruct() {
        if LOGGING {
            log::trace!("[0]:deleting image");
        }
    }
    
}

pub mod descriptors {
    use super::*;
    
    pub mod dlb {
        use super::*;
        pub fn create() {
            if LOGGING {
                log::trace!("\nCREATING:\tLAYOUT_BUILDER");
            }
        }
        
        pub fn destruct() {
            if LOGGING {
                log::trace!("[0]:deleting layout_builder");
            }
        }
    }
    
    pub mod dl {
        use super::*;
        pub fn create() {
            if LOGGING {
                log::trace!("\nCREATING:\tDESCRIPTOR_LAYOUT");
            }
        }
        
        pub fn destruct() {
            if LOGGING {
                log::trace!("[0]:deleting descriptor_layout");
            }
        }
    }
    
    pub mod dpa {
        use super::*;
        pub fn create() {
            if LOGGING {
                log::trace!("\nCREATING:\tDESCRIPTOR_POOL");
            }
        }
        
        pub fn destruct() {
            if LOGGING {
                log::trace!("[0]:deleting descriptor_pool");
            }
        }
    }
    
}


