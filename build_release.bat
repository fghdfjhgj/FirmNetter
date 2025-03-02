@echo off

REM 获取当前批处理脚本所在目录
set SCRIPT_DIR=%~dp0

REM 运行 cargo 构建命令
echo Running cargo build --release...
cargo build --release

IF %ERRORLEVEL% NEQ 0 (
    echo Cargo build failed!
    exit /b 1
)

REM 运行 Python 脚本修改生成的头文件
echo Running Python script to modify the generated header file...
python "%SCRIPT_DIR%change.py"

IF %ERRORLEVEL% NEQ 0 (
    echo Python script failed!
    exit /b 1
)

echo Build and modification completed successfully.