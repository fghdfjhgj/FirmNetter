# 项目概述

本项目是一个基于 Rust 编写的工具库，主要用于与 Android 设备交互、数据库操作、HTTP 请求处理等。项目中使用了多个外部库，并提供了丰富的 C FFI 接口，以便与其他语言（如 C/C++）进行互操作。

## 使用的库

### 2.1 reqwest
`reqwest` 是一个用于发起 HTTP 请求的 Rust 库，支持同步和异步请求。在本项目中，`reqwest` 主要用于实现 HTTP GET 和 POST 请求，以及文件下载功能。

### 2.2 diesel
`diesel` 是一个 ORM（对象关系映射）库，用于简化数据库操作。在本项目中，`diesel` 用于连接 PostgreSQL 数据库，并执行各种 CRUD 操作。

### 2.3 dotenv
`dotenv` 用于加载环境变量配置文件 `.env`，使得项目可以方便地读取环境变量中的配置信息。

### 2.4 libc
`libc` 提供了对 C 标准库的绑定，允许 Rust 代码直接调用 C 函数。在本项目中，`libc` 主要用于处理 C 风格字符串和其他 C 类型的数据结构。

## 模块介绍

### 3.1 flash_phone.rs
该模块主要负责与 Android 设备进行交互，获取设备的各种系统信息。提供了一系列函数来获取非 root 设备的数据，并且提供了内存管理相关的函数来释放动态分配的资源。

#### 3.1.1 函数列表
- `get_no_root_phone_data`: 获取非 root 设备的系统信息。
- `free_no_root_phone_data`: 释放 `NoRootPhoneData` 结构体中的资源。

### 3.2 utils.rs
该模块包含了一些通用的工具函数，例如执行外部命令、处理 C 风格字符串、时间处理等。

#### 3.2.1 函数列表
- `exec`: 执行外部命令并返回结果。
- `cstring_to_string`: 将 C 风格字符串转换为 Rust `String`。
- `str_to_cstr`: 将 Rust `String` 转换为 C 风格字符串。
- `free_command_result`: 释放 `CommandResult` 结构体中包含的 C 字符串内存。
- `Get_time`: 返回当前时间自 UNIX_EPOCH 以来的天数。
- `check_file`: 检查指定路径的文件是否存在。

### 3.3 kernel.rs
该模块提供了与内核相关的操作接口，例如验证文件完整性、签名图像文件、提取 payload 等。

#### 3.3.1 函数列表
- `unpack_img`: 解包图像文件。
- `repack_img`: 重新打包图像文件。
- `sign_img`: 对图像文件进行签名。
- `extract`: 提取 payload 中的特定分区。
- `hexpatch`: 对文件进行十六进制补丁操作。

### 3.4 sql.rs
该模块负责与数据库进行交互，包括建立连接、插入用户数据、检查卡密是否存在等。

#### 3.4.1 函数列表
- `establish_connection`: 建立数据库连接。
- `drop_db`: 释放数据库连接。
- `create_user`: 插入新的用户到 `users` 表。
- `check_kami_exists`: 检查指定名称的卡密是否存在。
- `get_user_id_by_imei`: 通过 IMEI 获取用户的唯一主键值。

### 3.5 web.rs
该模块提供了 HTTP 请求相关的接口，包括发送 GET 和 POST 请求、文件下载等。

#### 3.5.1 函数列表
- `web_get`: 发送 HTTP GET 请求。
- `web_post`: 发送 HTTP POST 请求。
- `downloader`: 下载文件并保存到本地。

### 3.6 ai.rs
该模块提供了与 AI 服务进行交互的接口，例如发送请求获取 AI 响应。

#### 3.6.1 函数列表
- `C_get_ai_stream`: 发送流式请求以获取 AI 响应。
- `C_get_ai_no_stream`: 发送非流式请求以获取 AI 响应。

