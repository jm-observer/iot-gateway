use base64;
use mio_httpc::CallBuilder;
use roxmltree;
use sha1::Sha1;

// fn main() {
// let time = "2021-03-16T15:01:11Z";
// let user = "admin";
// let password = "admin";
// let nonce = "-354666915";
// //LTM1NDY2NjkxNQ== LTM1NDY2NjkxNQ==
// println!("EncryptedNonce={:?}", base64::encode(nonce));
//
// let mut before = nonce.to_string();
// before.push_str(time);
// before.push_str(password);
// println!("before={:?}", before);
// let tmp_sha1 = Sha1::from(before).digest().bytes();
// println!("encryptedRaw ={:?}", tmp_sha1);
// println!("encryptPassword ={:?}", base64::encode(tmp_sha1));
//
// let gen = rand::random::<i32>();
// println!("rand={:?}", gen);
// let now = Utc::now();
// let gmt = now.format("%Y-%m-%dT%H:%M:%SZ").to_string();
// println!("now={:?}", gmt);
// }

fn main() {
    let url = "http://192.168.50.25/onvif/device_service";
    // let url = "http://192.168.50.25/onvif/media_service";
    let (_, body) = CallBuilder::post(Vec::from(get_profiles_xml("admin", "admin")))
        // let (response_meta, body) = CallBuilder::post(Vec::from(get_device_info_xml("admin", "admin")))
        // let (response_meta, body) = CallBuilder::post(Vec::from(get_snapshoturi_xml("admin","admin","MediaProfile000",)))
        .timeout_ms(5000)
        .header("Content-Type", "application/soap+xml; charset=utf-8")
        // .header("Accept-Encoding", "gzip")
        // .header("Connection", "close")
        .url(&url)
        .unwrap()
        .exec()
        .unwrap();

    println!("{:?}", String::from_utf8(body));
    // get_profiles_xml("admin", "admin");
    // get_stream_url_xml("admin", "admin", "MediaProfile000");
}

const REQUEST01: &str = r#"<env:Envelope xmlns:env="http://www.w3.org/2003/05/soap-envelope" xmlns:wsse="http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-wssecurity-secext-1.0.xsd" xmlns:wsu="http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-wssecurity-utility-1.0.xsd"><env:Header>"#;

const AUTH_HEARD01: &str = "<wsse:Security><wsse:UsernameToken><wsse:Username>";
const AUTH_HEARD02: &str = r#"</wsse:Username><wsse:Password Type="http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-username-token-profile-1.0#PasswordDigest">"#;
const AUTH_HEARD03: &str = r#"</wsse:Password><wsse:Nonce EncodingType="http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-soap-message-security-1.0#Base64Binary">"#;
const AUTH_HEARD04: &str = r#"</wsse:Nonce><wsu:Created>"#;
const AUTH_HEARD05: &str = r#"</wsu:Created></wsse:UsernameToken></wsse:Security>"#;

const REQUEST02: &str = r#"</env:Header><env:Body>"#;
#[allow(dead_code)]
const GET_DEVICEINFORMATION: &str =
    r#"<GetDeviceInformation xmlns="http://www.onvif.org/ver10/device/wsdl"/>"#;
#[allow(dead_code)]
const GET_CAPABILITIES: &str =
    r#"<GetCapabilities xmlns="http://www.onvif.org/ver10/device/wsdl"/>"#;
#[allow(dead_code)]
const GET_SNAPSHOTURI0: &str = r#"<trt:GetSnapshotUri><trt:ProfileToken>"#;
// const GET_SNAPSHOTURI1: &str = r#"</trt:ProfileToken></trt:GetSnapshotUri>"#;
#[allow(dead_code)]
const GET_SERVICES: &str = r#"<s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope"><s:Body xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:xsd="http://www.w3.org/2001/XMLSchema"><GetServices xmlns="http://www.onvif.org/ver10/device/wsdl"><IncludeCapability>false</IncludeCapability></GetServices></s:Body></s:Envelope>"#;

const GET_PROFILES: &str =
    r#"<GetProfiles xmlns="http://www.onvif.org/ver10/media/wsdl"/></env:Body></env:Envelope>"#;
#[allow(dead_code)]
const GET_STREAM_URL0: &str = r#"<GetStreamUri xmlns="http://www.onvif.org/ver10/media/wsdl"><StreamSetup>
<Stream xmlns="http://www.onvif.org/ver10/schema">RTP-Unicast</Stream>
<Transport xmlns="http://www.onvif.org/ver10/schema"><Protocol>UDP</Protocol></Transport></StreamSetup>
<ProfileToken>"#;
#[allow(dead_code)]
const GET_STREAM_URL1: &str = r#"</ProfileToken></GetStreamUri>"#;
#[allow(dead_code)]
const REQUEST04: &str = r#"</env:Body></env:Envelope>"#;
#[allow(dead_code)]
fn get_stream_url_xml(user: &str, password: &str, profile_token: &str) -> String {
    let mut head = REQUEST01.to_string();
    head.push_str(get_auth_head(user, password).as_str());
    head.push_str(REQUEST02);
    head.push_str(GET_STREAM_URL0);
    head.push_str(profile_token);
    head.push_str(GET_STREAM_URL1);
    head.push_str(REQUEST04);
    // head.push_str(REQUEST04);
    // println!("{}", head);
    let url = "http://192.168.50.166/onvif/media_service";
    let (_, body) = CallBuilder::post(Vec::from(head))
        // .digest_auth(true)
        .timeout_ms(5000)
        .header("Content-Type", "application/soap+xml; charset=utf-8")
        // .header("Accept-Encoding", "gzip")
        // .header("Connection", "close")
        .url(&url)
        .unwrap()
        .exec()
        .unwrap();
    let res = String::from_utf8(body).unwrap();
    println!("{:?}", res);
    let doc = roxmltree::Document::parse(res.as_str()).unwrap();
    let node = doc.descendants().find(|x| {
        return "Uri" == x.tag_name().name();
    });
    println!("{:?}", node.unwrap().text());
    res
}
#[allow(dead_code)]
fn get_services_xml() -> String {
    GET_SERVICES.to_string()
}

fn get_profiles_xml(user: &str, password: &str) -> String {
    // let mut head = r#"<?xml version="1.0" encoding="utf-8"?><env:Envelope xmlns:wsse="http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-wssecurity-secext-1.0.xsd" xmlns:env="http://www.w3.org/2003/05/soap-envelope"><env:Header>"#.to_string();
    let mut head = REQUEST01.to_string();
    head.push_str(get_auth_head(user, password).as_str());
    head.push_str(REQUEST02);
    head.push_str(GET_PROFILES);
    // head.push_str(REQUEST04);
    // println!("{}", head);
    let url = "http://192.168.50.166/onvif/media_service";
    let (_, body) = CallBuilder::post(Vec::from(head))
        // .digest_auth(true)
        .timeout_ms(5000)
        .header("Content-Type", "application/soap+xml; charset=utf-8")
        // .header("Accept-Encoding", "gzip")
        // .header("Connection", "close")
        .url(&url)
        .unwrap()
        .exec()
        .unwrap();
    let res = String::from_utf8(body).unwrap();
    println!("{:?}", res);

    let doc = roxmltree::Document::parse(res.as_str()).unwrap();
    // let nodes = doc.descendants().find(|x| {
    //     return "Profiles" == x.tag_name().name();
    // });
    let mut nodes = doc.descendants().filter(|x| {
        return "Profiles" == x.tag_name().name();
    });
    while let Some(node) = nodes.next() {
        println!("{:?}", node.attribute("token").unwrap());
    }

    res
}

#[allow(dead_code)]
fn get_snapshoturi_xml(user: &str, password: &str, token: &str) -> String {
    let mut head = REQUEST01.to_string();
    head.push_str(get_auth_head(user, password).as_str());
    head.push_str(REQUEST02);
    head.push_str(GET_SNAPSHOTURI0);
    head.push_str(token);
    head.push_str(GET_SNAPSHOTURI0);
    head.push_str(REQUEST04);
    head
}

//MediaProfile000
// fn GetServices() -> String {}
#[allow(dead_code)]
fn get_capabilites_xml(user: &str, password: &str) -> String {
    let mut head = REQUEST01.to_string();
    head.push_str(get_auth_head(user, password).as_str());
    head.push_str(REQUEST02);
    head.push_str(GET_CAPABILITIES);
    head.push_str(REQUEST04);
    head
}
#[allow(dead_code)]
fn get_device_info_xml(user: &str, password: &str) -> String {
    let mut head = REQUEST01.to_string();
    head.push_str(get_auth_head(user, password).as_str());
    head.push_str(REQUEST02);
    head.push_str(GET_DEVICEINFORMATION);
    head.push_str(REQUEST04);
    println!("{:?}", head);
    head
}

// fn get_common_request(user: &str, password: &str) -> String {
//     let mut head = REQUEST01.to_string();
//     head.push_str(get_auth_head(user, password).as_str());
//     head.push_str(REQUEST02);
//     head
// }

fn get_auth_head(user: &str, password: &str) -> String {
    let mut head = AUTH_HEARD01.to_string();
    head.push_str(user);
    head.push_str(AUTH_HEARD02);
    // let nonce = rand::random::<i32>().to_string();
    // let time = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let time = "2021-03-20T01:40:23Z".to_string();
    let nonce = "107907996".to_string();
    let mut before = nonce.clone();
    before.push_str(time.as_str());
    before.push_str(password);
    println!("before=[{}]", before);
    let encrypt_password = base64::encode(Sha1::from(before).digest().bytes());
    println!(
        "[{}]-[{}]-[{}]-[{}]-[{}]",
        user, password, nonce, time, encrypt_password
    );
    head.push_str(encrypt_password.as_str());
    head.push_str(AUTH_HEARD03);
    head.push_str(base64::encode(nonce).as_str());
    head.push_str(AUTH_HEARD04);
    head.push_str(time.as_str());
    head.push_str(AUTH_HEARD05);
    head
}

// fn main() {
//     let res = ureq::post("http://192.168.50.166/onvif/device_service")
//         .set("Content-Type", "application/soap+xml; charset=utf-8")
//         // .set()
//         .send_string(BODY)
//         .unwrap();
//     println!("!!!!!!!!!!!!!!!!!");
//     let mut buf: Vec<u8> = Vec::with_capacity(1024);
//     let mut abc = res.into_reader(); //.read_to_end(&mut buf).unwrap();
//     println!("!!!!!!!!!!!!!!!!!");
//     let ac1 = abc.read(&mut buf).unwrap();
//     println!("!!!!!!!!{}!!!!!!!!!{:?}", ac1, String::from_utf8(buf));
//     // println!("{:?}", text);
// }

// #[async_std::main]
// async fn main() -> surf::Result<()> {
//     // dbg!(surf::get("https://httpbin.org/get").recv_string().await?);
//     println!(
//         "{:?}",
//         surf::post("http://192.168.50.166/onvif/device_service")
//             .body(BODY)
//             .content_type("application/soap+xml; charset=utf-8")
//             .recv_string()
//             .await?
//     );
//
//     // let mut res = surf::get("https://httpbin.org/get").await?;
//     // dbg!(res.body_string().await?);
//     Ok(())
// }
