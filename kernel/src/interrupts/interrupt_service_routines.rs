use core::arch::asm;
use core::fmt;
use core::fmt::Formatter;
use crate::drivers::ps2::keyboard::{PS2Keyboard};
use crate::interrupts::{MASTER_PIC_COMMAND_PORT, PIC_EOI};
use crate::task::keyboard::add_scancode;

pub type HandlerFuncWithoutErrCode = extern "x86-interrupt" fn(InterruptStackFrame);
pub type HandlerFuncWithErrCode = extern "x86-interrupt" fn(InterruptStackFrame, error_code: u64);

#[repr(C)]
pub struct InterruptStackFrame {
    instruction_pointer: u64,
    code_segment: u64,
    cpu_flags: u64,
    stack_pointer: u64,
    stack_segment: u64,
}

impl fmt::Debug for InterruptStackFrame {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("InterruptStackFrame")
            .field("instruction_pointer", &format_args!("0x{:X}", self.instruction_pointer))
            .field("code_segment", &format_args!("0x{:X}", self.code_segment))
            .field("cpu_flags", &format_args!("{:b}", self.cpu_flags))
            .field("stack_pointer", &format_args!("0x{:X}", self.stack_pointer))
            .field("stack_segment", &format_args!("0x{:X}", self.stack_segment))
            .finish()
    }
}

pub extern "x86-interrupt" fn division_error_handler(stack_frame: InterruptStackFrame) {
    error!("Caught a division error interrupt!");
    println!("{:#?}", stack_frame);
    unsafe { asm!("hlt;"); };
}

pub extern "x86-interrupt" fn debug_handler(stack_frame: InterruptStackFrame) {
    error!("Caught a debug interrupt!");
    println!("{:#?}", stack_frame);
    unsafe { asm!("hlt;"); };
}

pub extern "x86-interrupt" fn non_maskable_interrupt_handler(stack_frame: InterruptStackFrame) {
    error!("Caught a non-maskable interrupt!");
    println!("{:#?}", stack_frame);
    unsafe { asm!("hlt;"); };
}

pub extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    error!("Caught a breakpoint interrupt!");
    println!("{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn overflow_handler(stack_frame: InterruptStackFrame) {
    error!("Caught an overflow interrupt!");
    println!("{:#?}", stack_frame);
    unsafe { asm!("hlt;"); };
}

pub extern "x86-interrupt" fn bound_range_exceeded_handler(stack_frame: InterruptStackFrame) {
    error!("Caught a bound range exceeded interrupt!");
    println!("{:#?}", stack_frame);
    unsafe { asm!("hlt;"); };
}

pub extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: InterruptStackFrame) {
    error!("Caught an invalid opcode interrupt!");
    println!("{:#?}", stack_frame);
    unsafe { asm!("hlt;"); };
}

pub extern "x86-interrupt" fn device_not_available_handler(stack_frame: InterruptStackFrame) {
    error!("Caught a device not available interrupt!");
    println!("{:#?}", stack_frame);
    unsafe { asm!("hlt;"); };
}

pub extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, error_code: u64) {
    error!("Caught a double fault! Error code 0x{:X}", error_code);
    println!("{:#?}", stack_frame);
    unsafe { asm!("hlt;"); };
}

pub extern "x86-interrupt" fn invalid_tss_handler(stack_frame: InterruptStackFrame, error_code: u64) {
    error!("Caught an invalid tss interrupt! Error code 0x{:X}", error_code);
    println!("{:#?}", stack_frame);
    unsafe { asm!("hlt;"); };
}

pub extern "x86-interrupt" fn segment_not_present_handler(stack_frame: InterruptStackFrame, error_code: u64) {
    error!("Caught a segment not present interrupt! Error code 0x{:X}", error_code);
    println!("{:#?}", stack_frame);
    unsafe { asm!("hlt;"); };
}

pub extern "x86-interrupt" fn stack_segment_fault_handler(stack_frame: InterruptStackFrame, error_code: u64) {
    error!("Caught a stack segment fault interrupt! Error code 0x{:X}", error_code);
    println!("{:#?}", stack_frame);
    unsafe { asm!("hlt;"); };
}

pub extern "x86-interrupt" fn general_protection_fault_handler(stack_frame: InterruptStackFrame, error_code: u64) {
    error!("Caught a general protection fault interrupt! Error code 0x{:X}", error_code);
    println!("{:#?}", stack_frame);
    unsafe { asm!("hlt;"); };
}

pub extern "x86-interrupt" fn page_fault_handler(stack_frame: InterruptStackFrame, error_code: u64) {
    error!("Caught a page fault interrupt! Error code 0x{:X}", error_code);
    println!("{:#?}", stack_frame);
    unsafe { asm!("hlt;"); };
}

pub extern "x86-interrupt" fn x87_floating_point_exception_handler(stack_frame: InterruptStackFrame) {
    error!("Caught an x86 floating point exception interrupt!");
    println!("{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn alignment_check_handler(stack_frame: InterruptStackFrame, error_code: u64) {
    error!("Caught an alignment check interrupt! Error code 0x{:X}", error_code);
    println!("{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn machine_check_handler(stack_frame: InterruptStackFrame) {
    error!("Caught a machine check interrupt!");
    println!("{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn simd_floating_point_exception_handler(stack_frame: InterruptStackFrame) {
    error!("Caught a SIMD floating point exception interrupt!");
    println!("{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn virtualization_exception_handler(stack_frame: InterruptStackFrame) {
    error!("Caught a virtualization exception interrupt!");
    println!("{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn control_protection_exception_handler(stack_frame: InterruptStackFrame, error_code: u64) {
    error!("Caught a control protection exception interrupt! Error code 0x{:X}", error_code);
    println!("{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn hypervisor_injection_exception_handler(stack_frame: InterruptStackFrame) {
    error!("Caught a hypervisor injection exception interrupt!");
    println!("{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn vmm_communication_exception_handler(stack_frame: InterruptStackFrame, error_code: u64) {
    error!("Caught a VMM communication exception interrupt! Error code 0x{:X}", error_code);
    println!("{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn security_exception_handler(stack_frame: InterruptStackFrame, error_code: u64) {
    error!("Caught a security exception interrupt! Error code 0x{:X}", error_code);
    println!("{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn irq0_handler(stack_frame: InterruptStackFrame) {
    println!("Caught IRQ0!");
    println!("{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn irq1_handler(stack_frame: InterruptStackFrame) {
    let scancode = PS2Keyboard::interrupt_read_byte();
    add_scancode(scancode);

    MASTER_PIC_COMMAND_PORT.lock().write(PIC_EOI).unwrap();
}

pub extern "x86-interrupt" fn irq2_handler(stack_frame: InterruptStackFrame) {
    println!("Caught IRQ2!");
    println!("{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn irq3_handler(stack_frame: InterruptStackFrame) {
    println!("Caught IRQ3!");
    println!("{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn irq4_handler(stack_frame: InterruptStackFrame) {
    println!("Caught IRQ4!");
    println!("{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn irq5_handler(stack_frame: InterruptStackFrame) {
    println!("Caught IRQ5!");
    println!("{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn irq6_handler(stack_frame: InterruptStackFrame) {
    println!("Caught IRQ6!");
    println!("{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn irq7_handler(stack_frame: InterruptStackFrame) {
    println!("Caught IRQ7!");
    println!("{:#?}", stack_frame);
}