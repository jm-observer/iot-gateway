use crate::pub_use::*;


pub trait ExtObject {
    fn ext_get_str_or_default(&self, key: &str, default: &str) -> std::string::String;
    fn ext_get_ref_string(&self, key: &str) -> Result<&String>;
    fn ext_get_string(&self, key: &str) -> Result<String>;
    fn ext_get_int_or_default(&self, key: &str, default: f64) -> f64;
    fn ext_get_int(&self, key: &str) -> Result<f64>;

    fn ext_add_str_object(&mut self, key: &str, val: &str) -> &mut Self;
    fn ext_add_string_object(&mut self, key: &str, val: String) -> &mut Self;
    fn ext_add_f64_object(&mut self, key: &str, val: f64) -> &mut Self;
    fn ext_add_all(&mut self, json: Json);

    fn ext_get(&self, key: &str) -> Option<Json>;
}

pub trait CloneJson {
    fn clone(&self) -> Json;
}

impl CloneJson for Json {
    fn clone(&self) -> Self {
        match self {
            Json::NULL => return Json::NULL,
            Json::BOOL(val) => return Json::BOOL(val.clone()),
            Json::NUMBER(val) => return Json::NUMBER(val.clone()),
            Json::STRING(val) => return Json::STRING(val.clone()),
            Json::OBJECT { name, value } => {
                return Json::OBJECT {
                    name: name.clone(),
                    value: Box::new(value.as_ref().clone()),
                }
            }
            Json::JSON(val) => {
                let mut json = Json::new();
                for tmp in val {
                    json.add(tmp.clone());
                }
                return json;
            }
            Json::ARRAY(val) => {
                let mut json = Json::ARRAY(Vec::new());
                for tmp in val {
                    json.add(tmp.clone());
                }
                return json;
            }
        }
    }
}

impl ExtObject for Json {
    fn ext_add_all(&mut self, json: Json) {
        match json {
            Json::NULL => (),
            Json::JSON(val) => {
                for tmp in val {
                    self.add(tmp);
                }
            }
            Json::ARRAY(val) => {
                for tmp in val {
                    self.add(tmp);
                }
            }
            val => {
                self.add(val);
            }
        }
    }
    fn ext_add_str_object(&mut self, key: &str, val: &str) -> &mut Self {
        self.ext_add_string_object(key, val.to_string());
        self
    }
    fn ext_add_string_object(&mut self, key: &str, val: String) -> &mut Self {
        match self {
            Json::ARRAY(ref mut value) => {
                value.push(Json::OBJECT {
                    name: key.to_string(),
                    value: Box::new(Json::STRING(val)),
                });
            }
            Json::JSON(ref mut value) => value.push(Json::OBJECT {
                name: key.to_string(),
                value: Box::new(Json::STRING(val)),
            }),
            _ => {
                error!("ext_add_string_object.异常.{:?}", self);
            }
        }
        self
    }

    fn ext_add_f64_object(&mut self, key: &str, val: f64) -> &mut Self {
        match self {
            Json::ARRAY(ref mut value) => {
                value.push(Json::OBJECT {
                    name: key.to_string(),
                    value: Box::new(Json::NUMBER(val)),
                });
            }
            Json::JSON(ref mut value) => value.push(Json::OBJECT {
                name: key.to_string(),
                value: Box::new(Json::NUMBER(val)),
            }),
            _ => {
                error!("ext_add_string_object.异常.{:?}", self);
            }
        }
        self
    }

    fn ext_get(&self, key: &str) -> Option<Json> {
        self.get(key).map(|val| val.clone())
    }

    fn ext_get_int_or_default(&self, key: &str, default: f64) -> f64 {
        if let Ok(res) = self.ext_get_int(key) {
            return res;
        }
        default
    }
    fn ext_get_int(&self, key: &str) -> Result<f64> {
        if let Some(json) = self.get(key) {
            match json {
                Json::OBJECT { name: _, value } => {
                    if let Json::NUMBER(val) = value.as_ref() {
                        return Ok(val.clone());
                    }
                }
                _ => bail!("not contain key={}！", key),
            }
        }
        bail!("not contain key={}！", key)
    }
    fn ext_get_str_or_default(&self, key: &str, default: &str) -> std::string::String {
        if let Ok(res) = self.ext_get_string(key) {
            return res;
        } else {
            return default.to_string();
        }
    }
    fn ext_get_string(&self, key: &str) -> Result<String> {
        self.ext_get_ref_string(key).map(|x| x.to_string())
    }
    fn ext_get_ref_string(&self, key: &str) -> Result<&String> {
        if let Some(json) = self.get(key) {
            match json {
                Json::OBJECT { name: _, value } => {
                    if let Json::STRING(val) = value.as_ref() {
                        return Ok(val);
                    }
                }
                _ => bail!("not contain key={}！", key),
            }
        }
        bail!("not contain key={}！", key)
    }
}
