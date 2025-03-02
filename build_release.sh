#!/bin/bash

# 获取当前脚本所在目录
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

# 检查操作系统类型
OS_TYPE=$(uname)

install_dependencies() {
    case $OS_TYPE in
        Linux)
            # 检查包管理器类型
            if command -v dpkg > /dev/null 2>&1; then
                echo "Detected Debian-based system."
                install_debian_based
            elif command -v rpm > /dev/null 2>&1; then
                echo "Detected Red Hat-based system."
                install_redhat_based
            else
                echo "Unsupported Linux distribution. Using generic method to check dependencies."
                check_generic
            fi
            ;;
        Darwin)
            echo "Detected macOS system."
            install_macos
            ;;
        *)
            echo "Unsupported OS: $OS_TYPE. Please install the required dependencies manually."
            exit 1
            ;;
    esac
}

install_debian_based() {
    # 检查并安装 OpenSSL 库
    if ! dpkg -l | grep -qw libssl-dev; then
        echo "OpenSSL library is NOT installed. Installing..."
        sudo apt-get update && sudo apt-get install -y libssl-dev
    else
        echo "OpenSSL library is installed."
    fi

    # 检查并安装 PostgreSQL 库
    if ! dpkg -l | grep -qw libpq-dev; then
        echo "PostgreSQL library is NOT installed. Installing..."
        sudo apt-get update && sudo apt-get install -y libpq-dev
    else
        echo "PostgreSQL library is installed."
    fi
}

install_redhat_based() {
    # 检查并安装 OpenSSL 库
    if ! rpm -q openssl-devel > /dev/null 2>&1; then
        echo "OpenSSL library is NOT installed. Installing..."
        sudo yum check-update && sudo yum install -y openssl-devel
    else
        echo "OpenSSL library is installed."
    fi

    # 检查并安装 PostgreSQL 库
    if ! rpm -q postgresql-devel > /dev/null 2>&1; then
        echo "PostgreSQL library is NOT installed. Installing..."
        sudo yum check-update && sudo yum install -y postgresql-devel
    else
        echo "PostgreSQL library is installed."
    fi
}

check_generic() {
    # 使用 ldconfig 检查共享库
    if ! ldconfig -p | grep -w libssl > /dev/null 2>&1; then
        echo "OpenSSL library is NOT installed. Please install it manually."
        exit 1
    else
        echo "OpenSSL library is installed."
    fi

    if ! ldconfig -p | grep -w libpq > /dev/null 2>&1; then
        echo "PostgreSQL library is NOT installed. Please install it manually."
        exit 1
    else
        echo "PostgreSQL library is installed."
    fi
}

install_macos() {
    # 在 macOS 上使用 brew 来检查和安装依赖库
    if ! brew list openssl > /dev/null 2>&1; then
        echo "OpenSSL library is NOT installed. Installing..."
        brew install openssl
    else
        echo "OpenSSL library is installed."
    fi

    if ! brew list libpq > /dev/null 2>&1; then
        echo "PostgreSQL library is NOT installed. Installing..."
        brew install libpq
    else
        echo "PostgreSQL library is installed."
    fi
}

# 安装依赖项
install_dependencies

# 运行 cargo 构建命令
echo "Running cargo build --release..."
cargo build --release

if [ $? -ne 0 ]; then
    echo "Cargo build failed!"
    exit 1
fi

# 运行 Python 脚本修改生成的头文件
echo "Running Python script to modify the generated header file..."
python3 "$SCRIPT_DIR/change.py"

if [ $? -ne 0 ]; then
    echo "Python script failed!"
    exit 1
fi

echo "Build and modification completed successfully."