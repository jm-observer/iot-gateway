// 实现腾讯云对象存储的Put Object功能：https://cloud.tencent.com/document/product/436/7749

use crate::*;
use async_std::fs::File;
use chrono::Utc;
use futures::AsyncReadExt;
use futures_util::AsyncWriteExt;
use hmacsha1::hmac_sha1;
use mio_httpc::CallBuilder;
use sha1::Sha1;
use urlencoding::encode;
// use http_client::h1::H1Client;
// use http_client::http_types::Url;
// use http_client::{http_types, HttpClient, Request};

/// 将腾讯云的cos文件（key）下载在本地路径（file_path），命名为file_name
///
/// # Examples
/// Basic usage:
/// get_image("/opt/iot", "iot_gateway", "/20210115/b827eb941205/ipcImages/1df22021-01-15111742.png")
///
// pub async fn get_image(file_path: &str, file_name: &str, key: &str) -> Result<()> {
//     //let url = "https://middle-test-1300019243.cos.ap-chengdu.myqcloud.com/20210115/b827eb941205/ipcImages/1df22021-01-15111742.png";
//     init_dir(&file_path)?;
//     let url = format!(
//         "https://middle-test-1300019243.cos.ap-chengdu.myqcloud.com{}",
//         key
//     );
//     let mut file = file_path.to_string();
//     file.push_str("/");
//     file.push_str(file_name);
//
//     let req = Request::new(http_types::Method::Get, Url::parse(&url).unwrap());
//     let client = H1Client::new();
//     match client.send(req).await {
//         Ok(mut res) => {
//             // assert_eq!(res.status(), http_types::StatusCode::Ok);
//             if let Ok(msg) = res.body_bytes().await {
//                 match File::create(file).await {
//                     Ok(mut file) => {
//                         if let Err(res) = file.write_all(msg.as_slice()).await {
//                             return Err(fail(format!("程序下载失败：{:?}", res)));
//                         } else {
//                             debug!("下载成功！");
//                         }
//                     }
//                     Err(e) => {
//                         return Err(fail(format!("程序下载失败：{:?}", e)));
//                     }
//                 }
//             } else {
//                 return Err(fail_from_str("程序下载失败：读取数据失败"));
//             }
//         }
//         Err(e) => {
//             return Err(fail(format!("程序下载失败：{:?}", e)));
//         }
//     }
//     Ok(())
// }

/// 将腾讯云的cos文件（key）下载在本地路径（file_path），命名为file_name
///
/// # Examples
/// Basic usage:
/// get_image("/opt/iot", "iot_gateway", "/20210115/b827eb941205/ipcImages/1df22021-01-15111742.png")
///
pub async fn get_image(file_path: &str, file_name: &str, key: &str) -> Result<()> {
    //let url = "https://middle-test-1300019243.cos.ap-chengdu.myqcloud.com/20210115/b827eb941205/ipcImages/1df22021-01-15111742.png";
    init_dir(&file_path)?;
    let url = format!(
        "https://middle-test-1300019243.cos.ap-chengdu.myqcloud.com{}",
        key
    );
    let mut file = file_path.to_string();
    file.push_str("/");
    file.push_str(file_name);
    let (_, body) = CallBuilder::get()
        .timeout_ms(5000)
        .max_response(60000000)
        .url(&url)
        .unwrap()
        .exec()
        .unwrap();
    let mut file = File::create(file).await?;
    // let mut buffer = Vec::new();
    // response.body_mut().read_to_end(&mut buffer);
    if let Err(res) = file.write_all(&body).await {
        return fail(format!("程序下载失败：{:?}", res));
    } else {
        debug!("下载成功！");
    }
    Ok(())
}

fn init_dir(file_path: &str) -> Result<()> {
    use std::fs::*;
    if let Err(_) = read_dir(file_path) {
        if let Err(res) = create_dir_all(file_path) {
            return fail(format!("{:?}", res));
        }
    }
    Ok(())
}

pub async fn get_sign_key_and_gmt(secret_key: &str) -> (String, String, String) {
    let now = Utc::now();
    let seconds_start = now.timestamp();
    let seconds_end = seconds_start + 60 * 60;
    let key_time = format!("{};{}", seconds_start, seconds_end);
    let gmt = now.format("%a, %d %b %Y %H:%M:%S GMT").to_string();
    let sign_key = hex::encode(hmac_sha1(secret_key.as_bytes(), key_time.as_bytes()));
    (gmt, sign_key, key_time)
}

pub async fn put_image(
    file_path: &str,
    host: &str,
    url: &str,
    secret_id: &str,
    gmt: String,
    sign_key: &str,
    key_time: &str,
    uri_path_name: &str,
) -> Result<String> {
    let mut file = File::open(file_path).await?;
    let ref mut content_buf = Vec::new();
    file.read_to_end(content_buf).await?;
    let length = content_buf.len().to_string();
    let length = length.as_str();
    let md5 = base64::encode(md5::compute(&content_buf).as_ref());
    let header_list = "content-length;content-md5;content-type;date;host".to_lowercase();
    let string_to_sign = format!(
        "sha1\n{}\n{}\n",
        key_time,
        Sha1::from(format!(
            "put\n{}\n\ncontent-length={}&content-md5={}&content-type={}&date={}&host={}\n",
            uri_path_name,
            encode(length),
            encode(&md5),
            encode("image/jpeg"),
            encode(&gmt),
            encode(&host)
        ))
        .hexdigest()
    );
    let signature = hex::encode(hmac_sha1(sign_key.as_bytes(), string_to_sign.as_bytes()));
    let authorization = format!(
        "q-sign-algorithm=sha1&q-ak={}&q-sign-time={}&q-key-time={}\
                        &q-header-list={}&q-url-param-list=&q-signature={}",
        secret_id, key_time, key_time, header_list, signature
    );
    let (res, _) = CallBuilder::put(content_buf.to_vec())
        .url(&url)?
        .header("Content-Type", "image/jpeg")
        .header("Content-Length", length)
        .header("Date", &gmt)
        .header("Host", &host)
        .header("Content-MD5", &md5)
        .header("Authorization", &authorization)
        .exec()?;
    Ok(res.status.to_string())
}

#[cfg(test)]
mod test {
    use crate::*;

    #[async_std::test]
    async fn get_image_test() -> Result<()> {
        get_image(
            "./target/tmp/tmp/tmp",
            "main_proc",
            "/netgate/main/main_proc",
        )
        .await?;
        Ok(())
    }
}
