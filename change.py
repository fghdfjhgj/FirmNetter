import re
import shutil
import os

def modify_header_file(input_path, output_path):
    # 读取原始头文件内容
    with open(input_path, 'r', encoding='utf-8') as file:
        content = file.read()

    # 定义正则表达式来匹配 Database 结构体
    pattern = re.compile(r'(typedef\s+struct\s+Database\s*\{(.*?)\}\s*Database;)', re.DOTALL)

    def replace_callback(match):
        original_struct = match.group(0)
        # 替换 conn 字段为 void* conn;
        modified_struct = re.sub(r'struct\s+Arc_Mutex_PgConnection\s+conn;', 'void* conn;', original_struct, flags=re.DOTALL)
        return modified_struct

    # 执行替换
    new_content = pattern.sub(replace_callback, content)

    # 将修改后的内容写入新的头文件（临时文件）
    temp_output_path = output_path + ".tmp"
    with open(temp_output_path, 'w', encoding='utf-8') as file:
        file.write(new_content)

    # 替换原文件
    shutil.move(temp_output_path, output_path)
    print(f"Modified file saved to {output_path}")

# 输入输出路径
input_path = 'target/release/FirmNetter.h'  # 原始头文件路径
output_path = 'target/release/FirmNetter.h'  # 修改后的头文件路径

modify_header_file(input_path, output_path)