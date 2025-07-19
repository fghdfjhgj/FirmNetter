#!/bin/bash

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 全局配置
ANDROID_API_LEVEL=21
OPENSSL_ROOT="/home/sunyuze/work/openssla64"
CARGO_NDK_VERSION="2.8.0"

# 完整支持的目标列表（保留所有平台）
SUPPORTED_TARGETS=(
    "aarch64-linux-android"     # 仅支持 aarch64 的 Android
    "x86_64-unknown-linux-gnu"
    "x86_64-pc-windows-gnu"
    "aarch64-unknown-linux-gnu"
    "x86_64-apple-darwin"
    "aarch64-apple-darwin"
)

# 日志函数
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 检查命令是否存在
check_command() {
    if ! command -v "$1" &> /dev/null; then
        log_error "Required command not found: $1"
        return 1
    fi
}

# 增强版NDK检测
detect_ndk() {
    log_info "正在检测Android NDK..."

    local ndk_paths=(
        "$HOME/Android/Sdk/ndk/*"
        "/opt/android-ndk*"
        "/usr/local/lib/android/ndk/*"
        "$HOME/Library/Android/sdk/ndk/*"
        "/usr/local/share/android-ndk"
    )

    local latest_ndk=""
    for path in "${ndk_paths[@]}"; do
        for expanded_path in $path; do
            if [[ -d "$expanded_path" && -d "$expanded_path/toolchains/llvm" ]]; then
                if [[ -z "$latest_ndk" || "$expanded_path" > "$latest_ndk" ]]; then
                    latest_ndk="$expanded_path"
                fi
            fi
        done
    done

    if [[ -n "$latest_ndk" ]]; then
        export ANDROID_NDK_HOME="$latest_ndk"
        log_info "检测到NDK: $ANDROID_NDK_HOME"
        return 0
    fi

    log_error "未找到Android NDK"
    echo -e "请通过以下方式安装:"
    echo -e "1. Android Studio → SDK Manager → SDK Tools → 勾选'NDK (Side by side)'"
    echo -e "2. 或手动下载: https://developer.android.com/ndk/downloads"
    echo -e "\n安装后可以设置环境变量:"
    echo -e "export ANDROID_NDK_HOME=/path/to/your/ndk"
    return 1
}

# 设置Android构建环境（仅 aarch64）
setup_android_env() {
    log_info "配置Android构建环境(aarch64)..."

    # 检测 NDK
    if [[ -z "$ANDROID_NDK_HOME" ]]; then
        detect_ndk || return 1
    fi

    # 设置 OpenSSL 环境变量（仅 aarch64）
    export OPENSSL_STATIC=1
    export OPENSSL_DIR="$OPENSSL_ROOT"   # ✅ 修改点：指定架构目录
    export OPENSSL_LIB_DIR="$OPENSSL_DIR/lib"
    export OPENSSL_INCLUDE_DIR="$OPENSSL_DIR/include"

    # 验证 OpenSSL 库是否存在
    local required_libs=("libcrypto.a" "libssl.a")
    for lib in "${required_libs[@]}"; do
        if [[ ! -f "$OPENSSL_LIB_DIR/$lib" ]]; then
            log_error "缺少OpenSSL静态库: $OPENSSL_LIB_DIR/$lib"
            return 1
        fi
    done

    log_info "Android环境配置完成"
    return 0
}

# 安装Rust目标工具链
install_rust_target() {
    local target=$1
    log_info "检查Rust目标工具链: $target"

    if ! rustup target list | grep -q "$target.*installed"; then
        log_warn "工具链未安装，正在安装..."
        if ! rustup target add "$target"; then
            log_error "无法安装目标工具链: $target"
            return 1
        fi
    fi
    return 0
}

# 安装cargo-ndk
install_cargo_ndk() {
    log_info "检查cargo-ndk..."
    if ! command -v cargo-ndk &> /dev/null; then
        log_warn "cargo-ndk未安装，正在安装..."
        if ! cargo install cargo-ndk --version "$CARGO_NDK_VERSION"; then
            log_error "cargo-ndk安装失败"
            return 1
        fi
    fi
    return 0
}

# 生成 C/C++ 头文件
generate_header() {
    log_info "正在生成 C/C++ 头文件..."

    # 清理旧的 OUT_DIR
    rm -rf "./target/release"

    # 触发 build.rs 生成头文件
    if ! cargo build --release; then
        log_error "运行 build.rs 失败"
        return 1
    fi

    local header_path="./target/release/FirmNetter.h"

    if [[ ! -f "$header_path" ]]; then
        log_error "未找到头文件: $header_path"
        return 1
    fi

    log_info "✅ 头文件已生成: $header_path"
    return 0
}

# 构建Android目标
build_android_target() {
    log_info "开始使用 cargo-ndk 构建 Android 动态库..."

    if ! setup_android_env; then
        return 1
    fi

    if ! install_cargo_ndk; then
        return 1
    fi

    local build_cmd=(
        "cargo" "ndk"
        "-t" "aarch64-linux-android"
        "-p" "$ANDROID_API_LEVEL"
        "build"
        "--release"
    )

    if [[ "$VERBOSE" == "true" ]]; then
        build_cmd+=("--verbose")
    fi

    log_info "执行构建命令: ${build_cmd[*]}"
    if ! "${build_cmd[@]}"; then
        log_error "Android构建失败"
        return 1
    fi

    return 0
}

# 构建普通目标
build_normal_target() {
    local target=$1
    log_info "开始构建目标: $target"

    local build_cmd=(
        "cargo" "build"
        "--target" "$target"
        "--release"
    )

    if [[ "$VERBOSE" == "true" ]]; then
        build_cmd+=("--verbose")
    fi

    log_info "执行构建命令: ${build_cmd[*]}"
    if ! "${build_cmd[@]}"; then
        log_error "$target 构建失败"
        return 1
    fi

    return 0
}

# 验证构建结果（动态库 + 头文件）
verify_build_result() {
    local target=$1
    local target_dir="./target/$target/release"
    local include_dir="$target_dir/include"

    local binary_path=""
    if [[ "$target" == "aarch64-linux-android" ]]; then
        binary_path=$(find "$target_dir/" -type f -name "libfirm_netter.so" | head -n 1)
    elif [[ "$target" == *"windows"* ]]; then
        binary_path="$target_dir/firm_netter.dll"
    elif [[ "$target" == *"apple"* ]]; then
        binary_path="$target_dir/libfirm_netter.dylib"
    else
        binary_path="$target_dir/libfirm_netter.so"
    fi

    if [[ ! -f "$binary_path" ]]; then
        log_error "未找到输出文件: $binary_path"
        return 1
    fi

    if [[ ! -f "$include_dir/FirmNetter.h" ]]; then
        log_error "未找到头文件: $include_dir/FirmNetter.h"
        return 1
    fi

    log_info "构建成功! 文件路径:"
    log_info "  动态库: $binary_path"
    log_info "  头文件: $include_dir/FirmNetter.h"

    return 0
}

# 主构建函数
build_target() {
    local target=$1

    echo -e "\n${GREEN}=== 开始构建: $target ===${NC}"

    # 只在第一次生成头文件
    if [[ ! -f "./target/release/FirmNetter.h" ]]; then
        if ! generate_header; then
            return 1
        fi
    fi

    if ! install_rust_target "$target"; then
        return 1
    fi

    case "$target" in
        "aarch64-linux-android")
            if ! build_android_target; then
                return 1
            fi
            ;;
        *)
            if ! build_normal_target "$target"; then
                return 1
            fi
            ;;
    esac

    # 复制头文件到目标目录
    local target_dir="./target/$target/release"
    mkdir -p "$target_dir/include"
    cp "./target/release/FirmNetter.h" "$target_dir/include/"

    if ! verify_build_result "$target"; then
        return 1
    fi

    return 0
}

# 构建所有目标
build_all_targets() {
    local failed_targets=()
    local success_count=0

    for target in "${SUPPORTED_TARGETS[@]}"; do
        if build_target "$target"; then
            ((success_count++))
        else
            failed_targets+=("$target")
        fi
    done

    echo -e "\n${GREEN}构建完成!${NC}"
    echo "成功构建: $success_count/${#SUPPORTED_TARGETS[@]}"

    if [[ ${#failed_targets[@]} -gt 0 ]]; then
        echo -e "${RED}失败的目标:${NC}"
        for target in "${failed_targets[@]}"; do
            echo "  - $target"
        done
        return 1
    fi

    return 0
}

# 交互式选择目标
interactive_select() {
    PS3="请选择目标架构(1-${#SUPPORTED_TARGETS[@]}): "
    select target in "${SUPPORTED_TARGETS[@]}" "退出"; do
        case "$target" in
            "退出")
                exit 0
                ;;
            *)
                if [[ -n "$target" ]]; then
                    build_target "$target"
                    break
                else
                    log_error "无效选择"
                fi
                ;;
        esac
    done
}

# 参数解析
TARGET=""
BUILD_ALL=false
VERBOSE=false
LIST_TARGETS=false

while [[ $# -gt 0 ]]; do
    case "$1" in
        -t|--target)
            TARGET=$2
            shift 2
            ;;
        -a|--all)
            BUILD_ALL=true
            shift
            ;;
        -l|--list-targets)
            LIST_TARGETS=true
            shift
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -h|--help)
            show_help
            exit 0
            ;;
        *)
            log_error "未知选项: $1"
            show_help
            exit 1
            ;;
    esac
done

# 检查基本命令
check_command "rustup" || exit 1
check_command "cargo" || exit 1

# 列出目标
if [[ "$LIST_TARGETS" == "true" ]]; then
    echo -e "${YELLOW}支持的目标架构:${NC}"
    for target in "${SUPPORTED_TARGETS[@]}"; do
        echo "  - $target"
    done
    exit 0
fi

# 构建逻辑
if [[ "$BUILD_ALL" == "true" ]]; then
    build_all_targets
elif [[ -n "$TARGET" ]]; then
    if ! printf '%s\n' "${SUPPORTED_TARGETS[@]}" | grep -q "^$TARGET$"; then
        log_error "不支持的目标架构: $TARGET"
        show_help
        exit 1
    fi
    build_target "$TARGET"
else
    interactive_select
fi