#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

#[cfg(feature = "axstd")]
use axstd::println;

const PLASH_START: usize = 0x22000000;

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

#[inline]
fn bytes_to_usize(bytes: &[u8]) -> usize {
    usize::from_be_bytes(bytes.try_into().unwrap())
}

#[inline]
fn bytes_to_u32(bytes: &[u8]) -> u32 {
    u32::from_be_bytes(bytes.try_into().unwrap())
}