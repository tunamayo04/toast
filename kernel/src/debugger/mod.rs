use alloc::string::String;
use alloc::vec::Vec;
use limine::memory_map::EntryType;
use x86_64::instructions::tables::sgdt;
use crate::arch::x86_64::registers::{cr0, cr2, cr3, cr4};
use crate::graphics::framebuffer_device::Writer;
use crate::memory::{MemoryManager, PAGE_SIZE};
use crate::MEMORY_MAP_REQUEST;

pub fn run_debug_shell() {
    Writer::instance().unwrap().lock().clear_screen();
    println!("TOAST DEBUGGING ENVIRONMENT");
    print!(">")
}

pub fn run_command(command: &String) {
    let command_parts: Vec<&str> = command.split(" ").collect();

    match command_parts[0] {
        "meminfo" => { mem_info(&command_parts[1..]); },
        "cpuinfo" => { cpu_info(&command_parts[1..]); },
        _ => {
            println!("unrecognized command \"{}\"", command_parts[0]);
            print!(">");
        }
    }
}

pub fn mem_info(args: &[&str]) {
    match args[0] {
        "alloc" => {
            let allocated_memory = MemoryManager::get_allocated_memory_amount();
            println!("physical memory allocated: {} bytes ({} frames)", allocated_memory.0, allocated_memory.0 / PAGE_SIZE);
            println!("virtual memory allocated: {} bytes ({} pages)", allocated_memory.1, allocated_memory.1 / PAGE_SIZE);
        },
        "virtual" => {
            MemoryManager::instance().lock().virtual_memory_manager.display_memory();
            print!(">");
        },
        "physical" => {
            // MemoryManager::instance().lock().frame_allocator.display_memory();
            print!(">");
        },/*
        "heap" => {
            let heap_bounds = ALLOCATOR.lock().heap_bounds();
            println!("heap from 0x{:X} to 0x{:X}", heap_bounds.0, heap_bounds.1);
            print!(">")
        },*/
        "map" => {
            print_memory_map();
            print!(">");
        }
        _ => {
            println!("unrecognized argument \"{}\"", args[0]);
            print!(">");
        }
    }
}

pub fn cpu_info(args: &[&str]) {
    match args[0] {
        "regs" => {
            println!("CR0={:X} CR2={:X} CR3={:X} CR4={:X}", cr0(), cr2(), cr3(), cr4());

            let gdt = sgdt().base;
            println!("GDT={:X}", gdt);
            print!(">");
        }
        _ => {
            println!("unrecognized argument \"{}\"", args[0]);
            print!(">");
        }
    }
}

fn print_memory_map() {
    MEMORY_MAP_REQUEST.get_response().unwrap().entries().iter().for_each(|entry| {
        match entry.entry_type {
            EntryType::USABLE => println!("0x{:016X} - 0x{:016X} ({:016X}) : usable", entry.base, entry.base + entry.length, entry.length),
            EntryType::RESERVED => println!("0x{:016X} - 0x{:016X} ({:016X}) : reserved",  entry.base, entry.base + entry.length, entry.length),
            EntryType::ACPI_RECLAIMABLE => println!("0x{:016X} - 0x{:016X} ({:016X}) : acpi reclaimable",  entry.base, entry.base + entry.length, entry.length),
            EntryType::ACPI_NVS => println!("0x{:016X} - 0x{:016X} ({:016X}) : acpi nvs",  entry.base, entry.base + entry.length, entry.length),
            EntryType::BAD_MEMORY => println!("0x{:016X} - 0x{:016X} ({:016X}) : bad memory",  entry.base, entry.base + entry.length, entry.length),
            EntryType::BOOTLOADER_RECLAIMABLE => println!("0x{:016X} - 0x{:016X} ({:016X}) : bootloader reclaimable",  entry.base, entry.base + entry.length, entry.length),
            EntryType::KERNEL_AND_MODULES => println!("0x{:016X} - 0x{:016X} ({:016X}) : kernel",  entry.base, entry.base + entry.length, entry.length),
            EntryType::FRAMEBUFFER => println!("0x{:016X} - 0x{:016X} ({:016X}) : framebuffer",  entry.base, entry.base + entry.length, entry.length),
            _ => ()
        }
    });
}