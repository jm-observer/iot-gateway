# ra2_bevy

#### 介绍

arm环境下，rust实现的网关程序。该网关与服务器仅依靠mqtt通讯。

##### 功能列表

- [x] onvif摄像头管理（发现、用户鉴权、监控转发、图片采集上报）
- [x] 系统监控（cpu、磁盘、内存）
- [x] 系统异常信息上报
- [x] ssh内网穿透（需要服务器配合）
- [x] 守护进程
- [x] mqtt
- [x] 腾讯cos对象存储（图片上传）
- [x] 定时心跳
- [x] 电流采集
- [x] 环境信息采集
- [x] 屏幕控制
- [ ] wifi智能连接
- [ ] 远程自动升级
- [ ] 短信发送（待集成）、拨打电话（待集成）
- [ ] esp8266节点芯片集成


##### 接口说明

1. [接口规范](https://note.youdao.com/s/6EtOjeSZ)
   1. mqttid规则：+config.toml:mqtt.id 
   2. mac地址规则:
      1. windows: 以太网网卡mac + "-" + WLAN网卡mac
      2. unix: eth0网卡mac + "-" + wlan0网卡mac
   4. 
      默认发给服务器：sbiot；默认类型：netgate
2. [接口清单](https://note.youdao.com/s/XTadGTBe)
3. 

#### 安装教程

本人是在window上开发，然后在子系统上交叉编译，最后将程序发送至树莓派

1. 安装rust环境
   1. 安装ubuntu子系统
      1. 安装rust环境
      2. 安装ipc的图片截图和视频转发：sudo apt-get -y install ffmpeg
      3. 安装arm的编译环境
         1. 添加源
         ```
         sudo cp /etc/apt/sources.list /etc/apt/sources.list.old
         sudo nano /etc/apt/sources.list
         ```
         ```
         # deb cdrom:[Ubuntu 20.04 LTS _Focal Fossa_ - Release amd64 (20200423)]/ focal main restricted
         deb [arch=amd64] http://archive.ubuntu.com/ubuntu/ focal main restricted universe multiverse
         deb [arch=amd64] http://security.ubuntu.com/ubuntu/ focal-security main restricted universe multiverse
         deb [arch=amd64] http://archive.ubuntu.com/ubuntu/ focal-updates main restricted universe multiverse
         # added these two:
         deb [arch=armhf] http://ports.ubuntu.com/ubuntu-ports focal main universe
         deb [arch=armhf] http://ports.ubuntu.com/ubuntu-ports focal-updates main universe
         # added these two:
         deb [arch=arm64] http://ports.ubuntu.com/ubuntu-ports focal main universe
         deb [arch=arm64] http://ports.ubuntu.com/ubuntu-ports focal-updates main universe
         ```
         2. 安装gcc
            ```
            sudo apt-get install gcc
            #确保有8版本
            sudo apt-cache search gcc-.-aarch64-linux-gnu
            sudo apt-get install gcc-8-aarch64-linux-gnu
            #确保有8版本
            sudo apt-cache search gcc-.-arm-linux-gnueabihf
            sudo apt-get install gcc-8-arm-linux-gnueabihf

            cd /usr/bin
            sudo ln aarch64-linux-gnu-gcc-8 aarch64-linux-gnu-gcc
            sudo ln arm-linux-gnueabihf-gcc-8 arm-linux-gnueabihf-gcc

            sudo dpkg --add-architecture armhf
            sudo dpkg --add-architecture arm64

            sudo apt-get install libudev-dev:armhf
            sudo apt-get install libudev-dev:arm64

            sudo apt-get install sshpass
            sudo apt-get install pkg-config
            ```
         3. 编译。请查看项目中的各个sh脚本
2. 修改项目配置参数

   config/config.toml中###开头的参数均需要修改
4. 


#### 使用说明

1. 安装必要软件
```
sudo apt-get -y install ffmpeg
```
2. 程序路径说明

路径文件|说明
|  ----  | ----  |
|/home/pi/iot/iot_gateway| 执行程序
|/home/pi/iot/config/config.toml| 配置参数
|/home/pi/iot/config/log4rs.yaml| 日记参数

##### 源码文件夹说明
|  文件/文件夹   | 说明  |
|  ----  | ----  |
| common  | 所有通用类、方法 |
| ext  | 当前只有onvif |
| ext_mqtt  | mqtt的消息处理模块 |
| ffmpeg  | ipc截图、视频转发模块 |
| mqtt  | mqtt客户端模块 |
| sub_task  | 子任务（线程）模块 |


#### 待优化/处理项

1. 错误枚举FailTypeEnum应该实现std::error::Error
2. 封装cos对象存储为crate

#### 问题

1. 视频转发可能有问题

#### 参与贡献

1.  Fork 本仓库
2.  新建 Feat_xxx 分支
3.  提交代码
4.  新建 Pull Request

