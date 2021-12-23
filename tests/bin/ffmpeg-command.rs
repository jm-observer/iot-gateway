use std::io;
use std::process::Command;
use std::time::Duration;

///
/// ffmpeg  -r 25 -t 60 -i "rtsp://admin:@192.168.50.25:554" -vcodec libx264 -r 25
/// -tune zerolatency -preset ultrafast -b:v 400k -f flv
/// rtmp://148.70.132.77:1935/live/b827eb4dabbe0457020200209230404
///
#[async_std::main]
async fn main() -> io::Result<()> {
    let mut a = Command::new("ffmpeg");

    a.arg("-r")
        .arg("25")
        .arg("-t")
        .arg("60")
        .arg("-i")
        .arg("rtsp://admin:@192.168.50.25:554")
        .arg("-vcodec")
        .arg("libx264")
        .arg("-tune")
        .arg("zerolatency")
        .arg("-preset")
        .arg("ultrafast")
        .arg("-b:v")
        .arg("400k")
        .arg("-f")
        .arg("flv")
        .arg("-tune")
        .arg("zerolatency")
        .arg("rtmp://148.70.132.77:1935/live/b827eb4dabbe0457020200209230404");
    println!("{:?}", a);
    let mut child = a.spawn()?;
    println!("Command spawned");
    let tmp = async_std::task::spawn(async move {
        async_std::task::sleep(Duration::from_secs(10)).await;
    });
    // async_std::task::sleep(Duration::from_secs(60)).await;
    tmp.await;
    println!("Proc end");
    child.kill()?;
    Ok(())
}
