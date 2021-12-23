use async_std::fs::File;
use async_std::io::ReadExt;
use chrono::Utc;
use futures_util::AsyncWriteExt;
use hmacsha1::hmac_sha1;
use log::{debug};
use log4rs;
use sha1::Sha1;
use surf;
use surf::Body;
use urlencoding::encode;

#[async_std::test]
async fn put_image() -> surf::Result<()> {
    log4rs::init_file("config/log4rs.yaml", Default::default()).unwrap();
    //https://cos5.cloud.tencent.com/static/cos-sign/
    //https://cloud.tencent.com/document/product/436/7778#.E5.87.86.E5.A4.87.E5.B7.A5.E4.BD.9C
    let host = "middle-test-1300019243.cos.ap-chengdu.myqcloud.com";
    let uri_path_name =
        "/2021-03-29/00155DA00F36_rust_support/7250e534-a3fd-4ea8-9809-9748f410080f.png";
    let url = format!("https://{}{}", host, uri_path_name);
    let url = url.as_str();
    let mut file =
        File::open("/home/pi/pic/2021-03-29/7250e534-a3fd-4ea8-9809-9748f410080f.png").await?;
    let ref mut content_buf = Vec::new();
    let n = file.read_to_end(content_buf).await;
    //let md5 = hex::encode(md5::compute(&content_buf).as_ref());
    let length = content_buf.len().to_string();
    let length = length.as_str();
    println!("content_buf.len={} {}", length, n.unwrap());
    let md5 = base64::encode(md5::compute(&content_buf).as_ref());
    let now = Utc::now();
    let seconds_start = now.timestamp();
    let seconds_end = seconds_start + 60 * 60;
    let key_time = "1617001726;1617005326";
    // let key_time = format!("{};{}", seconds_start, seconds_end);
    // let gmt = now.format("%a, %d %b %Y %H:%M:%S GMT").to_string();
    let gmt = "Mon, 29 Mar 2021 07:08:46 GMT";
    let secret_key = "qQkrjLiu3TeZDQN3UheL5c3GFxkQtKuP";
    //let sign_key = String::from_utf8(hmac_sha1(secret_key.as_bytes(), key_time.as_bytes()).to_vec()).unwrap();
    let sign_key = hex::encode(hmac_sha1(secret_key.as_bytes(), key_time.as_bytes()));

    let secret_id = "AKIDpTWLwIBrnVx9aojY7SXF8l9VKM8es9LH";

    println!("{};{}", seconds_start, seconds_end);
    println!("sign_key={:?}", sign_key);
    let header_list = "Content-Length;Content-MD5;Content-Type;Date;Host".to_lowercase();
    //let __format = "Content-Length={}&Content-MD5={}&Content-Type={}&Date={}&Host={}".to_lowercase().as_str();
    let http_headers = format!(
        "content-length={}&content-md5={}&content-type={}&date={}&host={}",
        encode(length),
        encode(&md5),
        encode("image/jpeg"),
        encode(&gmt),
        encode(host)
    );
    println!(
        "header_list:{}           \nhttp_headers:{}",
        header_list, http_headers
    );

    let http_string = format!("put\n{}\n\n{}\n", uri_path_name, http_headers);

    //put\n/netgate/main/main_proc/1df22020-05-01125506.png\n\nHost: middle-test-1300019243.cos.ap-chengdu.myqcloud.com\nDate: Sun, 24 Jan 2021 16:51:07 GMT\nContent-Type: Content Type\nContent-Length: Content Length\nContent-MD5: 000000000000000\n

    let tmp_sha1 = Sha1::from(http_string.clone());
    let string_to_sign = format!("sha1\n{}\n{}\n", key_time, tmp_sha1.hexdigest());

    println!(
        "sign_key={}\nHttpString={}\nStringToSign ={}",
        sign_key, http_string, string_to_sign
    );
    let signature = hex::encode(hmac_sha1(sign_key.as_bytes(), string_to_sign.as_bytes()));
    let authorization = format!(
        "q-sign-algorithm=sha1&q-ak={}&q-sign-time={}&q-key-time={}\
                        &q-header-list={}&q-url-param-list=&q-signature={}",
        secret_id, key_time, key_time, header_list, signature
    );
    println!("sign={}", authorization);

    let res = surf::put(url)
        .header("Content-Type", "image/jpeg")
        .header("Content-Length", length)
        .header("Date", gmt)
        .header("Host", host)
        .header("Content-MD5", md5)
        .header("Authorization", authorization)
        .body(Body::from_bytes(content_buf.to_vec()))
        .await?;

    println!("\n\n{:?}", res);
    Ok(())
}

#[async_std::test]
async fn get_image() -> Result<(), surf::Error> {
    log4rs::init_file("config/log4rs.yaml", Default::default()).unwrap();
    //let url = "https://middle-test-1300019243.cos.ap-chengdu.myqcloud.com/20210115/b827eb941205/ipcImages/1df22021-01-15111742.png";
    let url = "https://middle-test-1300019243.cos.ap-chengdu.myqcloud.com/netgate/main/main_proc";
    debug!("start to download file...");
    let req = surf::get(url).header("Connection", "close");
    let res = req.recv_bytes().await?;
    // assert_eq!(res.status(), surf::StatusCode::Ok);
    // let msg = res.body_bytes().await?;
    //let mut file = File::create("1df22020-05-01125506.png").await?;
    debug!("start to write file...");
    let mut file = File::create("main_proc").await?;
    // file.write_all(msg.as_slice()).await?;
    file.write_all(res.as_ref()).await?;
    Ok(())
}
#[async_std::test]
async fn get_param() -> surf::Result<()> {
    log4rs::init_file("config/log4rs.yaml", Default::default()).unwrap();
    //https://cos5.cloud.tencent.com/static/cos-sign/
    //https://cloud.tencent.com/document/product/436/7778#.E5.87.86.E5.A4.87.E5.B7.A5.E4.BD.9C

    // let url = "https://middle-test-1300019243.cos.ap-chengdu.myqcloud.com/netgate/main/main_proc";

    let host = "middle-test-1300019243.cos.ap-chengdu.myqcloud.com";
    let uri_path_name = "/netgate/main/main_proc";
    let now = Utc::now();
    let seconds_start = now.timestamp();
    let seconds_end = seconds_start + 60 * 60;
    let key_time = format!("{};{}", seconds_start, seconds_end);
    let gmt = now.format("%a, %d %b %Y %H:%M:%S GMT").to_string();
    let secret_key = "EDwPdeQxEG3atxwBBxTlnZ6QUJtDMHxO";
    let sign_key = hex::encode(hmac_sha1(secret_key.as_bytes(), key_time.as_bytes()));
    let secret_id = "AKIDMTM0PZW9anDOCl7AzZt1uYw60f5GDzUj";
    let header_list = "Date;Host".to_lowercase();
    let http_headers = format!("date={}&host={}", encode(&gmt), encode(host));
    let http_string = format!("get\n{}\n\n{}\n", uri_path_name, http_headers);
    let tmp_sha1 = Sha1::from(http_string.clone());
    let string_to_sign = format!("sha1\n{}\n{}\n", key_time, tmp_sha1.hexdigest());
    let signature = hex::encode(hmac_sha1(sign_key.as_bytes(), string_to_sign.as_bytes()));
    let authorization = format!(
        "q-sign-algorithm=sha1&q-ak={}&q-sign-time={}&q-key-time={}\
                        &q-header-list={}&q-url-param-list=&q-signature={}",
        secret_id, key_time, key_time, header_list, signature
    );
    debug!(
        "key_time=\n{}, sign_key=\n{}, HttpString =\n{}, http_headers=\n{}StringToSign =\n{}, sign=\n{}",
        key_time, sign_key, http_string, http_headers, string_to_sign, authorization
    );

    let url = format!("https://{}{}", host, uri_path_name);
    let url = url.as_str();
    println!("url={}", url);
    let mut res = surf::get(url)
        .header("Date", gmt)
        .header("Host", host)
        .header("Authorization", authorization)
        .send()
        // .body(Body::from_bytes(content_buf.to_vec()))
        .await?;
    assert_eq!(res.status(), surf::StatusCode::Ok);
    debug!("{:?}", res);
    let msg = res.body_bytes().await?;
    //let mut file = File::create("1df22020-05-01125506.png").await?;
    debug!("start to write file...");
    let mut file = File::create("main_proc").await?;
    // file.write_all(msg.as_slice()).await?;
    file.write_all(msg.as_ref()).await?;

    Ok(())
}

#[test]
fn regex_test() {
    use regex::Regex;
    let s = "/clientid/req/server/serverId/ssh";
    let re = Regex::new(r"^/([^/]+)/([^/]+)/([^/]+)/([^/]+)/([^/]+)$").unwrap();
    assert!(re.is_match("/clientid/req/server/serverId/ssh"));

    re.captures_iter(&s);
    for i in re.captures_iter(s) {
        for j in 0..i.len() {
            println!("group {} : {}", j, &i[j]);
        }
        println!("{:?}", i.get(1));
        println!("{:?}", i.get(2));
    }
}
