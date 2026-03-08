use std::fs;
use std::os::unix::net::UnixDatagram;
use std::path::Path;
use std::time::Instant;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct OB {
    bids: [f32; 20],
    asks: [f32; 20],
}

impl OB {
    fn as_bytes(&self) -> &[u8] {
        unsafe {
            ::core::slice::from_raw_parts(
                (self as *const OB) as *const u8,
                ::core::mem::size_of::<OB>(),
            )
        }
    }
    fn from_bytes(bytes: &[u8]) -> Self {
        unsafe {
            assert_eq!(bytes.len(), std::mem::size_of::<Self>());
            *(bytes.as_ptr() as *const Self)
        }
    }
}

fn main() -> std::io::Result<()> {
    let salt = "meow";
    let mut orderbook = OB {
        bids: [5.2f32; 20],
        asks: [3.2f32; 20],
    };
    orderbook.asks[10] = 9.1f32;
    let path_rust_rx = format!("/dev/shm/rust_{}.sock", salt);
    let path_ocaml_rx = format!("/dev/shm/ocaml_{}.sock", salt);

    if Path::new(&path_rust_rx).exists() {
        fs::remove_file(&path_rust_rx)?;
    }

    let receiver = UnixDatagram::bind(&path_rust_rx)?;
    receiver.connect(&path_ocaml_rx)?;
    receiver.set_nonblocking(false)?;
    let mut latencies = Vec::with_capacity(1_000_000);

    let mut buf = [0; std::mem::size_of::<OB>()];
    for _ in 0..1000 {
        let start = Instant::now();

        receiver.send(orderbook.as_bytes())?;

        let (count, _) = receiver.recv_from(&mut buf)?;
        let ob = OB::from_bytes(&buf);
    }
    for _ in 0..1_000_000 {
        let start = Instant::now();

        receiver.send(orderbook.as_bytes())?;

        let _ = receiver.recv(&mut buf)?;
        let ob = OB::from_bytes(&buf);

        latencies.push(start.elapsed().as_nanos());
    }

    let avg = latencies.iter().sum::<u128>() / latencies.len() as u128;
    println!("Average Round-trip Latency: {}ns", avg);
    Ok(())
}
