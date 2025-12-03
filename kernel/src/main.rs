#![no_std]
#![no_main]
#![allow(dead_code)]
#![allow(internal_features)]
#![feature(lang_items)]
#![feature(ptr_internals)]
#![feature(panic_info_message)]
#![feature(allocator_api)]
#![feature(const_mut_refs)]
#![feature(abi_x86_interrupt)]
#![feature(never_type)]
#![feature(new_uninit)]
#![feature(str_from_raw_parts)]
#![feature(extract_if)]
#![feature(btree_extract_if)]
#![feature(custom_test_frameworks)]
#![feature(int_roundings)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate downcast_rs;
extern crate alloc;

use alloc::string::String;
use core::panic::PanicInfo;
use lazy_static::lazy_static;
use limine::BaseRevision;
use limine::request::{FramebufferRequest, HhdmRequest, MemoryMapRequest};
use x86_64::registers::model_specific::Efer;
use x86_64::registers::control::{Cr0, Cr0Flags, EferFlags};
use drivers::ps2::init_ps2_controller;
use drivers::ps2::keyboard::PS2Keyboard;
use drivers::ps2::PS2DeviceType;
use fs::ext2::mount_filesystem;
use drivers::fbdev::FrameBufferDevice;
use fs::Vfs;
use graphics::framebuffer_device::Writer;
use interrupts::{INTERRUPT_CONTROLLER, InterruptController};
use memory::{MemoryManager, VirtualAddress};
use task::keyboard::print_key_inputs;
use task::executor::Executor;
use task::Task;
use utils::hcf;
use crate::drivers::cpuid::CPUInfo;

#[cfg(test)]
use crate::utils::tests::{exit_qemu, QemuExitCode, Testable};

#[macro_use]
mod graphics;
#[macro_use]
mod serial;
mod arch;
mod memory;
mod interrupts;
mod utils;
mod drivers;
mod task;
mod fs;
mod debugger;

pub const KERNEL_START_VMA_ADDRESS: VirtualAddress = 0xFFFFFFFF80000000;

lazy_static! {
    pub static ref HHDM_OFFSET: VirtualAddress = HHDM_REQUEST.get_response().expect("could not retrieve the HHDM info").offset() as usize;
}

pub static BASE_REVISION: BaseRevision = BaseRevision::new();

pub static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();
pub static MEMORY_MAP_REQUEST: MemoryMapRequest = MemoryMapRequest::new();
pub static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
unsafe extern fn _entry() {
    assert!(BASE_REVISION.is_supported());

    init();

    #[cfg(test)]
    test_main();

    hcf();
}

unsafe fn init() {
    if let Err(err) = MemoryManager::init(MEMORY_MAP_REQUEST.get_response().expect("could not retrieve the memory map")) {
        panic!("{}", err);
    };

    FRAMEBUFFER_REQUEST.get_response().expect("could not retrieve the frame buffer").framebuffers().for_each(|fbdev| {
        FrameBufferDevice::init(&fbdev, String::from("fb0"));
    });
    //FramebufferWriter::init().expect("could not initialize the framebuffer");

    Writer::init().expect("could not initialize the framebuffer");

    Vfs::init();
    FrameBufferDevice::register_devices();

    info!("Toast version v0.0.1-x86_64");
    CPUInfo::print_cpu_info();

    unsafe {
        Efer::write(EferFlags::NO_EXECUTE_ENABLE);
        Cr0::write(Cr0::read() | Cr0Flags::WRITE_PROTECT);
    }

    InterruptController::init();
    //GlobalDescriptorTable::init();

    // init_acpi(boot_info); // TODO: This broke at some point, fix it

    let mut ahci_devices = drivers::pci::ahci::init();

    /*
    let fs = mount_filesystem(&mut ahci_devices[0]);

    let file_name = "/files/file.txt";
    println!("Reading file {}...", file_name);
    let file = fs.get_file_contents(&mut ahci_devices[0], file_name).unwrap_or_else(|| panic!("could not find the file {}", file_name));
    let string_content = core::str::from_utf8(file.as_slice()).expect("Failed to read file");
    println!("{}", string_content);*/

    let ps2_devices = init_ps2_controller();
    let mut executor = Executor::new();
    if ps2_devices.0.is_some() {
        let device = ps2_devices.0.unwrap();
        if let PS2DeviceType::MF2Keyboard = device.device_type() {
            let keyboard: PS2Keyboard = *device.downcast::<PS2Keyboard>().unwrap();
            executor.spawn(Task::new(print_key_inputs(keyboard)));
            INTERRUPT_CONTROLLER.lock().enable_keyboard_interrupts();
        }
    }

    /*
    print!(">");

    executor.run();*/
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    error!("{}", info);

    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failure);

    loop {}
}

#[cfg(test)]
pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }

    exit_qemu(QemuExitCode::Success);
}