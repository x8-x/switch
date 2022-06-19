use std::env;
use std::ptr::null_mut;
use std::os::raw::c_char;
use std::ffi::c_void;
use std::ffi::CString;

extern "C" fn lcore_hello(_: *mut c_void) -> i32 {
    unsafe {
        println!("hello from core {}", dpdk_sys::rte_lcore_id());
    }
    0
}


fn main() {
    println!("Hello,Rust DPDK!");
    let args: Vec<_> = env::args().map(|s| CString::new(s).unwrap()).collect();
    let mut cargs: Vec<_> = args.iter().map(|s| s.as_ptr() as *mut c_char).collect();

    unsafe {
        let ret = dpdk_sys::rte_eal_init(cargs.len() as i32, cargs.as_mut_ptr());
        if ret < 0 {
            panic!("Cannot init EAL\n");
        }

        // vfioドライバ呼び出すのに必要、、、
        dpdk_sys::output_test_log();

        let avail_port_num = dpdk_sys::rte_eth_dev_count_avail();
        if avail_port_num <= 0 {
            panic!("Cannot avail device\n");
        }

        // allocate pktmbuf
        let cstr_mbuf_pool = CString::new("mbuf_pool").unwrap();
        let mut buf = dpdk_sys::rte_pktmbuf_pool_create(
            cstr_mbuf_pool.as_ptr() as *mut c_char,
            8192,
            256,
            0,
            dpdk_sys::RTE_MBUF_DEFAULT_BUF_SIZE.try_into().unwrap(),
            dpdk_sys::rte_socket_id().try_into().unwrap()
        );


        // init port
        for i in 0..avail_port_num {
            let port_conf: dpdk_sys::rte_eth_conf = Default::default();
            if dpdk_sys::rte_eth_dev_configure(i, 1, 1, &port_conf as *const _) < 0 {
                panic!("Cannot configure device\n");
            }

            let dev_socket_id = dpdk_sys::rte_eth_dev_socket_id(i).try_into().unwrap();

            if dpdk_sys::rte_eth_rx_queue_setup(i, 0, 1024, dev_socket_id, null_mut(), buf) < 0 {
                panic!("Error rte_eth_rx_queue_setup\n");

            }

            if dpdk_sys::rte_eth_tx_queue_setup(i, 0, 1024, dev_socket_id, null_mut()) < 0 {
                panic!("Error rte_eth_tx_queue_setup\n");

            }

            if dpdk_sys::rte_eth_dev_start(i) < 0 {
                panic!("Error rte_eth_dev_start\n");

            }

            dpdk_sys::rte_eth_promiscuous_enable(i);
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
