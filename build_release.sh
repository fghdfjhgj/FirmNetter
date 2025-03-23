#!/bin/bash

# 定义一个函数来执行构建逻辑
build_project() {
    echo "Waiting 2 seconds before starting the build..."
    sleep 2


    # 构建项目
    echo "Starting build with all features enabled..."
    cargo build --release --all-features

    # 检查构建结果
    if [ $? -ne 0 ]; then
        echo "Build failed! Check the error logs above for details."
        exit 1
    fi

    echo "Build succeeded!"
}

# 调用构建函数
build_project