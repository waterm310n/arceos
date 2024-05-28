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
```rust
// App信息
struct AppInfo {
    app_addr:usize, // App代码地址
    app_size:usize, // App大小
}

impl AppInfo {
    fn new() -> Self{
        Self {
            app_addr:0,
            app_size:0,
        }
    }

    fn init(& mut self,app_addr:usize) {
        self.app_addr = app_addr;
        let header_size = 12; 
        let header = unsafe { core::slice::from_raw_parts(app_addr as *const u8, header_size) };
        let magic_number = bytes_to_u32(&header[..4]);
        if magic_number != 0xDEADBEAF{
            panic!("App header wants :0xDEADBEAF, real :{:x}",magic_number);
        }else {
            println!("App header start with magic_number {:x}",magic_number);
        }
        self.app_size = bytes_to_usize(&header[4..12]);
        println!("App size is {}",self.app_size);
    }

    fn print_content(&self) { //打印App内容
        let code = unsafe { core::slice::from_raw_parts((self.app_addr+12) as *const u8, self.app_size)};
        println!("App content: {:?}",code);
    }
    
}
```

# 练习2

首先生成两个加入了Header的bin，然后添加新的header
python脚本如下
```python
import struct

def combine_to_binary_file(input_file_path_list,output_file_path):
    """
    将多个应用程序组合在一起

    :param file_path: 二进制文件的路径
    """
    try:
        MULTI_APP_MAGIC_NUMBER = 0x89ABCDEF
        app_cnt = len(input_file_path_list)
        extend_header = struct.pack(">II",MULTI_APP_MAGIC_NUMBER,app_cnt)
        with open(output_file_path,"wb+") as output_file:
            output_file.write(extend_header)
            for input_file_path in input_file_path_list:
                # 以读写模式打开文件
                with open(input_file_path, 'rb+') as input_file:
                    # 读取二进制文件的内容
                    old_content = input_file.read()
                    # 写入新数据
                    output_file.write(old_content)
                    print(f"Successfully add an app")
    except FileNotFoundError:
        print(f"Error: File {output_file_path} not found.")
    except IOError as e:
        print(f"IO error occurred: {e}")

if __name__ == "__main__":
    input_file_path_list = ["./hello_app_with_header.bin","./ebreak_with_header.bin"]
    combine_to_binary_file(input_file_path_list,"multi_apps.bin")
```

shell脚本
```bash
dd if=/dev/zero of=./apps.bin bs=1M count=32
dd if=./multi_apps.bin of=./apps.bin conv=notrunc

mkdir -p ../arceos/payload
mv ./apps.bin ../arceos/payload/apps.bin
```

## 解析
```rust
// App信息
struct AppInfo {
    app_addr:usize, // App代码地址
    app_size:usize, // App大小
}

impl AppInfo {
    pub fn new() -> Self{
        Self {
            app_addr:0,
            app_size:0,
        }
    }

    pub fn init(& mut self,app_addr:usize) {
        self.app_addr = app_addr;
        let header_size = 12; 
        let header = unsafe { core::slice::from_raw_parts(app_addr as *const u8, header_size) };
        let magic_number = bytes_to_u32(&header[..4]);
        if magic_number != 0xDEADBEAF{
            panic!("App header wants :0xDEADBEAF, real :{:x}",magic_number);
        }else {
            println!("App header start with magic number {:x}",magic_number);
        }
        self.app_size = bytes_to_usize(&header[4..12]);
        println!("App size is {}",self.app_size);
    }

    fn print_content(&self) { //打印App内容
        let code = unsafe { core::slice::from_raw_parts((self.app_addr+12) as *const u8, self.app_size)};
        println!("App content: {:?}",code);
    }
    
    pub fn app_size(&self) -> usize {
        self.app_size
    }
}

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    let app_addr = PLASH_START;
    println!("Load payload ...");
    let magic_number = unsafe { core::slice::from_raw_parts(app_addr as *const u8, 4) };
    let magic_number = bytes_to_u32(magic_number);
    if 0x89abcdef == magic_number{//存在多个app
        println!("find multi app magic number {:X}",magic_number);
        let app_cnt = unsafe { core::slice::from_raw_parts((app_addr+4) as *const u8, 4) };
        let app_cnt = bytes_to_u32(app_cnt);
        println!("app cnt is {}",app_cnt);
        let mut cur_addr = app_addr+8;
        let mut app_info = AppInfo::new();
        for i in 0..app_cnt {
            println!("app {i} --------------------------------");
            app_info.init(cur_addr);
            app_info.print_content();
            cur_addr += 12+app_info.app_size();
        }
    }else{//单个APP
        let mut app_info = AppInfo::new();
        app_info.init(PLASH_START);
        app_info.print_content();
    }
    println!("Load payload ok!");
}
```