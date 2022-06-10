use std::env;
use std::ptr::null_mut;
use std::os::raw::c_char;
use std::ffi::c_void;

extern "C" fn lcore_hello(_: *mut c_void) -> i32 {
    unsafe {
        print!("hello from core {}\n", dpdk_sys::rte_lcore_id());
    }
    0
}

fn main() {
    println!("Hello, Rust DPDK!");
    let mut args: Vec<_> = env::args().map(|s| s.as_ptr() as *mut c_char).collect();

    unsafe {
        let ret = dpdk_sys::rte_eal_init(args.len() as i32, args.as_mut_ptr());
        if ret < 0 {
            panic!("Cannot init EAL\n");
        }

        let mut lcore_id: u32 = dpdk_sys::rte_get_next_lcore(u32::MIN, 1, 0);
        while lcore_id < dpdk_sys::RTE_MAX_LCORE {
            dpdk_sys::rte_eal_remote_launch(Some(lcore_hello), null_mut(), lcore_id);
            lcore_id = dpdk_sys::rte_get_next_lcore(lcore_id, 1, 0);
        }

        lcore_hello(null_mut());
        dpdk_sys::rte_eal_mp_wait_lcore();
        dpdk_sys::rte_eal_cleanup();
    }
}
