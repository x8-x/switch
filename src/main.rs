mod fib;
use std::env;
use std::ptr::null_mut;
use std::os::raw::c_char;
use std::ffi::c_void;
use std::ffi::CString;
use fib::cache::*;
use fib::l1_cache::*;

extern "C" fn lcore_hello(_: *mut c_void) -> i32 {
    unsafe {
        println!("hello from core {}", dpdk_sys::rte_lcore_id());
    }
    0
}

// extern "C" fn lcore_test1(_: *mut c_void) -> i32 {
//     0
// }

// extern "C" fn lcore_test2(_: *mut c_void) -> i32 {
//     0
// }


fn main() {
    println!("Hello,Rust DPDK!");
    let args: Vec<_> = env::args().map(|s| CString::new(s).unwrap()).collect();
    let mut cargs: Vec<_> = args.iter().map(|s| s.as_ptr() as *mut c_char).collect();

    unsafe {
        let ret = dpdk_sys::rte_eal_init(cargs.len() as i32, cargs.as_mut_ptr());
        if ret < 0 {
            panic!("Cannot init EAL\n");
        }

        // 仮対応
        dpdk_sys::output_test_log();
        dpdk_sys::load_rte_eth_tap();

        let avail_port_num = dpdk_sys::rte_eth_dev_count_avail();
        if avail_port_num <= 0 {
            panic!("Cannot avail device\n");
        }
        println!("{}", avail_port_num);

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

        // start up worker
        let mut lcore_id: u32 = dpdk_sys::rte_get_next_lcore(u32::MIN, 1, 0);
        while lcore_id < dpdk_sys::RTE_MAX_LCORE {
            dpdk_sys::rte_eal_remote_launch(Some(lcore_hello), null_mut(), lcore_id);
            lcore_id = dpdk_sys::rte_get_next_lcore(lcore_id, 1, 0);
        }

        // main core process
        lcore_hello(null_mut());

        // L1 Cache
        let cstr_l1_cache = CString::new("l1_cache").unwrap();
        let l1_cache_mempool = dpdk_sys::rte_mempool_create(
            cstr_l1_cache.as_ptr() as *mut c_char,
            1023,
            1,
            0,
            0,
            None,
            null_mut(),
            None,
            null_mut(),
            0,
            0
        );

        // test cast raw -> struct
        // println!("data pointer {:p}", (*l1_cache_mempool).__bindgen_anon_1.pool_data);

        // let ce = std::mem::transmute::<*mut std::os::raw::c_void, *mut CacheElement>((*l1_cache_mempool).__bindgen_anon_1.pool_data);
        // (*ce).action_id = 5;
        // (*ce).tag = 100;
        // (*ce.offset(1)).action_id = 6;
        // println!("test action_id {}", (*ce).action_id);
        // println!("test tag {}", (*ce).tag);
        // println!("test action_id {}", (*ce.offset(1)).action_id);
        // (*ce.offset(1)).action_id = 8;
        // println!("test action_id {}", (*ce.offset(1)).action_id);

        // let l1_cache = L1Cache::new();
        // println!("{}", l1_cache.hash_table);




        // packet process
        let mut pkts: [*mut dpdk_sys::rte_mbuf; 32] = [null_mut(); 32];
        while true {
            let tap_rx = dpdk_sys::rte_eth_rx_burst(0, 0, pkts.as_ptr() as *mut *mut dpdk_sys::rte_mbuf, 32);
            if tap_rx <= 0 {
                continue;
            }
            println!("recv: {}", tap_rx);
            for i in 0..tap_rx {
                let pkt = std::mem::transmute::<*mut  std::os::raw::c_void, *mut u8>((*pkts[i as usize]).buf_addr);
                let len = (*pkts[i as usize]).data_len;
                let off = (*pkts[i as usize]).data_off;
                println!("{}: len{}, off{}", i, len, off);
                for j in off..len+off {
                    print!("{:x} ", *pkt.offset(j.try_into().unwrap()));
                }
                println!("");

                // print!("dst? {:x}:", *pkt);
                // print!("{:x}:", *pkt.offset(1));
                // print!("{:x}:", *pkt.offset(2));
                // print!("{:x}:", *pkt.offset(3));
                // print!("{:x}:", *pkt.offset(4));
                // println!("{:x}", *pkt.offset(5));
                // print!("src? {:x}:", *pkt.offset(6));
                // print!("{:x}:", *pkt.offset(7));
                // print!("{:x}:", *pkt.offset(8));
                // print!("{:x}:", *pkt.offset(9));
                // print!("{:x}:", *pkt.offset(10));
                // println!("{:x}", *pkt.offset(11));
                // print!("{:x}:", *pkt.offset(12));
                // print!("{:x}:", *pkt.offset(13));
                // print!("{:x}:", *pkt.offset(14));
                // print!("{:x}:", *pkt.offset(15));
                dpdk_sys::rte_pktmbuf_free(pkts[i as usize]);
            }
            // let tap_tx = dpdk_sys::rte_eth_tx_burst(1, 0, pkts.as_ptr() as *mut *mut dpdk_sys::rte_mbuf, tap_rx);
            // println!("send: {}", tap_tx);
        }

        dpdk_sys::rte_eal_mp_wait_lcore();
        dpdk_sys::rte_eal_cleanup();
    }
}
