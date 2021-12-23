// extern crate backtrace;
extern crate iot_gateway;

fn main() {
    let a = u16::from_be_bytes([0xffu8, 0xffu8]) as f64;
    println!("{:?}", a);

    let before = 17.69708280254777;
    let after = f64::trunc(before  * 100.0) / 100.0; // or f32::trunc
    println!("{}", after);
}
