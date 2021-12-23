use json_minimal::Json;
use std::net::UdpSocket;

#[test]
fn str_from_utf8_test() {
    use std::str;
    println!("str_from_utf8_test");
    let vec_u8 = vec![255u8];
    println!("{:b}", 255u8);
    if let Ok(abc) = str::from_utf8(&vec_u8) {
        println!("{}", abc);
    }
    //will pinic
    // str::from_utf8(&vec_u8).unwrap();
    // println!("end");
}

#[test]
pub fn get_ip() {
    if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
        if let Ok(()) = socket.connect("8.8.8.8:80") {
            if let Ok(addr) = socket.local_addr() {
                println!("{}", addr.ip().to_string());
            }
        }
        // if let Ok(addr) = socket.local_addr() {
        //     println!("{}", addr.ip().to_string());
        // }
    };
    println!("error");
}

#[test]
pub fn get_cargo_pkg_version() {
    println!(env!("CARGO_PKG_VERSION"));
}

#[test]
fn json_test() {
    let msg = Json::parse("{}".as_bytes()).unwrap();
    println!("{:?}", msg);
}

#[test]
fn serial_test() {
    // let msg = [0x01u8, 0x03, 0x00, 0x0B, 0x00, 0x01, 0xF5, 0xC8];
    // let msg = [0x01u8, 0x03, 0x00, 0x00, 0x00, 0x02, 0xC4, 0x0B];
    //01 03 02 27 08 a2 72
    let res = [0x01u8, 0x03, 0x02, 0x27, 0x11, 0x02, 0xC4];

    // let res = 0x2708;
    let mut tmp = [0u8; 2];

    tmp.copy_from_slice(&res[3..5]);
    // println!("res0={:x?}", tmp);
    println!("res0={:.2?}", u16::from_be_bytes([0x27, 0x11]) as f32 * 0.1);
    println!("res0={:?}", u16::from_be_bytes(tmp));
    // println!("res0={:?}", i8Toi32(tmp));
    // println!("res0={:?}", i8Toi32([01u8, 00u8]));
    // let res0 = res[3..4] as f32 * 0.1f32;
    // println!("res0={}", res0);
    // let msg = [0x01u8, 0x03, 0x00, 0x08, 0x00, 0x04, 0xC5, 0xCB];
    // let msg = [0x01u8, 0x03, 0x00, 0x0B, 0x00, 0x01, 0xF5, 0xC8];
    // println!("{:?}", msg);
}

// fn i8Toi32(v: [u8; 2]) -> i16 {
//     unsafe {
//         let i32Ptr: *const i16 = v.as_ptr() as *const i16;
//         return *i32Ptr;
//     }
// }

#[test]
fn format_test() {
    // Hello {arg 0 ("x")} is {arg 1 (0.01) with precision specified inline (5)}
    println!("Hello {0} is {1:.5}", "x", 0.01);

    // Hello {arg 1 ("x")} is {arg 2 (0.01) with precision specified in arg 0 (5)}
    println!("Hello {1} is {2:.0$}", 5, "x", 0.01);

    // Hello {arg 0 ("x")} is {arg 2 (0.01) with precision specified in arg 1 (5)}
    println!("Hello {0} is {2:.1$}", "x", 5, 0.01);

    // Hello {next arg ("x")} is {second of next two args (0.01) with precision
    //                          specified in first of next two args (5)}
    println!("Hello {} is {:.*}", "x", 5, 0.01);

    // Hello {next arg ("x")} is {arg 2 (0.01) with precision
    //                          specified in its predecessor (5)}
    println!("Hello {} is {1:.3}", "x", 0.0123421);

    // Hello {next arg ("x")} is {arg "number" (0.01) with precision specified
    //                          in arg "prec" (5)}
    println!(
        "Hello {} is {number:.prec$}",
        "x",
        prec = 2,
        number = 0.01601
    );
}
