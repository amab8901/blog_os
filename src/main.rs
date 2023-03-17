#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use bootloader::{BootInfo, entry_point};

mod serial;
mod vga_buffer;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    // use blog_os::memory;
    use blog_os::memory::BootInfoFrameAllocator;
    use x86_64::{
        structures::paging::Page, 
        VirtAddr,
    };
    
    println!("Hello world{}", "!");
    blog_os::init();
    
    let _phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    
    let mut _frame_allocator = unsafe { 
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };
    
    let page:Page = Page::containing_address(VirtAddr::new(0xdeadbeef000));
    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e) };
    
    let _addresses = [
        0xb8000,
        0x200008,
        0x0100_0020_1a10,
        boot_info.physical_memory_offset,
    ];
    
    #[cfg(test)]
    test_main();

    println!("It did not crash!");
    blog_os::hlt_loop();
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    blog_os::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    blog_os::test_panic_handler(info)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

pub trait Testable {
    fn run(&self);
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}
