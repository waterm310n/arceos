# 练习1

在helloapp下面创建script文件夹，然后加入下面的两个脚本

使用python向之前生成的可执行文件添加Image扩展头，具体的代码如下

```python
import struct
import argparse

# 单个可执行文件的Image扩展头
class ImageHeader(object):

    MAGIC_NUMBER = 0xDEADBEAF # 魔数，表示单个执行文件
    def __init__(self,content) -> None:
        self.size = len(content) # 可执行文件大小
        self.app_content = content
        # 可执行文件内容不能超过0xffffffff，否则不能用8个字节表示
        assert(len(content) <= 0xffffffffffffffff) 

    def to_bytes(self):
        return struct.pack('>IQ',self.MAGIC_NUMBER,self.size)+self.app_content
        

def add_image_header_to_binary_file(input_file_path,output_file_path):
    """
    向二进制文件添加image头

    :param file_path: 二进制文件的路径
    """
    try:
        # 以读写模式打开文件
        with open(input_file_path, 'rb+') as input_file:
            # 读取二进制文件的内容
            old_content = input_file.read()
            image_header = ImageHeader(old_content)
            # 写入新数据
            with open(output_file_path,'wb+') as output_file:
                output_file.write(image_header.to_bytes())
                print(f"Successfully add image header to excutable binary file.")
    except FileNotFoundError:
        print(f"Error: File {input_file_path} not found.")
    except IOError as e:
        print(f"IO error occurred: {e}")

if __name__ == "__main__":
    # 示例用法
    parser = argparse.ArgumentParser(description='argparse testing')
    parser.add_argument('--input_file_path','-i',type=str, default = "hello_app.bin",required=True)
    parser.add_argument('--output_file_path','-o',type=str, default= "hello_app_with_header.bin",required=True)
    args = parser.parse_args()
    input_file_path = args.input_file_path
    output_file_path = args.output_file_path
    add_image_header_to_binary_file(input_file_path,output_file_path)
```

使用shell脚本构建

```bash
#!/bin/bash
cargo build --target riscv64gc-unknown-none-elf --release

rust-objcopy --binary-architecture=riscv64 --strip-all -O binary target/riscv64gc-unknown-none-elf/release/hello_app ./hello_app.bin

python3 ./scripts/add_image_header.py -i ./hello_app.bin -o ./hello_app_with_header.bin

dd if=/dev/zero of=./apps.bin bs=1M count=32
dd if=./hello_app_with_header.bin of=./apps.bin conv=notrunc

mkdir -p ../arceos/payload
mv ./apps.bin ../arceos/payload/apps.bin

```

## 解析方法
```
```

# 练习2

## 解析