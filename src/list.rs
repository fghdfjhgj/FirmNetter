pub mod list {
    use std::fmt;
    use std::iter::{FromIterator, IntoIterator};
    use std::marker::PhantomData;
    use std::ptr;

    #[derive(Debug)]
    pub struct Node<T> {
        pub(crate) data: T,
        prev: *mut Node<T>,
        pub(crate) next: *mut Node<T>,
    }

    pub struct DoublyLinkedList<T> {
        pub(crate) head: *mut Node<T>,
        tail: *mut Node<T>,
        len: usize,
        marker: PhantomData<Box<Node<T>>>,
    }

    // 基础实现
    impl<T> DoublyLinkedList<T> {
        // ... existing code ...

        /// 构造一个新的空双向链表
        ///
        /// # 泛型参数
        /// - T: 链表节点存储的数据类型
        ///
        /// # 返回值
        /// 返回一个初始化为空的 `DoublyLinkedList` 结构体实例，其中：
        /// - `head`: 指向头节点的原始指针，初始化为空指针
        /// - `tail`: 指向尾节点的原始指针，初始化为空指针
        /// - `len`: 链表长度，初始化为 0
        /// - `marker`: 类型标记，用于确保泛型参数 T 的正确性

        pub fn new() -> Self {
            DoublyLinkedList {
                head: ptr::null_mut(),
                tail: ptr::null_mut(),
                len: 0,
                marker: PhantomData,
            }
        }

        // ... existing code ...

        // ... existing code ...

        /// 获取链表当前的元素数量
        ///
        /// # 泛型参数
        /// - T: 链表中存储元素的类型
        ///
        /// # 返回值
        /// 返回一个 `usize` 类型值，表示链表中当前存储的元素个数

        pub fn len(&self) -> usize {
            self.len
        }

        // ... existing code ...

        // ... existing code ...

        /// 判断链表是否为空
        ///
        /// # 泛型参数
        /// - T: 链表中存储元素的类型
        ///
        /// # 返回值
        /// 返回一个布尔值，表示链表是否为空（即长度为 0）

        pub fn is_empty(&self) -> bool {
            self.len == 0
        }

        // ... existing code ...

        // ... existing code ...

        /// 在双向链表的头部插入一个新元素
        ///
        /// # 泛型参数
        /// - T: 链表节点中存储的数据类型
        ///
        /// # 参数
        /// - `data`: 要插入到链表头部的数据
        ///
        /// # 操作逻辑
        /// 1. 在堆上创建一个新的节点对象，并将其转换为原始指针
        /// 2. 新节点的前驱指针初始化为空（因为是插入头部）
        /// 3. 新节点的后继指针指向当前头节点
        /// 4. 如果当前头节点非空，更新其前驱指针为新节点
        /// 5. 如果链表原本为空（头指针为空），则同时更新尾指针为新节点
        /// 6. 更新头指针指向新节点
        /// 7. 链表长度增加 1

        pub fn push_front(&mut self, data: T) {
            let new_node = Box::into_raw(Box::new(Node {
                data,
                prev: ptr::null_mut(),
                next: self.head,
            }));

            if !self.head.is_null() {
                unsafe {
                    (*self.head).prev = new_node;
                }
            } else {
                self.tail = new_node;
            }

            self.head = new_node;
            self.len += 1;
        }

        // ... existing code ...

        // ... existing code ...

        /// 在双向链表的尾部插入一个新元素
        ///
        /// # 泛型参数
        /// - T: 链表节点中存储的数据类型
        ///
        /// # 参数
        /// - `data`: 要插入到链表尾部的数据
        ///
        /// # 操作逻辑
        /// 1. 在堆上创建一个新的节点对象，并将其转换为原始指针
        /// 2. 新节点的前驱指针指向当前尾节点
        /// 3. 新节点的后继指针初始化为空（因为是插入尾部）
        /// 4. 如果当前尾节点非空，更新其后继指针为新节点
        /// 5. 如果链表原本为空（尾指针为空），则同时更新头指针为新节点
        /// 6. 更新尾指针指向新节点
        /// 7. 链表长度增加 1

        pub fn push_back(&mut self, data: T) {
            let new_node = Box::into_raw(Box::new(Node {
                data,
                prev: self.tail,
                next: ptr::null_mut(),
            }));

            if !self.tail.is_null() {
                unsafe {
                    (*self.tail).next = new_node;
                }
            } else {
                self.head = new_node;
            }

            self.tail = new_node;
            self.len += 1;
        }

        // ... existing code ...

        // ... existing code ...

        /// 移除并返回链表头部的元素
        ///
        /// # 泛型参数
        /// - T: 链表节点中存储的数据类型
        ///
        /// # 返回值
        /// 返回一个 `Option<T>` 类型值：
        /// - 如果链表非空，返回 `Some(data)`，其中 `data` 是被移除的头部节点的数据
        /// - 如果链表为空，返回 `None`
        ///
        /// # 操作逻辑
        /// 1. 检查头指针是否为空，为空则直接返回 None（链表为空）
        /// 2. 否则，将头节点从堆上取回并解引用
        /// 3. 更新头指针指向原头节点的下一个节点
        /// 4. 如果新头节点存在，将其前驱指针置空（成为新的头部）
        /// 5. 如果新头节点不存在，说明链表已空，更新尾指针为空
        /// 6. 链表长度减 1，并返回原头节点的数据

        pub fn pop_front(&mut self) -> Option<T> {
            if self.head.is_null() {
                return None;
            }

            unsafe {
                let old_head = Box::from_raw(self.head);
                self.head = old_head.next;

                if !self.head.is_null() {
                    (*self.head).prev = ptr::null_mut();
                } else {
                    self.tail = ptr::null_mut();
                }

                self.len -= 1;
                Some(old_head.data)
            }
        }

        // ... existing code ...

        // ... existing code ...

        /// 移除并返回链表尾部的元素
        ///
        /// # 泛型参数
        /// - T: 链表节点中存储的数据类型
        ///
        /// # 返回值
        /// 返回一个 `Option<T>` 类型值：
        /// - 如果链表非空，返回 `Some(data)`，其中 `data` 是被移除的尾部节点的数据
        /// - 如果链表为空，返回 `None`
        ///
        /// # 操作逻辑
        /// 1. 检查尾指针是否为空，为空则直接返回 None（链表为空）
        /// 2. 否则，将尾节点从堆上取回并解引用
        /// 3. 更新尾指针指向原尾节点的前一个节点
        /// 4. 如果新尾节点存在，将其后继指针置空（成为新的尾部）
        /// 5. 如果新尾节点不存在，说明链表已空，更新头指针为空
        /// 6. 链表长度减 1，并返回原尾节点的数据

        pub fn pop_back(&mut self) -> Option<T> {
            if self.tail.is_null() {
                return None;
            }

            unsafe {
                let old_tail = Box::from_raw(self.tail);
                self.tail = old_tail.prev;

                if !self.tail.is_null() {
                    (*self.tail).next = ptr::null_mut();
                } else {
                    self.head = ptr::null_mut();
                }

                self.len -= 1;
                Some(old_tail.data)
            }
        }

        // ... existing code ...

        // ... existing code ...

        /// 获取链表头部元素的引用
        ///
        /// # 泛型参数
        /// - T: 链表节点中存储的数据类型
        ///
        /// # 返回值
        /// 返回一个 `Option<&T>` 类型值：
        /// - 如果链表非空，返回指向头部节点数据的引用 `Some(&data)`
        /// - 如果链表为空，返回 `None`
        ///
        /// # 操作逻辑
        /// 1. 检查头指针是否为空，为空则返回 None（链表为空）
        /// 2. 否则，通过解引用头指针获取节点数据，并返回其引用

        pub fn front(&self) -> Option<&T> {
            if self.head.is_null() {
                None
            } else {
                unsafe { Some(&(*self.head).data) }
            }
        }

        // ... existing code ...

        // ... existing code ...

        /// 获取链表尾部元素的引用
        ///
        /// # 泛型参数
        /// - T: 链表节点中存储的数据类型
        ///
        /// # 返回值
        /// 返回一个 `Option<&T>` 类型值：
        /// - 如果链表非空，返回指向尾部节点数据的引用 `Some(&data)`
        /// - 如果链表为空，返回 `None`
        ///
        /// # 操作逻辑
        /// 1. 检查尾指针是否为空，为空则返回 None（链表为空）
        /// 2. 否则，通过解引用尾指针获取节点数据，并返回其引用

        pub fn back(&self) -> Option<&T> {
            if self.tail.is_null() {
                None
            } else {
                unsafe { Some(&(*self.tail).data) }
            }
        }

        // ... existing code ...

        // ... existing code ...

        /// 获取链表头部元素的可变引用
        ///
        /// # 泛型参数
        /// - T: 链表节点中存储的数据类型
        ///
        /// # 返回值
        /// 返回一个 `Option<&mut T>` 类型值：
        /// - 如果链表非空，返回指向头部节点数据的可变引用 `Some(&mut data)`
        /// - 如果链表为空，返回 `None`
        ///
        /// # 操作逻辑
        /// 1. 检查头指针是否为空，为空则返回 None（链表为空）
        /// 2. 否则，通过解引用头指针获取节点数据，并返回其可变引用

        pub fn front_mut(&mut self) -> Option<&mut T> {
            if self.head.is_null() {
                None
            } else {
                unsafe { Some(&mut (*self.head).data) }
            }
        }

        // ... existing code ...

        // ... existing code ...

        /// 获取链表尾部元素的可变引用
        ///
        /// # 泛型参数
        /// - T: 链表节点中存储的数据类型
        ///
        /// # 返回值
        /// 返回一个 `Option<&mut T>` 类型值：
        /// - 如果链表非空，返回指向尾部节点数据的可变引用 `Some(&mut data)`
        /// - 如果链表为空，返回 `None`
        ///
        /// # 操作逻辑
        /// 1. 检查尾指针是否为空，为空则返回 None（链表为空）
        /// 2. 否则，通过解引用尾指针获取节点数据，并返回其可变引用

        pub fn back_mut(&mut self) -> Option<&mut T> {
            if self.tail.is_null() {
                None
            } else {
                unsafe { Some(&mut (*self.tail).data) }
            }
        }

        // ... existing code ...
    }

    // 移除操作
    impl<T: PartialEq> DoublyLinkedList<T> {
        // ... existing code ...

        /// 移除链表中第一个与指定值相等的元素
        ///
        /// # 泛型参数
        /// - T: 链表节点中存储的数据类型，必须实现 `PartialEq` trait 以支持比较操作
        ///
        /// # 参数
        /// - `data`: 要移除的元素的引用
        ///
        /// # 返回值
        /// 返回一个布尔值：
        /// - 如果成功找到并移除了匹配的节点，返回 `true`
        /// - 如果没有找到匹配的节点，返回 `false`
        ///
        /// # 操作逻辑
        /// 1. 从头节点开始遍历链表
        /// 2. 对于每个节点，比较其数据与给定值：
        ///    - 如果找到匹配节点，则：
        ///      - 更新其前驱节点的 `next` 指针
        ///      - 更新其后继节点的 `prev` 指针
        ///      - 释放当前节点的内存
        ///      - 减少链表长度并返回 `true`
        ///    - 否则继续遍历下一个节点
        /// 3. 遍历结束后未找到匹配项则返回 `false`

        pub fn remove(&mut self, data: &T) -> bool {
            let mut current = self.head;

            while !current.is_null() {
                unsafe {
                    if &(*current).data == data {
                        // 更新前驱节点的next指针
                        if !(*current).prev.is_null() {
                            (*(*current).prev).next = (*current).next;
                        } else {
                            self.head = (*current).next;
                        }

                        // 更新后继节点的prev指针
                        if !(*current).next.is_null() {
                            (*(*current).next).prev = (*current).prev;
                        } else {
                            self.tail = (*current).prev;
                        }

                        // 释放内存
                        let _ = Box::from_raw(current);
                        self.len -= 1;
                        return true;
                    }
                    current = (*current).next;
                }
            }
            false
        }

        // ... existing code ...

        // ... existing code ...

        /// 移除链表中所有与指定值相等的元素
        ///
        /// # 泛型参数
        /// - T: 链表节点中存储的数据类型，必须实现 `PartialEq` trait 以支持比较操作
        ///
        /// # 参数
        /// - `data`: 要移除的元素的引用
        ///
        /// # 返回值
        /// 返回一个 `usize` 类型值，表示成功移除的匹配节点数量
        ///
        /// # 操作逻辑
        /// 1. 初始化计数器 `count` 为 0
        /// 2. 从头节点开始遍历链表
        /// 3. 对于每个节点，保存其后继节点以备后续遍历使用
        /// 4. 如果当前节点的数据与给定值相等，则：
        ///    - 更新其前驱节点的 `next` 指针
        ///    - 更新其后继节点的 `prev` 指针
        ///    - 释放当前节点的内存
        ///    - 减少链表长度，计数器加 1
        /// 5. 继续遍历下一个节点，直到链表结束
        /// 6. 返回总共移除的节点数量

        pub fn remove_all(&mut self, data: &T) -> usize {
            let mut count = 0;
            let mut current = self.head;

            while !current.is_null() {
                unsafe {
                    let next = (*current).next;

                    if &(*current).data == data {
                        // 更新前驱节点的next指针
                        if !(*current).prev.is_null() {
                            (*(*current).prev).next = (*current).next;
                        } else {
                            self.head = (*current).next;
                        }

                        // 更新后继节点的prev指针
                        if !(*current).next.is_null() {
                            (*(*current).next).prev = (*current).prev;
                        } else {
                            self.tail = (*current).prev;
                        }

                        // 释放内存
                        let _ = Box::from_raw(current);
                        self.len -= 1;
                        count += 1;
                    }
                    current = next;
                }
            }
            count
        }

        // ... existing code ...
    }

    // 迭代器实现
    impl<T> DoublyLinkedList<T> {
        // ... existing code ...

        /// 创建一个遍历链表的前向不可变迭代器
        ///
        /// # 泛型参数
        /// - T: 链表节点中存储的数据类型
        ///
        /// # 返回值
        /// 返回一个 `Iter<'_, T>` 类型的前向迭代器，用于从头到尾遍历链表中的元素
        ///
        /// # 迭代器字段初始化
        /// - `current`: 当前迭代位置的节点指针，初始化为链表头部
        /// - `marker`: 类型标记，用于协变，确保迭代器的生命周期与链表的不可变借用一致

        pub fn iter(&self) -> Iter<'_, T> {
            Iter {
                current: self.head,
                marker: PhantomData,
            }
        }

        // ... existing code ...

        // ... existing code ...

        /// 创建一个遍历链表的前向可变迭代器
        ///
        /// # 泛型参数
        /// - T: 链表节点中存储的数据类型
        ///
        /// # 返回值
        /// 返回一个 `IterMut<'_, T>` 类型的前向可变迭代器，用于从头到尾遍历并修改链表中的元素
        ///
        /// # 迭代器字段初始化
        /// - `current`: 当前迭代位置的节点指针，初始化为链表头部
        /// - `marker`: 类型标记，用于协变，确保迭代器的生命周期与链表的可变借用一致

        pub fn iter_mut(&mut self) -> IterMut<'_, T> {
            IterMut {
                current: self.head,
                marker: PhantomData,
            }
        }

        // ... existing code ...

        // ... existing code ...

        /// 创建一个消费型迭代器，用于拥有链表所有权的遍历
        ///
        /// # 泛型参数
        /// - T: 链表节点中存储的数据类型
        ///
        /// # 返回值
        /// 返回一个 `IntoIter<T>` 类型的消费迭代器，该迭代器将逐步取得链表节点的所有权
        ///
        /// # 操作逻辑
        /// 该消费迭代器通过将当前链表实例 `self` 移动到迭代器结构体 `IntoIter` 中来创建

        pub fn into_iter(self) -> IntoIter<T> {
            IntoIter { list: self }
        }

        // ... existing code ...
    }

    // 前向不可变迭代器
    pub struct Iter<'a, T> {
        current: *mut Node<T>,
        marker: PhantomData<&'a Node<T>>,
    }

    impl<'a, T> Iterator for Iter<'a, T> {
        type Item = &'a T;

        // ... existing code ...

        /// 实现迭代器的 `next` 方法，用于获取下一个元素的引用
        ///
        /// # 泛型参数
        /// - T: 链表节点中存储的数据类型
        ///
        /// # 返回值
        /// 返回一个 `Option<&T>` 类型值：
        /// - 如果还有剩余元素，返回 `Some(&data)`，其中 `data` 是当前节点的数据引用
        /// - 如果已到达链表尾部，返回 `None`
        ///
        /// # 操作逻辑
        /// 1. 检查当前节点指针是否为空，为空则表示迭代完成
        /// 2. 否则，获取当前节点数据的不可变引用
        /// 3. 更新当前指针为下一个节点
        /// 4. 返回当前节点数据的引用

        fn next(&mut self) -> Option<Self::Item> {
            if self.current.is_null() {
                None
            } else {
                unsafe {
                    let item = &(*self.current).data;
                    self.current = (*self.current).next;
                    Some(item)
                }
            }
        }

        // ... existing code ...
    }

    // 前向可变迭代器
    pub struct IterMut<'a, T> {
        current: *mut Node<T>,
        marker: PhantomData<&'a mut Node<T>>,
    }

    impl<'a, T> Iterator for IterMut<'a, T> {
        type Item = &'a mut T;

        // ... existing code ...

        /// 实现可变迭代器的 `next` 方法，用于获取下一个元素的可变引用
        ///
        /// # 泛型参数
        /// - T: 链表节点中存储的数据类型
        ///
        /// # 返回值
        /// 返回一个 `Option<&mut T>` 类型值：
        /// - 如果还有剩余元素，返回 `Some(&mut data)`，其中 `data` 是当前节点的可变引用
        /// - 如果已到达链表尾部，返回 `None`
        ///
        /// # 操作逻辑
        /// 1. 检查当前节点指针是否为空，为空则表示迭代完成
        /// 2. 否则，获取当前节点数据的可变引用
        /// 3. 更新当前指针为下一个节点
        /// 4. 返回当前节点数据的可变引用

        fn next(&mut self) -> Option<Self::Item> {
            if self.current.is_null() {
                None
            } else {
                unsafe {
                    let item = &mut (*self.current).data;
                    self.current = (*self.current).next;
                    Some(item)
                }
            }
        }

        // ... existing code ...
    }

    // 消费迭代器
    pub struct IntoIter<T> {
        list: DoublyLinkedList<T>,
    }

    impl<T> Iterator for IntoIter<T> {
        type Item = T;

        // ... existing code ...

        /// 实现消费迭代器的 `next` 方法，用于逐个取出链表头部元素
        ///
        /// # 泛型参数
        /// - T: 链表节点中存储的数据类型
        ///
        /// # 返回值
        /// 返回一个 `Option<T>` 类型值：
        /// - 如果链表非空，返回从头部移除的元素所有权 `Some(data)`
        /// - 如果链表为空，返回 `None`
        ///
        /// # 操作逻辑
        /// 通过调用 `pop_front` 方法移除并返回链表头部的元素

        fn next(&mut self) -> Option<Self::Item> {
            self.list.pop_front()
        }

        // ... existing code ...
    }

    // 从迭代器创建链表
    impl<T> FromIterator<T> for DoublyLinkedList<T> {
        // ... existing code ...

        /// 从迭代器创建一个新的双向链表
        ///
        /// # 泛型参数
        /// - T: 链表节点中存储的数据类型
        /// - I: 实现 `IntoIterator` trait 的迭代器类型，其元素类型为 T
        ///
        /// # 参数
        /// - `iter`: 一个实现了 `IntoIterator<Item = T>` 的迭代器对象
        ///
        /// # 返回值
        /// 返回一个包含迭代器所有元素的 `DoublyLinkedList<T>` 实例，元素顺序与迭代器一致
        ///
        /// # 操作逻辑
        /// 1. 创建一个空的双向链表
        /// 2. 遍历输入迭代器中的每个元素
        /// 3. 将每个元素通过 `push_back` 方法插入到链表尾部
        /// 4. 返回构建完成的链表

        fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
            let mut list = DoublyLinkedList::new();
            for item in iter {
                list.push_back(item);
            }
            list
        }

        // ... existing code ...
    }

    // 链表转换为迭代器
    impl<T> IntoIterator for DoublyLinkedList<T> {
        type Item = T;
        type IntoIter = IntoIter<T>;

        // ... existing code ...

        /// 将链表转换为其对应的消费迭代器
        ///
        /// # 泛型参数
        /// - T: 链表节点中存储的数据类型
        ///
        /// # 返回值
        /// 返回一个 `Self::IntoIter` 类型的消费迭代器，用于逐个取得链表节点的所有权
        ///
        /// # 操作逻辑
        /// 直接调用链表的 `into_iter` 方法获取对应的消费迭代器

        fn into_iter(self) -> Self::IntoIter {
            self.into_iter()
        }

        // ... existing code ...
    }

    // 格式化输出
    impl<T: fmt::Debug> fmt::Debug for DoublyLinkedList<T> {
        // ... existing code ...

        /// 实现链表的格式化显示功能
        ///
        /// # 泛型参数
        /// - T: 链表节点中存储的数据类型
        ///
        /// # 参数
        /// - `f`: 一个可变的 `Formatter` 引用，用于控制格式化输出
        ///
        /// # 返回值
        /// 返回一个 `fmt::Result` 类型值，表示格式化操作是否成功
        ///
        /// # 操作逻辑
        /// 使用 `debug_list` 格式化工具，通过 `iter()` 遍历链表元素，生成类似 `Vec` 的调试输出格式

        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_list().entries(self.iter()).finish()
        }

        // ... existing code ...
    }

    // 清理资源
    impl<T> Drop for DoublyLinkedList<T> {
        // ... existing code ...

        /// 实现链表的析构逻辑，用于安全地释放所有节点占用的资源
        ///
        /// # 泛型参数
        /// - T: 链表节点中存储的数据类型
        ///
        /// # 操作逻辑
        /// 通过不断移除头部节点直到链表为空，确保所有节点被正确析构
        /// 这会依次释放链表中每个节点的堆内存资源

        fn drop(&mut self) {
            while self.pop_front().is_some() {}
        }

        // ... existing code ...
    }

    // 克隆实现
    impl<T: Clone> Clone for DoublyLinkedList<T> {
        // ... existing code ...

        /// 创建链表的一个深拷贝副本
        ///
        /// # 泛型参数
        /// - T: 链表节点中存储的数据类型，必须实现 `Clone` trait
        ///
        /// # 返回值
        /// 返回一个新的 `DoublyLinkedList<T>` 实例，包含原链表所有元素的副本
        ///
        /// # 操作逻辑
        /// 1. 通过 `iter()` 获取原链表的不可变迭代器
        /// 2. 使用 `cloned()` 方法克隆每个元素的值
        /// 3. 通过 `collect()` 构造出一个新的链表

        fn clone(&self) -> Self {
            self.iter().cloned().collect()
        }

        // ... existing code ...
    }

    // 默认实现
    impl<T> Default for DoublyLinkedList<T> {
        // ... existing code ...

        /// 创建一个默认初始化的空链表实例
        ///
        /// # 泛型参数
        /// - T: 链表节点中存储的数据类型
        ///
        /// # 返回值
        /// 返回一个默认的 `Self` 实例，即一个空的双向链表

        fn default() -> Self {
            Self::new()
        }

        // ... existing code ...
    }
}
pub mod c_list {
    // src/ffi.rs
    use std::os::raw::{c_void, c_int};
    use std::ptr;
    use crate::other_list::{DoublyLinkedList, Node};

    // 不透明指针类型，对C完全隐藏实现细节
    #[repr(C)]
    pub struct CDoublyLinkedList {
        inner: DoublyLinkedList<*mut c_void>,
    }

    // 迭代器结构，用于C端遍历
    #[repr(C)]
    pub struct CIterator {
        current: *mut Node<*mut c_void>,
    }

    // 错误码定义
    pub const DLL_SUCCESS: c_int = 0;
    pub const DLL_ERROR_NULL_PTR: c_int = -1;
    pub const DLL_ERROR_EMPTY: c_int = -2;
    pub const DLL_ERROR_OUT_OF_BOUNDS: c_int = -3;

    // ... existing code ...

/// 创建一个新的C语言接口可用的双向链表实例
///
/// 该函数用于在C语言环境中构造一个双向链表对象，通过将Rust的DoublyLinkedList
/// 包装在CDoublyLinkedList结构体中，并将其转换为裸指针返回。
///
/// 返回值:
/// - 返回指向CDoublyLinkedList实例的裸指针，该实例内部包含一个初始化的双向链表。
#[unsafe(no_mangle)]
pub extern "C" fn dll_new() -> *mut CDoublyLinkedList {
    Box::into_raw(Box::new(CDoublyLinkedList {
        inner: DoublyLinkedList::new(),
    }))
}

// ... existing code ...


    // ... existing code ...

    /// 释放由[dll_new]创建的双向链表实例
    ///
    /// 该函数用于释放由[dll_new]函数分配的双向链表资源。该函数接受一个指向
    /// CDoublyLinkedList结构体的指针，并将其转换回Box以触发内存释放。
    ///
    /// 参数:
    /// - `list`: 指向CDoublyLinkedList实例的裸指针，该实例将被释放。
    ///
    /// 注意:
    /// - 该函数内部检查指针是否为空，若为空则不执行任何操作。
    /// - 该函数使用`unsafe`块来执行从裸指针恢复Box的操作，这是必要的，
    ///   因为该函数负责释放由Box::into_raw分配的内存。
    #[unsafe(no_mangle)]
    pub extern "C" fn dll_free(list: *mut CDoublyLinkedList) {
        if !list.is_null() {
            unsafe { let _ = Box::from_raw(list); }
        }
    }

    // ... existing code ...

    // ... existing code ...

    /// 获取双向链表的当前元素数量
    ///
    /// 该函数用于获取由[dll_new]创建的双向链表中的元素数量。该函数接受一个指向
    /// CDoublyLinkedList结构体的常量指针，并返回内部双向链表的长度。
    ///
    /// 参数:
    /// - `list`: 指向CDoublyLinkedList实例的常量裸指针，用于查询链表长度。
    ///
    /// 返回值:
    /// - 返回双向链表中元素的数量。如果输入指针为空，则返回0。
    ///
    /// 注意:
    /// - 该函数内部检查指针是否为空，若为空则返回0。
    /// - 该函数使用`unsafe`块来访问链表的内部结构，这是必要的，
    ///   因为它需要直接操作裸指针以获取链表长度。
    #[unsafe(no_mangle)]
    pub extern "C" fn dll_len(list: *const CDoublyLinkedList) -> usize {
        if list.is_null() { 0 } else { unsafe { (*list).inner.len() } }
    }

    // ... existing code ...

    // ... existing code ...

    /// 检查双向链表是否为空
    ///
    /// 该函数用于检查由[dll_new]创建的双向链表是否为空。该函数接受一个指向
    /// CDoublyLinkedList结构体的常量指针，并返回表示链表是否为空的整数值。
    ///
    /// 参数:
    /// - `list`: 指向CDoublyLinkedList实例的常量裸指针，用于检查链表是否为空。
    ///
    /// 返回值:
    /// - 如果输入指针为空，返回`DLL_ERROR_NULL_PTR`错误码；
    /// - 否则返回一个`c_int`值，表示链表是否为空（1表示空，0表示非空）。
    ///
    /// 注意:
    /// - 该函数内部检查指针是否为空，若为空则返回错误码。
    /// - 该函数使用`unsafe`块来访问链表的内部结构，这是必要的，
    ///   因为它需要直接操作裸指针以检查链表是否为空。
    #[unsafe(no_mangle)]
    pub extern "C" fn dll_is_empty(list: *const CDoublyLinkedList) -> c_int {
        if list.is_null() {
            DLL_ERROR_NULL_PTR
        } else {
            unsafe { (*list).inner.is_empty() as c_int }
        }
    }

    // ... existing code ...

    // ... existing code ...

    /// 在双向链表的前端插入一个元素
    ///
    /// 该函数用于在由[dll_new]创建的双向链表的前端插入一个元素。该函数接受一个指向
    /// CDoublyLinkedList结构体的可变指针和一个指向要插入数据的指针，并在链表前端插入该数据。
    ///
    /// 参数:
    /// - `list`: 指向CDoublyLinkedList实例的可变裸指针，用于插入元素。
    /// - `data`: 指向要插入数据的裸指针。
    ///
    /// 返回值:
    /// - 如果输入指针`list`为空，返回`DLL_ERROR_NULL_PTR`错误码；
    /// - 否则返回`DLL_SUCCESS`表示插入操作成功。
    ///
    /// 注意:
    /// - 该函数内部检查指针是否为空，若为空则返回错误码。
    /// - 该函数使用`unsafe`块来操作链表的内部结构，这是必要的，
    ///   因为它需要直接操作裸指针以插入数据到链表前端。
    #[unsafe(no_mangle)]
    pub extern "C" fn dll_push_front(list: *mut CDoublyLinkedList, data: *mut c_void) -> c_int {
        if list.is_null() {
            return DLL_ERROR_NULL_PTR;
        }

        unsafe {
            (*list).inner.push_front(data);
        }
        DLL_SUCCESS
    }

    // ... existing code ...

    // ... existing code ...

    /// 在双向链表的尾端插入一个元素
    ///
    /// 该函数用于在由[dll_new]创建的双向链表的尾端插入一个元素。该函数接受一个指向
    /// CDoublyLinkedList结构体的可变指针和一个指向要插入数据的指针，并在链表尾端插入该数据。
    ///
    /// 参数:
    /// - `list`: 指向CDoublyLinkedList实例的可变裸指针，用于插入元素。
    /// - `data`: 指向要插入数据的裸指针。
    ///
    /// 返回值:
    /// - 如果输入指针`list`为空，返回`DLL_ERROR_NULL_PTR`错误码；
    /// - 否则返回`DLL_SUCCESS`表示插入操作成功。
    ///
    /// 注意:
    /// - 该函数内部检查指针是否为空，若为空则返回错误码。
    /// - 该函数使用`unsafe`块来操作链表的内部结构，这是必要的，
    ///   因为它需要直接操作裸指针以插入数据到链表尾端。
    #[unsafe(no_mangle)]
    pub extern "C" fn dll_push_back(list: *mut CDoublyLinkedList, data: *mut c_void) -> c_int {
        if list.is_null() {
            return DLL_ERROR_NULL_PTR;
        }

        unsafe {
            (*list).inner.push_back(data);
        }
        DLL_SUCCESS
    }

    // ... existing code ...

    // ... existing code ...

    /// 从双向链表的前端移除并返回一个元素
    ///
    /// 该函数用于从由[dll_new]创建的双向链表的前端移除并返回一个元素。
    /// 该函数接受一个指向CDoublyLinkedList结构体的可变指针，并返回指向被移除数据的指针。
    ///
    /// 参数:
    /// - `list`: 指向CDoublyLinkedList实例的可变裸指针，用于移除元素。
    ///
    /// 返回值:
    /// - 如果输入指针`list`为空，返回空指针`ptr::null_mut()`；
    /// - 否则返回指向被移除数据的裸指针；
    /// - 如果链表为空，也返回空指针。
    ///
    /// 注意:
    /// - 该函数内部检查指针是否为空，若为空则返回空指针。
    /// - 该函数使用`unsafe`块来操作链表的内部结构，这是必要的，
    ///   因为它需要直接操作裸指针以从链表前端移除数据。
    #[unsafe(no_mangle)]
    pub extern "C" fn dll_pop_front(list: *mut CDoublyLinkedList) -> *mut c_void {
        if list.is_null() {
            return ptr::null_mut();
        }

        unsafe {
            (*list).inner.pop_front().unwrap_or(ptr::null_mut())
        }
    }

    // ... existing code ...

    // ... existing code ...

    /// 从双向链表的尾端移除并返回一个元素
    ///
    /// 该函数用于从由[dll_new]创建的双向链表的尾端移除并返回一个元素。
    /// 该函数接受一个指向CDoublyLinkedList结构体的可变指针，并返回指向被移除数据的指针。
    ///
    /// 参数:
    /// - `list`: 指向CDoublyLinkedList实例的可变裸指针，用于移除元素。
    ///
    /// 返回值:
    /// - 如果输入指针`list`为空，返回空指针`ptr::null_mut()`；
    /// - 否则返回指向被移除数据的裸指针；
    /// - 如果链表为空，也返回空指针。
    ///
    /// 注意:
    /// - 该函数内部检查指针是否为空，若为空则返回空指针。
    /// - 该函数使用`unsafe`块来操作链表的内部结构，这是必要的，
    ///   因为它需要直接操作裸指针以从链表尾端移除数据。
    #[unsafe(no_mangle)]
    pub extern "C" fn dll_pop_back(list: *mut CDoublyLinkedList) -> *mut c_void {
        if list.is_null() {
            return ptr::null_mut();
        }

        unsafe {
            (*list).inner.pop_back().unwrap_or(ptr::null_mut())
        }
    }

    // ... existing code ...

    // ... existing code ...

    /// 获取双向链表前端元素的副本
    ///
    /// 该函数用于获取由[dll_new]创建的双向链表前端元素的副本。
    /// 该函数接受一个指向CDoublyLinkedList结构体的常量指针，并返回指向前端元素副本的指针。
    ///
    /// 参数:
    /// - `list`: 指向CDoublyLinkedList实例的常量裸指针，用于获取前端元素。
    ///
    /// 返回值:
    /// - 如果输入指针`list`为空，返回空指针`ptr::null_mut()`；
    /// - 否则返回指向前端元素副本的裸指针；
    /// - 如果链表为空，也返回空指针。
    ///
    /// 注意:
    /// - 该函数内部检查指针是否为空，若为空则返回空指针。
    /// - 该函数使用`unsafe`块来访问链表的内部结构，这是必要的，
    ///   因为它需要直接操作裸指针以获取链表前端元素。
    #[unsafe(no_mangle)]
    pub extern "C" fn dll_front(list: *const CDoublyLinkedList) -> *mut c_void {
        if list.is_null() {
            return ptr::null_mut();
        }

        unsafe {
            (*list).inner.front().copied().unwrap_or(ptr::null_mut())
        }
    }

    // ... existing code ...

    // ... existing code ...

    /// 获取双向链表尾端元素的副本
    ///
    /// 该函数用于获取由[dll_new]创建的双向链表尾端元素的副本。
    /// 该函数接受一个指向CDoublyLinkedList结构体的常量指针，并返回指向尾端元素副本的指针。
    ///
    /// 参数:
    /// - `list`: 指向CDoublyLinkedList实例的常量裸指针，用于获取尾端元素。
    ///
    /// 返回值:
    /// - 如果输入指针`list`为空，返回空指针`ptr::null_mut()`；
    /// - 否则返回指向尾端元素副本的裸指针；
    /// - 如果链表为空，也返回空指针。
    ///
    /// 注意:
    /// - 该函数内部检查指针是否为空，若为空则返回空指针。
    /// - 该函数使用`unsafe`块来访问链表的内部结构，这是必要的，
    ///   因为它需要直接操作裸指针以获取链表尾端元素。
    #[unsafe(no_mangle)]
    pub extern "C" fn dll_back(list: *const CDoublyLinkedList) -> *mut c_void {
        if list.is_null() {
            return ptr::null_mut();
        }

        unsafe {
            (*list).inner.back().copied().unwrap_or(ptr::null_mut())
        }
    }

    // ... existing code ...


    // ... existing code ...

    /// 获取双向链表前端元素的可变指针
    ///
    /// 该函数用于获取由[dll_new]创建的双向链表前端元素的原始指针。
    /// 该函数接受一个指向CDoublyLinkedList结构体的可变指针，并返回指向前端元素的可变裸指针。
    ///
    /// 参数:
    /// - `list`: 指向CDoublyLinkedList实例的可变裸指针，用于获取前端元素指针。
    ///
    /// 返回值:
    /// - 如果输入指针`list`为空或链表为空，返回空指针`ptr::null_mut()`；
    /// - 否则返回指向前端元素的可变裸指针。
    ///
    /// 注意:
    /// - 该函数使用`unsafe`块来操作链表的内部结构，这是必要的，
    ///   因为它需要直接操作裸指针以获取链表前端元素的地址。
    #[unsafe(no_mangle)]
    pub extern "C" fn dll_front_mut(list: *mut CDoublyLinkedList) -> *mut c_void {
        unsafe {
            list.as_mut()
                .and_then(|list| list.inner.front_mut())
                .map(|ptr_ref| ptr_ref as *mut _ as *mut c_void)
                .unwrap_or(ptr::null_mut())
        }
    }

    // ... existing code ...

    // ... existing code ...

    /// 获取双向链表尾端元素的可变指针
    ///
    /// 该函数用于获取由[dll_new]创建的双向链表尾端元素的原始指针。
    /// 该函数接受一个指向CDoublyLinkedList结构体的可变指针，并返回指向尾端元素的可变裸指针。
    ///
    /// 参数:
    /// - `list`: 指向CDoublyLinkedList实例的可变裸指针，用于获取尾端元素指针。
    ///
    /// 返回值:
    /// - 如果输入指针`list`为空或链表为空，返回空指针`ptr::null_mut()`；
    /// - 否则返回指向尾端元素的可变裸指针。
    ///
    /// 注意:
    /// - 该函数使用`unsafe`块来操作链表的内部结构，这是必要的，
    ///   因为它需要直接操作裸指针以获取链表尾端元素的地址。
    #[unsafe(no_mangle)]
    pub extern "C" fn dll_back_mut(list: *mut CDoublyLinkedList) -> *mut c_void {
        unsafe {
            list.as_mut()
                .and_then(|list| list.inner.back_mut())
                .map(|ptr_ref| ptr_ref as *mut *mut c_void as *mut c_void)
                .unwrap_or(ptr::null_mut())
        }
    }

    // ... existing code ...

    // ... existing code ...

    /// 获取双向链表的C语言接口兼容迭代器
    ///
    /// 该函数用于获取一个指向CIterator结构体的裸指针，该结构体可用于遍历
    /// 由[dll_new]创建的双向链表。迭代器初始化时指向链表的第一个节点。
    ///
    /// 参数:
    /// - `list`: 指向CDoublyLinkedList实例的可变裸指针，用于创建迭代器。
    ///
    /// 返回值:
    /// - 如果输入指针`list`为空，返回空指针`ptr::null_mut()`；
    /// - 否则返回指向CIterator结构体的裸指针，该结构体可用于遍历链表。
    ///
    /// 注意:
    /// - 该函数内部检查指针是否为空，若为空则直接返回空指针。
    /// - 该函数使用`unsafe`块来访问链表的内部结构，这是必要的，
    ///   因为它需要直接操作裸指针以获取链表的头节点。
    #[unsafe(no_mangle)]
    pub extern "C" fn dll_into_iter(list: *mut CDoublyLinkedList) -> *mut CIterator {
        if list.is_null() {
            return ptr::null_mut();
        }

        Box::into_raw(Box::new(CIterator {
            current: unsafe { (*list).inner.head },
        }))
    }

    // ... existing code ...

    // ... existing code ...

    /// 获取迭代器当前位置的元素并移动到下一个节点
    ///
    /// 该函数用于获取迭代器当前指向的元素，并将迭代器移动到下一个节点。
    /// 该函数接受一个指向CIterator结构体的可变指针，并返回当前元素的裸指针。
    ///
    /// 参数:
    /// - `iter`: 指向CIterator实例的可变裸指针，用于遍历双向链表。
    ///
    /// 返回值:
    /// - 如果输入指针`iter`为空或迭代器已到达末尾，返回空指针`ptr::null_mut()`；
    /// - 否则返回当前节点中存储的数据的裸指针。
    ///
    /// 注意:
    /// - 该函数内部检查指针是否为空，若为空则直接返回空指针。
    /// - 该函数使用`unsafe`块来操作迭代器的内部结构，这是必要的，
    ///   因为它需要直接访问裸指针以获取当前节点数据并更新迭代器状态。
    #[unsafe(no_mangle)]
    pub extern "C" fn dll_iter_next(iter: *mut CIterator) -> *mut c_void {
        if iter.is_null() {
            return ptr::null_mut();
        }

        unsafe {
            if (*iter).current.is_null() {
                // 迭代器已经到达末尾，返回空指针
                ptr::null_mut()
            } else {
                // 确保 current 指针指向有效节点
                let current_node = &*(*iter).current;
                let data = current_node.data;
                (*iter).current = current_node.next;
                data
            }
        }
    }

    // ... existing code ...

    // ... existing code ...

    // ... existing code ...

    /// 释放由[dll_into_iter]创建的迭代器
    ///
    /// 该函数用于释放由[dll_into_iter]函数分配的CIterator结构体。
    /// 该函数接受一个指向CIterator结构体的裸指针，并将其转换回Box以触发内存释放。
    ///
    /// 参数:
    /// - `iter`: 指向CIterator实例的裸指针，该实例将被释放。
    ///
    /// 注意:
    /// - 该函数内部检查指针是否为空，若为空则不执行任何操作。
    /// - 该函数使用`unsafe`块来执行从裸指针恢复Box的操作，这是必要的，
    ///   因为该函数负责释放由Box::into_raw分配的内存。
    #[unsafe(no_mangle)]
    pub extern "C" fn dll_iter_free(iter: *mut CIterator) {
        if !iter.is_null() {
            unsafe { let _ = Box::from_raw(iter); }
        }
    }

    // ... existing code ...
}
