mod common;
use common::*;
use sha1::Sha1;
use std::fs::{File};
use std::io::prelude::*;
use std::net::TcpStream;
use std::path::Path;
use std::time::Duration;

#[async_std::test]
async fn update() {
    init_log_config();
    let local = Path::new("./target/armv7-unknown-linux-gnueabihf/release/iot_gateway");
    let mut sha: String = String::from("");
    let mut data: Vec<u8>;
    if let Ok(mut file) = File::open(local) {
        // let meta = file.metadata().unwrap();
        data = Vec::new();
        file.read_to_end(&mut data).unwrap();
        let sha1 = Sha1::from(data);
        sha = sha1.digest().to_string();
    }
    upload();
    loop {
        data = Vec::new();
        async_std::task::sleep(Duration::from_secs(2)).await;
        if let Ok(mut file) = File::open(local) {
            file.read_to_end(&mut data).unwrap();
            let sha_tmp = Sha1::from(data).digest().to_string();
            if sha_tmp == sha {
                debug!("no change!");
            } else {
                sha = sha_tmp;
                debug!("changed!");
                upload();
            }
        }
    }
}

fn upload() {
    let local = Path::new("./target/armv7-unknown-linux-gnueabihf/release/iot_gateway");
    let mut file = File::open(local).unwrap();
    // let meta = file.metadata().unwrap();
    let mut data = Vec::new();
    file.read_to_end(&mut data).unwrap();
    let sess = authed_session();
    let sftp = sess.sftp().unwrap();
    // let path = Path::new("/opt/iot/iot_gateway_tmp");
    // sftp.unlink(path);
    // sftp.rename(
    //     Path::new("/opt/iot/iot_gateway"),
    //     Path::new("/opt/iot/iot_gateway_tmp"),
    //     None,
    // );
    sftp.create(Path::new("/opt/iot/iot_gateway_tmp"))
        .unwrap()
        .write_all(data.as_ref())
        .unwrap();
    // sftp.rename(
    //     Path::new("/opt/iot/iot_gateway_tmp"),
    //     Path::new("/opt/iot/iot_gateway"),
    //     None,
    // );
}

pub fn socket() -> TcpStream {
    TcpStream::connect(&test_addr()).unwrap()
}

pub fn test_addr() -> String {
    "192.168.0.100:22".to_string()
}

pub fn authed_session() -> ssh2::Session {
    // let user = "pi";
    let socket = socket();
    let mut sess = ssh2::Session::new().unwrap();
    sess.set_tcp_stream(socket);
    sess.handshake().unwrap();
    assert!(!sess.authenticated());

    sess.userauth_password("pi", "admin").unwrap();
    assert!(sess.authenticated());
    sess
}

#[test]
fn translate_to_pi() {
    let sess = authed_session();
    let sftp = sess.sftp().unwrap();
    let path = Path::new("/opt/iot/iot_client");
    //sftp.unlink(path).unwrap();
    sftp.create(path).unwrap().write_all(b"foo11").unwrap();
    let local = Path::new("./target/armv7-unknown-linux-gnueabihf/release/iot_gateway");
    let mut file = File::open(local).unwrap();
    // let meta = file.metadata().unwrap();
    let mut data = Vec::new();
    file.read_to_end(&mut data).unwrap();
    let sha = Sha1::from(data);
    println!("{:?}", sha.digest().to_string());
    // let file = read_dir(local).unwrap();
    // for path in file {
    //     println!("Name: {}", path.unwrap().path().display())
    // }
}
#[test]
fn ops() {
    let sess = authed_session();
    let sftp = sess.sftp().unwrap();
    let path = Path::new("./foo");
    sftp.opendir(path).unwrap();
    let mut foo = sftp.open(path.join("bar").as_path()).unwrap();
    sftp.mkdir(path.join("bar2").as_path(), 0o755).unwrap();
    sftp.rmdir(path.join("bar2").as_path()).unwrap();

    sftp.create(path.join("foo5").as_path())
        .unwrap()
        .write_all(b"foo")
        .unwrap();
    let mut v = Vec::new();

    assert_eq!(sftp.stat(path.join("foo").as_path()).unwrap().size, Some(0));
    v.truncate(0);
    foo.read_to_end(&mut v).unwrap();
    // assert_eq!(v, Vec::new());

    sftp.symlink(path.join("foo").as_path(), path.join("foo2").as_path())
        .unwrap();
    let readlink = sftp.readlink(path.join("foo2").as_path()).unwrap();
    assert!(readlink == path.join("foo"));
    let realpath = sftp.realpath(path.join("foo2").as_path()).unwrap();
    assert_eq!(realpath, path.join("foo").canonicalize().unwrap());

    let files = sftp.readdir(path).unwrap();
    assert_eq!(files.len(), 4);
}
