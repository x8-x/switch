use std::env;
use std::os::raw::c_char;

fn main() {
    println!("Hello, Rust DPDK!");
    let mut args: Vec<_> = env::args().map(|s| s.as_ptr() as *mut c_char).collect();
    unsafe {
        let ret = dpdk_sys::rte_eal_init(args.len() as i32, args.as_mut_ptr());
        if ret < 0 {
            panic!("Cannot init EAL\n");
        }
    }
}
