pub mod sql {
    use crate::other_utils::free_and_reset_c_string;
    use diesel::pg::PgConnection;
    use diesel::prelude::*;
    use dotenv::dotenv;
    use libc::c_char;
    use std::ffi::{CStr, CString};
    use std::sync::{Arc, Mutex};
    use std::{env, ptr};

    #[repr(C)]
    pub struct UserData {
        user_id: i32,
        user_name: *const c_char,      // 使用 C 字符串指针
        user_email: *const c_char,     // 使用 C 字符串指针
        user_password: *const c_char,  // 使用 C 字符串指针
        user_ip: *const c_char,        // 使用 C 字符串指针
        user_imei: *const c_char,      // 使用 C 字符串指针
        user_kami: *const c_char,
    }

    #[repr(C)]
    pub struct KamiData {
        kami_id: i32,
        kami_name: *const c_char,      // 使用 C 字符串指针
        kami_time: *const c_char,                // 使用时间戳 (秒)
        kami_if_kami: *const c_char,
    }
    // Diesel 表定义
    table! {
        users (id) {
            id -> Integer,
            name -> Text,
            email -> Text,
            password -> Text,
            ip -> Text,
            imei -> Text,
            kami -> Text,
        }
    }

    table! {
        kami (id) {
            id -> Integer,
            name -> Text,
            time -> Text,
            if_kami -> Text,
        }
    }

    #[derive(Insertable)]
    #[diesel(table_name = users)]
    pub struct NewUser<'a> {
        pub name: &'a str,
        pub email: &'a str,
        pub password: &'a str,
        pub ip: &'a str,
        pub imei: &'a str,
        pub kami: &'a str,
    }

    #[derive(Insertable)]
    #[diesel(table_name = kami)]
    pub struct NewKami<'a> {
        pub name: &'a str,
        pub time: &'a str,
        pub if_kami: &'a str,
    }

    // 定义一个持有数据库连接的结构体
    #[repr(C)]
    pub struct Database {
        conn: Arc<Mutex<PgConnection>>,
    }

    /// 建立到 PostgresSQL 数据库的连接。
    #[no_mangle]
    pub extern "C" fn establish_connection() -> *mut Database {
        dotenv().ok();

        let database_url = env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");

        let conn = PgConnection::establish(&database_url)
            .expect(&format!("Error connecting to {}", database_url));

        Box::into_raw(Box::new(Database {
            conn: Arc::new(Mutex::new(conn)),
        }))
    }
    /// 释放数据库连接。
    #[no_mangle]
    pub extern "C" fn drop_db(db: *mut Database) {
        if !db.is_null() {
            unsafe { drop(Box::from_raw(db)) };
        }
    }
    /// 插入新的用户到 users 表。
    ///
    /// # 参数
    ///
    /// * `db` - 数据库的可变指针。
    /// * `user_name` - 用户名的 C 风格字符串指针。
    /// * `user_email` - 用户邮箱的 C 风格字符串指针。
    /// * `user_password` - 用户密码的 C 风格字符串指针。
    /// * `user_ip` - 用户 IP 地址的 C 风格字符串指针。
    /// * `user_kami` - 用户是否是管理员的布尔值。
    ///
    /// # 返回
    ///
    /// 返回一个 C 风格字符串指针，表示操作的结果信息。
    #[no_mangle]
    pub extern "C" fn create_user(
        db: *mut Database,
        user_name: *const c_char,
        user_email: *const c_char,
        user_password: *const c_char,
        user_ip: *const c_char,
        user_imei:*const c_char,
        user_kami: *const c_char,
    ) -> *const c_char {
        // 检查传入的指针是否为空
        if db.is_null() || user_name.is_null() || user_email.is_null() || user_password.is_null() || user_ip.is_null() {
            return CString::new("Invalid parameters").unwrap().into_raw();
        }

        // 将 C 风格字符串指针转换为 Rust 的 CStr 类型
        let c_name = unsafe { CStr::from_ptr(user_name) };
        let c_email = unsafe { CStr::from_ptr(user_email) };
        let c_password = unsafe { CStr::from_ptr(user_password) };
        let c_ip = unsafe { CStr::from_ptr(user_ip) };
        let c_imei = unsafe { CStr::from_ptr(user_imei) };
        let c_kami = unsafe { CStr::from_ptr(user_kami) };

        // 将 CStr 类型转换为 Rust 的字符串切片
        let name_str = match c_name.to_str() {
            Ok(s) => s,
            Err(_) => return CString::new("Failed to convert name").unwrap().into_raw(),
        };
        let email_str = match c_email.to_str() {
            Ok(s) => s,
            Err(_) => return CString::new("Failed to convert email").unwrap().into_raw(),
        };
        let password_str = match c_password.to_str() {
            Ok(s) => s,
            Err(_) => return CString::new("Failed to convert password").unwrap().into_raw(),
        };
        let ip_str = match c_ip.to_str() {
            Ok(s) => s,
            Err(_) => return CString::new("Failed to convert ip").unwrap().into_raw(),
        };
        let imei_str = match c_imei.to_str() {
            Ok(s) => s,
            Err(_) => return CString::new("Failed to convert imei").unwrap().into_raw(),
        };
        let kami_str = match c_kami.to_str() {
            Ok(s) => s,
            Err(_) => return CString::new("Failed to convert kami").unwrap().into_raw(),
        };
        // 创建一个表示新用户的结构体实例
        let new_user = NewUser {
            name: name_str,
            email: email_str,
            password: password_str,
            ip: ip_str,
            imei: imei_str,
            kami: kami_str,
        };

        // 执行数据库插入操作
        let result = {
            let db_ref = unsafe { &mut *db }; // 解引用原始指针为可变引用
            let mut conn = db_ref.conn.lock().unwrap(); // 获取 MutexGuard 的可变引用
            diesel::insert_into(users::table)
                .values(&new_user)
                .execute(&mut *conn) // 使用可变引用
        };

        // 根据操作结果返回相应的 C 风格字符串指针
        match result {
            Ok(_) => CString::new("User created successfully").unwrap().into_raw(),
            Err(e) => CString::new(format!("Failed to create user: {}", e)).unwrap().into_raw(),
        }
    }


    /// 插入新地记录到 kami 表。
    ///
    /// # 参数
    ///
    /// - `db`: 数据库实例的可变指针。
    /// - `kami_name`: kami 名称的 C 风格字符串指针。
    /// - `kami_time`: kami 时间的 C 风格字符串指针。
    /// - `kami_if_kami`: 一个布尔值，表示卡密是否启用
    ///
    /// # 返回
    ///
    /// - 成功时返回成功消息的 C 风格字符串指针。
    /// - 失败时返回错误消息的 C 风格字符串指针。
    #[no_mangle]
    pub extern "C" fn create_kami(
        db: *mut Database,
        kami_name: *const c_char,
        kami_time: *const c_char,
        kami_if_kami: *const c_char,
    ) -> *const c_char {
        // 检查传入的指针是否为空
        if db.is_null() || kami_name.is_null() || kami_time.is_null() {
            return CString::new("Invalid parameters").unwrap().into_raw();
        }

        // 将 C 风格字符串指针转换为 Rust 的 CStr 类型
        let c_name = unsafe { CStr::from_ptr(kami_name) };
        let c_time = unsafe { CStr::from_ptr(kami_time) };
        let c_kami = unsafe { CStr::from_ptr(kami_if_kami) };

        // 将 CStr 类型转换为 Rust 的字符串切片
        let name_str = match c_name.to_str() {
            Ok(s) => s,
            Err(_) => return CString::new("Failed to convert name").unwrap().into_raw(),
        };
        let kami_if_str = match c_kami.to_str() {
            Ok(s) => s,
            Err(_) => return CString::new("Failed to convert kami").unwrap().into_raw(),
        };
        let time_str=match c_time.to_str() {
            Ok(s) => s,
            Err(_) => return CString::new("Failed to convert time").unwrap().into_raw(),
        };

        // 创建 NewKami 实例，用于插入到数据库
        let new_kami = NewKami {
            name: name_str,
            time: time_str,
            if_kami: kami_if_str,
        };

        // 执行数据库插入操作
        let result = {
            // 将原始指针解引用为可变引用
            let db_ref = unsafe { &mut *db };
            // 获取数据库连接的 MutexGuard
            let mut conn = db_ref.conn.lock().unwrap();
            // 执行插入操作
            diesel::insert_into(kami::table)
                .values(&new_kami)
                .execute(&mut *conn)
        };

        // 根据插入结果返回相应的消息
        match result {
            Ok(_) => CString::new("Kami record created successfully").unwrap().into_raw(),
            Err(e) => CString::new(format!("Failed to create kami record: {}", e)).unwrap().into_raw(),
        }
    }

    /// 检查指定名称的卡密是否存在。
    ///
    /// # 参数
    ///
    /// * `db` - 数据库的可变指针。
    /// * `kami_name` - 卡密名称的 C 风格字符串指针。
    ///
    /// # 返回
    ///
    /// 返回一个 C 风格字符串指针，表示操作的结果信息。
    #[no_mangle]
    pub extern "C" fn check_kami_exists(
        db: *mut Database,
        kami_name: *const c_char,
    ) -> *const c_char {
        // 检查传入的指针是否为空
        if db.is_null() || kami_name.is_null() {
            return CString::new("Invalid parameters").unwrap().into_raw();
        }

        // 将 C 风格字符串指针转换为 Rust 的 CStr 类型
        let c_name = unsafe { CStr::from_ptr(kami_name) };

        // 将 CStr 类型转换为 Rust 的字符串切片
        let name_str = match c_name.to_str() {
            Ok(s) => s,
            Err(_) => return CString::new("Failed to convert name").unwrap().into_raw(),
        };

        // 执行数据库查询操作
        let exists = {
            let db_ref = unsafe { &mut *db }; // 解引用原始指针为可变引用
            let mut conn = db_ref.conn.lock().unwrap(); // 获取 MutexGuard 的可变引用
            // 使用 Diesel 查询构建器检查卡密是否存在
            diesel::select(diesel::dsl::exists(
                kami::table.filter(kami::name.eq(name_str))
            ))
                .get_result::<bool>(&mut *conn).unwrap_or(false)
        };

        // 根据查询结果返回相应的 C 风格字符串指针
        if exists {
            CString::new("Kami exists").unwrap().into_raw()
        } else {
            CString::new("Kami does not exist").unwrap().into_raw()
        }
    }
    /// 定义一个 C 风格的函数，用于检查用户是否存在
    /// 该函数通过原始指针接收数据库连接和用户名，并返回一个表示用户是否存在的 C 风格字符串指针
    #[no_mangle]
    pub extern "C" fn check_user_exists(
        db: *mut Database,
        user: *const c_char,
    ) -> *const c_char {
        // 检查传入的指针是否为空
        if db.is_null() || user.is_null() {
            return CString::new("Invalid parameters").unwrap().into_raw();
        }

        // 将 C 风格字符串指针转换为 Rust 的 CStr 类型
        let c_name = unsafe { CStr::from_ptr(user)};

        // 将 CStr 类型转换为 Rust 的字符串切片
        let name_str = match c_name.to_str() {
            Ok(s) => s,
            Err(_) => return CString::new("Failed to convert name").unwrap().into_raw(),
        };

        // 执行数据库查询操作
        let exists = {
            let db_ref = unsafe { &mut *db }; // 解引用原始指针为可变引用
            let mut conn = db_ref.conn.lock().unwrap(); // 获取 MutexGuard 的可变引用
            diesel::select(diesel::dsl::exists(
                kami::table.filter(kami::name.eq(name_str))
            ))
                .get_result::<bool>(&mut *conn).unwrap_or(false) // 使用可变引用
        };

        // 根据查询结果返回相应的 C 风格字符串指针
        if exists {
            CString::new("Kami exists").unwrap().into_raw()
        } else {
            CString::new("Kami does not exist").unwrap().into_raw()
        }
    }
    /// 释放用户数据
    ///
    /// 此函数用于释放之前分配的用户数据结构。它接受一个指向用户数据的指针，
    /// 并安全地释放其中的字符串字段和整体结构的内存。
    ///
    /// # 参数
    /// - `data`: 指向 `UserData` 结构的指针。如果指针为 NULL，函数将直接返回。
    ///
    /// # 安全性
    /// 该函数涉及裸指针的使用和释放，因此需要谨慎处理以避免内存泄漏或未定义行为。
    /// 确保传递给此函数的指针是有效的，且未被其他地方使用。
    #[no_mangle]
    pub extern "C" fn free_user_data(data: *mut UserData) {
        if data.is_null() {
            return;
        }

        let mut data = unsafe { Box::from_raw(data) };

        // 安全地释放并重置 C 字符串

            free_and_reset_c_string(&mut data.user_name);
            free_and_reset_c_string(&mut data.user_email);
            free_and_reset_c_string(&mut data.user_password);
            free_and_reset_c_string(&mut data.user_ip);
            free_and_reset_c_string(&mut data.user_imei);
            free_and_reset_c_string(&mut data.user_kami);
            data.user_id = 0;


        // `data` 在这里被丢弃，释放 Box 分配的内存
    }

    /// 释放 KamiData 结构体的内存
    ///
    /// 当与 C 代码互操作时，需要提供一个外部接口来释放内存。
    /// 此函数确保通过 C 代码分配的 KamiData 结构体在使用后被正确释放。
    ///
    /// # 参数
    ///
    /// * `data` - 指向 KamiData 结构体的指针。如果指针为 NULL，则函数直接返回。
    #[no_mangle]
    pub extern "C" fn free_kami_data(data: *mut KamiData) {
        // 检查指针是否为 NULL，如果为 NULL，则直接返回
        if data.is_null() {
            return;
        }

        // 将原始指针转换为 Box，以便在 Rust 中管理内存
        let mut data = unsafe { Box::from_raw(data) };

        // 安全地释放并重置 C 字符串

            free_and_reset_c_string(&mut data.kami_name);
            free_and_reset_c_string(&mut data.kami_if_kami);
            free_and_reset_c_string(&mut data.kami_time);
            data.kami_id = 0;



        // `data` 在这里被丢弃，释放 Box 分配的内存
    }
    /// 通过 IMEI 获取用户的唯一主键值。
    ///
    /// # 参数
    ///
    /// * `db` - 数据库的可变指针。
    /// * `imei` - 用户 IMEI 的 C 风格字符串指针。
    ///
    /// # 返回
    ///
    /// 返回用户的唯一主键值，如果用户不存在则返回 -1。
    #[no_mangle]
    pub extern "C" fn get_user_id_by_imei(db: *mut Database, imei: *const c_char) -> i32 {
        // 检查传入的指针是否为空
        if db.is_null() || imei.is_null() {
            return -1;
        }

        // 将 C 风格字符串指针转换为 Rust 的 CStr 类型
        let c_imei = unsafe { CStr::from_ptr(imei) };

        // 将 CStr 类型转换为 Rust 的字符串切片
        let imei_str = match c_imei.to_str() {
            Ok(s) => s,
            Err(_) => return -1,
        };

        // 执行数据库查询操作
        let user_id = {
            let db_ref = unsafe { &mut *db }; // 解引用原始指针为可变引用
            let mut conn = db_ref.conn.lock().unwrap(); // 获取 MutexGuard 的可变引用
            // 使用 Diesel 查询构建器获取用户 ID
            users::table
                .filter(users::imei.eq(imei_str))
                .select(users::id)
                .first::<i32>(&mut *conn)
                .optional() // 返回 Option<i32>
                .unwrap_or(None) // 处理查询错误
        };

        // 根据查询结果返回相应的用户 ID 或 -1
        user_id.unwrap_or_else(|| -1)
    }
    /// 通过用户 ID 获取用户数据。
    ///
    /// # 参数
    ///
    /// * `db` - 数据库的可变指针。
    /// * `user_id` - 用户的唯一主键值。
    ///
    /// # 返回
    ///
    /// 返回一个指向 `UserData` 结构体的指针，如果用户不存在则返回 NULL。
    #[no_mangle]
    pub extern "C" fn get_user_by_id(db: *mut Database, user_id: i32) -> *mut UserData {
        if db.is_null() {
            return ptr::null_mut();
        }

        // 执行数据库查询操作
        let user = {
            let db_ref = unsafe { &mut *db }; // 解引用原始指针为可变引用
            let mut conn = db_ref.conn.lock().unwrap(); // 获取 MutexGuard 的可变引用
            // 使用 Diesel 查询构建器获取用户数据
            users::table
                .filter(users::id.eq(user_id))
                .first::<(i32, String, String, String, String, String, String)>(&mut *conn)
                .optional() // 返回 Option<(i32, String, String, String, String, String, bool)>
                .unwrap_or(None) // 处理查询错误
        };

        // 根据查询结果返回相应的 UserData 结构体或 NULL
        if let Some((id, name, email, password, ip, imei, kami)) = user {
            let user_name_cstr = CString::new(name).unwrap();
            let user_email_cstr = CString::new(email).unwrap();
            let user_password_cstr = CString::new(password).unwrap();
            let user_ip_cstr = CString::new(ip).unwrap();
            let user_imei_cstr = CString::new(imei).unwrap();
            let user_kami_cstr = CString::new(kami).unwrap();

            let user_data = UserData {
                user_id: id,
                user_name: user_name_cstr.into_raw(),
                user_email: user_email_cstr.into_raw(),
                user_password: user_password_cstr.into_raw(),
                user_ip: user_ip_cstr.into_raw(),
                user_imei: user_imei_cstr.into_raw(),
                user_kami: user_kami_cstr.into_raw(),
            };

            Box::into_raw(Box::new(user_data))
        } else {
            ptr::null_mut()
        }
    }
    #[no_mangle]
    /// 根据名称获取Kami数据
    ///
    /// 此函数被设计为C语言接口，用于从数据库中根据名称查询Kami信息，并返回一个KamiData结构体。
    /// 如果数据库指针或名称指针为空，或者没有找到对应的Kami信息，则返回空指针。
    ///
    /// # 参数
    /// - `db`: *mut Database - 数据库的指针
    /// - `kami_name`: *const c_char - Kami名称的C字符串指针
    ///
    /// # 返回
    /// - 成功时返回一个指向KamiData结构体的指针
    /// - 失败时返回空指针
    pub extern "C" fn get_kami_by_name(db: *mut Database, kami_name: *const c_char) -> *mut KamiData {
        // 检查传入的指针是否为空
        if db.is_null() || kami_name.is_null() {
            return ptr::null_mut();
        }

        // 将C字符串指针转换为Rust字符串
        let kami_name_str = unsafe { CStr::from_ptr(kami_name).to_str().unwrap() };

        // 从数据库中查询Kami信息
        let kami = {
            // 获取数据库引用并解锁连接
            let db_ref = unsafe { &mut *db };
            let mut conn = db_ref.conn.lock().unwrap();

            // 执行数据库查询并获取结果
            kami::table
                .filter(kami::name.eq(kami_name_str))
                .first::<(i32, String,String, String)>(&mut *conn)
                .optional()
                .unwrap_or(None)
        };

        // 根据查询结果构建KamiData结构体并返回
        if let Some((kami_id, kami_name, kami_if_kami, kami_time)) = kami {
            // 将字符串转换为C字符串
            let kami_name_cstr = CString::new(kami_name).unwrap();
            let kami_if_kami_cstr = CString::new(kami_if_kami).unwrap();
            let kami_time_cstr = CString::new(kami_time).unwrap();

            // 构建KamiData结构体
            let kami_data = KamiData {
                kami_id,
                kami_name: kami_name_cstr.into_raw(),
                kami_if_kami: kami_if_kami_cstr.into_raw(),
                kami_time: kami_time_cstr.into_raw(),
            };

            // 将KamiData结构体转换为指针并返回
            Box::into_raw(Box::new(kami_data))
        } else {
            // 如果查询结果为空，则返回空指针
            ptr::null_mut()
        }
    }
}

