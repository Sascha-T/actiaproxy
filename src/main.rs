#![feature(str_as_str)]
#![feature(let_chains)]

use std::ffi::{c_char, CString};
use std::io;
use std::io::BufRead;
use std::num::ParseIntError;
use lazy_static::lazy_static;

lazy_static! {
    static ref LIB: libloading::Library = unsafe { libloading::Library::new("C:\\AWRoot\\drv\\VCIAccess.dll").unwrap() };
    static ref OPEN_SESSION: libloading::Symbol<'static, unsafe extern fn() -> u32> = unsafe { LIB.get(b"_openSession").unwrap() };
    static ref CLOSE_SESSION: libloading::Symbol<'static, unsafe extern fn() -> u32> = unsafe { LIB.get(b"_closeSession").unwrap() };
    static ref GET_VERSION: libloading::Symbol<'static, unsafe extern fn() -> u32> = unsafe { LIB.get(b"_getVersion").unwrap() };
    static ref GET_FW_VERSION: libloading::Symbol<'static, unsafe extern fn(a: *const c_char, s: u32) -> u32> = unsafe { LIB.get(b"_getFirmwareVersion").unwrap() };
    static ref CHANGE_COM_LINE: libloading::Symbol<'static, unsafe extern fn(to: u32) -> u32> = unsafe { LIB.get(b"_changeComLine").unwrap() };
    static ref BIND_PROTOCOL: libloading::Symbol<'static, unsafe extern fn(a: *const u8, s: u32) -> u32> = unsafe { LIB.get(b"_bindProtocol").unwrap() };
    static ref WRITE_AND_READ: libloading::Symbol<'static, unsafe extern fn(ecu_descriptor: *const u8, ecu_descriptor_s: u32, in_buffer: *const u8, in_buffer_s: u32, out_buffer: *const c_char, out_buffer_s: u32, time_out: u32) -> u32> = unsafe { LIB.get(b"_writeAndRead").unwrap() };
}
fn main() {
    std::env::set_current_dir("C:\\");
    const BUFFER_SIZE: usize = 2048;
    let mut read_buffer: Vec<u8> = vec![0; BUFFER_SIZE as usize];
    unsafe {
        println!("info|rdy");
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let line = line.unwrap();
            let parts = line.split("|").collect::<Vec<&str>>();

            match parts.get(0).unwrap().as_str() {
                "open_session" => {
                    let ret = OPEN_SESSION();
                    println!("open_session|{}", ret);
                }
                "close_session" => {
                    let ret = CLOSE_SESSION();
                    println!("close_session|{}", ret);
                }
                "get_version" => {
                    let ret = GET_VERSION();
                    println!("get_version|{}", ret);
                }
                "get_firmware_version" => {
                    const STRING_SIZE: u32 = 256;

                    let mut fw: Vec<u8> = vec![0; STRING_SIZE as usize];
                    let ptr = fw.as_mut_ptr() as *mut i8;
                    let ret = GET_FW_VERSION(ptr, STRING_SIZE as u32);
                    let s = String::from_utf8(fw);

                    if s.is_err() || ret != 0 {
                        println!("get_firmware_version|{}", ret);
                    } else if let Ok(e) = s {
                        println!("get_firmware_version|{}|{}", ret, e);
                    }
                }
                "bind_protocol" => {
                    let descriptor = *parts.get(1).unwrap();
                    let mut code = hex::decode(descriptor).unwrap();
                    let ret = BIND_PROTOCOL(code.as_ptr(), descriptor.len() as u32); // todo: check if null termination required
                    println!("bind_protocol|{}", ret);
                }
                "change_com_line" => {
                    let number = parts.get(1).unwrap();
                    let value = number.parse::<u32>().unwrap();
                    let ret = CHANGE_COM_LINE(value);
                    println!("change_com_line|{}", ret);
                }
                "write_and_read" => {
                    let ecuDescriptor = *parts.get(1).unwrap();
                    let mut ecuCode = hex::decode(ecuDescriptor).unwrap();

                    let inBuffer = *parts.get(2).unwrap();
                    let mut inCode =  hex::decode(inBuffer).unwrap();

                    let timeout = parts.get(3).unwrap();
                    let timeoutValue = timeout.parse::<u32>().unwrap();

                    let ptr = read_buffer.as_mut_ptr() as *mut i8;
                    let changed = WRITE_AND_READ(ecuCode.as_ptr(), 4, inCode.as_ptr(), inCode.len() as u32, ptr, BUFFER_SIZE as u32, timeoutValue);

                    if changed > 0 {
                        let mut output = String::new();
                        for i in 0..changed {
                            let formatted = format!("{:02X}", read_buffer.get(i as usize).unwrap());
                            output.push_str(formatted.as_str());
                        }
                        println!("write_and_read|{}|{}", changed, output)
                    } else {
                        println!("write_and_read|{}", changed)
                    }
                }
                &_ => {

                }
            }
        }
    }
}
