#[ocaml::func]
#[ocaml::sig("unit -> int")]
pub fn hello_world() -> ocaml::Int {
    println!("hello,  world!");
    let x = 21;
    return x;
}

#[derive(ocaml::ToValue, ocaml::FromValue)]
#[ocaml::sig("{bids: int array; asks: float array}")]
pub struct Ob {
    bids: Vec<f64>,
    asks: Vec<f64>,
}

#[ocaml::func]
#[ocaml::sig("unit -> ob")]
pub fn get_orderbook() -> Ob {
    let mut ob = Ob {
        bids: vec![3.2; 20],
        asks: vec![2.2; 20],
    };
    ob.bids[10] = 1.2;
    ob
}

#[repr(C, packed)]
#[derive(Debug)]
pub struct Order {
    pub price: i64,  // Bytes 0-7
    pub qty: i64,    // Bytes 8-15
    pub side: u8,    // Byte 16
    pub type_id: u8, // Byte 17
}

impl Order {
    fn from_bytes(raw_ptr: *mut i64, start: usize) -> Self {
        unsafe {
            Self {
                price: std::ptr::read_volatile(raw_ptr.add(start)),
                qty: std::ptr::read_volatile(raw_ptr.add(start + 8)),
                side: std::ptr::read_volatile(raw_ptr.add(start + 16)) as u8,
                type_id: std::ptr::read_volatile(raw_ptr.add(start + 17)) as u8,
            }
        }
    }
}

#[ocaml::func]
#[ocaml::sig("(int64, Bigarray.int64_elt, Bigarray.c_layout) Bigarray.Array1.t -> unit")]
pub unsafe fn spawn_worker(mut ba: ocaml::bigarray::Array1<i64>) {
    let slice = ba.data_mut();
    let ptr: *mut i64 = slice.as_mut_ptr();
    let len = slice.len();
    let addr = ptr as usize;
    std::thread::spawn(move || {
        // 3. Reconstruct the raw pointer
        let raw_ptr = addr as *mut i64;

        // 4. Create a mutable slice from parts (Unsafe, but required for SHM)
        let hot_slice = unsafe { std::slice::from_raw_parts_mut(raw_ptr, len) };
        /*
        loop {
            for i in 0..32 {
                let val = unsafe { std::ptr::read_volatile(raw_ptr.add(i)) };
                println!("Rust saw value {} at index {}", val, i);
                //                unsafe { std::ptr::write_volatile(raw_ptr.add(i), 0) }; // Clear it
            }
            std::hint::spin_loop();
         }*/
        //24 is acting as our switch to know if we need to execute
        loop {
            let cmd = std::ptr::read_volatile(raw_ptr.add(18));
            /*
            println!("slot 0: {:}", cmd);
            */
            if cmd == 0 {
                /*
                    println!("slot 0 detected by rust not to be 0!");
                println!("Total size: {}", std::mem::size_of::<Order>());
                println!("Price offset: {}", offset_of!(Order, price));
                println!("Qty offset:   {}", offset_of!(Order, qty));
                println!("Side offset:  {}", offset_of!(Order, side));
                println!("type_id offset:  {}", offset_of!(Order, type_id));
                 */
                let _ = random_work(cmd);
                let order = Order::from_bytes(raw_ptr, 0);
                //                dbg!(&order);

                //std::thread::sleep(std::time::Duration::from_secs(2));
                std::ptr::write_volatile(raw_ptr.add(18), order.price * 3);
                //clear the order
                std::ptr::write_volatile(raw_ptr.add(0), 0);
                std::ptr::write_volatile(raw_ptr.add(8), 0);
                std::ptr::write_volatile(raw_ptr.add(16), 0);
                std::ptr::write_volatile(raw_ptr.add(17), 0);
                /*
                    println!("rust slot 16: {:}", std::ptr::read_volatile(raw_ptr.add(0)));
                */
            }
            //            std::thread::sleep(std::time::Duration::from_secs(2));
            //            for i in 0..len {
            //              hot_slice[i] = 22;
            //        }
        }
    });
}
fn random_work(y: i64) -> i64 {
    let mut x = y * y;
    for i in 1..10000 {
        x *= i;
    }
    x
}
