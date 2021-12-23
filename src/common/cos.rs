/// 腾讯云的对象存储（cos: https://cloud.tencent.com/product/cos https://cloud.tencent.com/document/product/436/7778）的相关实现
use crate::*;

pub struct CosTask {
    pub is_wait: bool,
    pub file_path: String,
    pub file_uri_path: String,
    pub send_res: Sender<Result<String>>,
}

pub async fn start_cos_client(global: Arc<Global>) -> Result<()> {
    let (secret_id, secret_key, host) = global.toml_config.read().await.get_cos_config()?;
    task::spawn(async move {
        cos_client(global, secret_id, secret_key, host).await;
    });
    Ok(())
}

async fn cos_client(global: Arc<Global>, secret_id: String, secret_key: String, host: String) {
    let rec_channel = global.rec_cos.clone();
    loop {
        match rec_channel.recv().await {
            Ok(mut ct) => {
                let host_tmp = host.clone();
                let secret_key_tmp = secret_key.clone();
                let secret_id_tmp = secret_id.clone();
                task::spawn(async move {
                    let url = format!("https://{}{}", host_tmp, ct.file_uri_path);
                    if ct.is_wait == false {
                        if let Err(e) = ct.send_res.send(Ok(url.clone())).await {
                            warn!("{:?}", e);
                            ct.is_wait = true;
                        }
                    }
                    let (gmt, sign_key, key_time) = get_sign_key_and_gmt(&secret_key_tmp).await;
                    let mut res = put_image(
                        ct.file_path.as_str(),
                        host_tmp.as_str(),
                        url.as_str(),
                        secret_id_tmp.as_str(),
                        gmt,
                        sign_key.as_str(),
                        key_time.as_str(),
                        ct.file_uri_path.as_str(),
                    )
                    .await;
                    if res.is_err() {
                        warn!("{:?}", res.as_ref().unwrap_err());
                    } else {
                        res = Ok(url);
                    }
                    if ct.is_wait {
                        if let Err(e) = ct.send_res.send(res).await {
                            warn!("{:?}", e);
                        }
                    }
                });
            }
            Err(e) => {
                warn!("{:?}", e);
            }
        }
    }
}
