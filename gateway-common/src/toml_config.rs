use crate::pub_use::*;
use crate::{get_index_group_str, IntoResult};
/// 配置文件的相关操作
use regex::Regex;
use std::fs::create_dir_all;
use toml_edit::{table, value, Document, Item, Value};
//初始化

//取值

//更新
#[derive(Debug)]
pub struct Toml {
    config: Document,
    path: String,
    reg: Regex,
}

const RTSP_REG: &str = r"^rtsp://[.\d]+(:?\d**/?.*)$";

impl Toml {
    ///
    /// 重新读取文件数据
    ///
    pub fn update_doc(&mut self) -> Result<()> {
        // debug!("开始更新config.toml...");
        let config = String::from_utf8(std::fs::read(self.path.as_str())?)?.parse::<Document>()?;
        self.config = config;
        debug!("更新config.toml成功！");
        Ok(())
    }
    pub fn init(path: &str) -> Result<Self> {
        let config = String::from_utf8(std::fs::read(path)?)?.parse::<Document>()?;
        Ok(Toml {
            config,
            path: path.to_string(),
            reg: Regex::new(RTSP_REG).unwrap(),
        })
    }
    ///
    /// 将内存数据重新写入文件中
    ///
    pub async fn update(&self) -> Result<()> {
        async_std::fs::write(&self.path, self.config.to_string()).await?;
        Ok(())
    }

    pub async fn add_ipc_config(
        &mut self,
        uuid: &str,
        stream_url: String,
        user: &str,
        password: &str,
        manufacturer: &str,
        width: u32,
        heigth: u32,
    ) -> Result<()> {
        debug!("add_ipc_config");
        if self.config["onvif"].is_none() {
            self.config["onvif"] = table();
            self.config["onvif"]["ipc"] = table();
        } else if self.config["onvif"]["ipc"].is_none() {
            self.config["onvif"]["ipc"] = table();
        }
        self.config["onvif"]["ipc"][uuid] = table();
        self.config["onvif"]["ipc"][uuid]["name"] = value(user);
        self.config["onvif"]["ipc"][uuid]["password"] = value(password);
        self.config["onvif"]["ipc"][uuid]["manufacturer"] = value(manufacturer);
        let cap = self
            .reg
            .captures(stream_url.as_str())
            .into_result_by_msg(format!("stream_url[{}]不规范！", stream_url))?;
        let sub_url = get_index_group_str(&cap, 1)?;
        self.config["onvif"]["ipc"][uuid]["stream_url"] = value(sub_url);
        self.config["onvif"]["ipc"][uuid]["width"] = value(width.to_string());
        self.config["onvif"]["ipc"][uuid]["height"] = value(heigth.to_string());
        self.update().await
    }
    pub fn is_exist_ipc_config(&self, uuid: &str) -> Result<bool> {
        Ok(!self.config["onvif"]["ipc"][uuid].is_none())
    }
    pub fn get_ipcs(&self) -> Result<Vec<String>> {
        // println!("{:?}", tmp.iter());
        if self.config["onvif"]["ipc"].as_table().is_none() {
            return Ok(Vec::with_capacity(1));
        }
        let tmp = self.config["onvif"]["ipc"]
            .as_table()
            .into_result_by_msg("配置节点[onvif.ipc]异常")?;
        // let ipcs = self.config["ipc"].as_array_of_tables().unwrap();
        let mut ipcs = Vec::<String>::with_capacity(15);
        for ipc in tmp.iter() {
            ipcs.push(ipc.0.to_string());
        }
        Ok(ipcs)
    }
    pub fn get_ipcs_data(&self) -> Result<Vec<(String, String, String, String, String)>> {
        if self.config["onvif"]["ipc"].as_table().is_none() {
            return Ok(Vec::with_capacity(1));
        }
        let tmp = self.config["onvif"]["ipc"]
            .as_table()
            .into_result_by_msg("配置节点[onvif.ipc]异常")?;
        // let ipcs = self.config["ipc"].as_array_of_tables().unwrap();
        let mut ipcs = Vec::<(String, String, String, String, String)>::with_capacity(15);
        for (uuid, _) in tmp.iter() {
            ipcs.push(self.get_ipc_data(uuid)?);
        }
        Ok(ipcs)
    }

    pub fn get_ipc_data(&self, uuid: &str) -> Result<(String, String, String, String, String)> {
        let ipc = self.config["onvif"]["ipc"][uuid].as_table();
        if ipc.is_none() {
            bail!("该ipc未鉴权");
        }
        let ipc = ipc.unwrap();
        Ok((
            uuid.to_string(),
            self._get_item_string(ipc.get("name").into_result_by_msg("该ipc配置信息缺失")?)?,
            self._get_item_string(
                ipc.get("password")
                    .into_result_by_msg("该ipc配置信息缺失")?,
            )?,
            self._get_item_string(
                ipc.get("stream_url")
                    .into_result_by_msg("该ipc配置信息缺失")?,
            )?,
            self._get_item_string(
                ipc.get("manufacturer")
                    .into_result_by_msg("该ipc配置信息缺失")?,
            )?,
        ))
    }
    pub fn remove_ipc_config(&mut self, uuid: &str) -> Result<bool> {
        if self.config["onvif"]["ipc"][uuid].is_none() {
            bail!("该uuid不存在，无法删除");
        }
        if let Some(root) = self.config["onvif"].as_table_mut() {
            let servers = root.entry("ipc");
            if let Some(servers) = servers.as_table_mut() {
                return Ok(servers.remove(uuid).is_some());
            }
        }
        Ok(true)
    }
    pub async fn update_ipc_auth(
        &mut self,
        uuid: &str,
        user: String,
        password: String,
        // stream_url: String,
    ) -> Result<()> {
        if self.config["onvif"]["ipc"][uuid].is_none() {
            bail!("当前ipc不存在，需要重新扫描");
        }
        self.config["onvif"]["ipc"][uuid]["name"] = value(user);
        self.config["onvif"]["ipc"][uuid]["password"] = value(password);
        // self.config["ipc"][uuid]["stream_url"] = value(stream_url);
        self.update().await
    }

    pub async fn update_two_level_string(&mut self, class: &str, key: &str, val: &str) {
        self._update_two_level_item(class, key, value(val)).await;
    }
    pub async fn update_two_level_f64(&mut self, class: &str, key: &str, val: f64) {
        self._update_two_level_item(class, key, value(val)).await;
    }
    pub async fn update_two_level_int(&mut self, class: &str, key: &str, val: i64) {
        self._update_two_level_item(class, key, value(val)).await;
    }
    pub async fn _update_two_level_item(&mut self, class: &str, key: &str, val: Item) {
        if self.config[class].is_none() {
            error!("当前节点{}不存在，无法更新，请确认", class);
        }
        self.config[class][key] = val;
        if let Err(e) = self.update().await {
            error!("当前节点{}更新失败：{:?}", class, e);
        }
    }

    pub fn get_ipc_auth(&self, uuid: &str) -> Result<(String, String)> {
        if self.is_exist_ipc_config(uuid)? {
            return Ok((
                self._get_item_string(&self.config["onvif"]["ipc"][uuid]["name"])?,
                self._get_item_string(&self.config["onvif"]["ipc"][uuid]["password"])?,
            ));
        }
        bail!("当前ipc不存在，需要重新扫描");
    }
    pub fn get_ipc_rtsp(&self, uuid: &str) -> Result<String> {
        if self.is_exist_ipc_config(uuid)? {
            return self._get_item_string(&self.config["onvif"]["ipc"][uuid]["stream_url"]);
        }
        bail!("ipc[{}]不存在，或者未进行密码认证！", uuid)
    }
    pub fn get_ipc_auths(&self) -> Result<(Vec<String>, Vec<String>)> {
        // (self.config["ipc"]["auth"]["name"], self.config["ipc"]["auth"]["password"];
        let names = &self.config["onvif"]["auth"]["name"];
        let mut names_vec: Vec<String> = Vec::with_capacity(10);
        for i in names.as_array().unwrap().iter() {
            // debug!("[{:?}][{:?}]", i.as_str().unwrap(), i.to_string());
            names_vec.push(self._get_value_string(i)?);
        }
        let names = &self.config["onvif"]["auth"]["password"];
        let mut password_vec: Vec<String> = Vec::with_capacity(10);
        for i in names.as_array().unwrap().iter() {
            // password_vec.push(i.to_string());
            password_vec.push(self._get_value_string(i)?);
        }
        return Ok((names_vec, password_vec));
    }
    pub fn get_server_config_string(&self, item: &str) -> Result<String> {
        self._get_item_string(&self.config["server"][item])
    }
    pub fn get_two_level_string(&self, class: &str, key: &str) -> Result<String> {
        self._get_item_string(&self.config[class][key])
    }
    pub fn get_two_level_f64(&self, class: &str, key: &str) -> Result<f64> {
        self._get_item_f64(&self.config[class][key])
    }
    pub fn get_two_level_int(&self, class: &str, key: &str) -> Result<i64> {
        self._get_item_int(&self.config[class][key])
    }
    pub fn get_two_level_array_string(&self, class: &str, key: &str) -> Result<Vec<String>> {
        self._get_item_array_string(&self.config[class][key])
    }
    ///获取所有的路径配置，用于初始化文件夹
    pub fn init_server_paths(&self) {
        if self.config["server"]["path"].as_table().is_none() {
            return;
        }
        let tmp = match self.config["server"]["path"].as_table() {
            Some(table) => table,
            None => {
                return;
            }
        };
        for (_, item) in tmp.iter() {
            if let Err(e) = self.init_server_path(item) {
                warn!("{:?}", e);
            }
        }
    }
    fn init_server_path(&self, path: &Item) -> Result<()> {
        create_dir_all(self._get_item_string(path)?)?;
        Ok(())
    }
    pub fn get_server_path(&self, name: &str, default: &str) -> String {
        match self._get_item_string(&self.config["server"]["path"][name]) {
            Ok(path) => path,
            Err(_) => default.to_string(),
        }
    }
    pub fn _get_value_string(&self, item: &Value) -> Result<String> {
        match item.as_str() {
            Some(val) => Ok(val.to_string()),
            None => bail!("item[{:?}]的值获取失败", item),
        }
    }
    pub fn _get_item_string(&self, item: &Item) -> Result<String> {
        if item.is_none() {
            bail!("配置节点不存在，无法获取配置信息");
        }
        match item.as_str() {
            Some(val) => Ok(val.to_string()),
            None => bail!("item[{:?}]的值获取失败", item),
        }
    }

    pub fn _get_item_array_string(&self, item: &Item) -> Result<Vec<String>> {
        if item.is_none() {
            bail!("配置节点不存在，无法获取配置信息");
        }
        if let Some(array) = item.as_array() {
            let mut res = Vec::with_capacity(array.len());
            for node in array.iter() {
                res.push(self._get_value_string(node)?);
            }
            Ok(res)
        } else {
            bail!("item[{:?}]的值转成数组失败", item)
        }
    }
    pub fn get_os_config_f64(&self, item: &str) -> Result<f64> {
        self._get_item_f64(&self.config["os"][item])
    }
    pub fn _get_item_f64(&self, item: &Item) -> Result<f64> {
        if item.is_none() {
            bail!("配置节点不存在，无法获取配置信息");
        }
        item.as_float().into_result_by_msg("节点值不为float")
    }
    pub fn _get_item_int(&self, item: &Item) -> Result<i64> {
        if item.is_none() {
            bail!("配置节点不存在，无法获取配置信息");
        }
        item.as_integer().into_result_by_msg("节点值不为int")
    }

    pub fn get_cos_config(&self) -> Result<(String, String, String)> {
        let secret_id = self._get_item_string(&self.config["cos"]["secret_id"])?;
        let secret_key = self._get_item_string(&self.config["cos"]["secret_key"])?;
        let bucket = self._get_item_string(&self.config["cos"]["bucket"])?;
        let region = self._get_item_string(&self.config["cos"]["region"])?;
        // let scheme = self._get_item_string(&self.config["cos"]["scheme"])?;
        // let images = self._get_item_string(&self.config["cos"]["images"])?;
        // "middle-test-1300019243.cos.ap-chengdu.myqcloud.com";
        let host = format!("{}.cos.{}.myqcloud.com", bucket, region);
        Ok((secret_id, secret_key, host))
    }
}

#[async_std::test]
async fn toml_test() -> Result<()> {
    init_log();
    // let paths = std::fs::read_dir("./")?;
    // for path in paths {
    //     println!("Name: {}", path.unwrap().path().display())
    // }
    // let mut toml = Toml::init("./config.toml")?;
    // let (usernames, passwords) = toml.get_ipc_auth();
    // debug!("{:?}", usernames);
    // debug!("{:?}", passwords);
    //
    // toml.add_ipc_config("111", "rstp://127.00.1.1/8jie/iowq&jiofew".to_string()
    //                     , "user".to_string(), "pass".to_string(), 200, 100).await;
    //
    // debug!("{:?}", toml.update_ipc_auth("111", "user0".to_string(), "pass0".to_string(), "rstp://127.00.1.1/8jie/iowq&jiofew1111".to_string()).await);
    // debug!("{:?}", toml.update_ipc_auth("1112", "user2".to_string(), "pass2".to_string(), "rstp://127.00.1.1/8jie/iowq&jiofew22222".to_string()).await);
    Ok(())
}
#[async_std::test]
async fn reg_test() -> Result<()> {
    let test_str = "rtsp://192.168.50.25:554/0/av1";
    let test_str1 =
        "rtsp://192.168.50.166:554/cam/realmonitor?channel=1&subtype=1&unicast=true&proto=Onvif";
    let reg = Regex::new(RTSP_REG).unwrap();
    let cap = reg.captures(test_str).unwrap();
    assert_eq!(get_index_group_str(&cap, 1).unwrap(), ":554/0/av1");
    let cap = reg.captures(test_str1).unwrap();
    assert_eq!(
        get_index_group_str(&cap, 1).unwrap(),
        ":554/cam/realmonitor?channel=1&subtype=1&unicast=true&proto=Onvif"
    );
    Ok(())
}
