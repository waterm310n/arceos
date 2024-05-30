#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]
#![feature(asm_const)] 

#[cfg(feature = "axstd")]
use axstd::{print,println,process};

const SYS_HELLO: usize = 1;
const SYS_PUTCHAR: usize = 2;
#[cfg(feature = "axstd")]
const SYS_TERMINATE:usize = 3;

static mut ABI_TABLE: [usize; 16] = [0; 16];

fn register_abi(num: usize, handle: usize) {
    println!("SYS_CALL_NUM[{}] address is {:x} ",num,handle);
    unsafe { ABI_TABLE[num] = handle; }
}

fn register_multi_abi(){
    register_abi(SYS_HELLO, abi_hello as usize);
    register_abi(SYS_PUTCHAR, abi_putchar as usize);
    #[cfg(feature = "axstd")]
    register_abi(SYS_TERMINATE, abi_terminate as usize);
}

fn abi_hello() {
    println!("[ABI:Hello] Hello, Apps!");
}

fn abi_putchar(c: char) {
    print!("{c}");
    // println!("[ABI:Print] {c}");
}

#[cfg(feature = "axstd")]
fn abi_terminate(){
    println!("[ABI:Terminate]");
    process::exit(0);
}

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
            println!("App header start with legal magic number {:x}",magic_number);
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

    pub fn excute_code(&self) { //执行App

        let load_code = unsafe { core::slice::from_raw_parts((self.app_addr+12) as *const u8, self.app_size)};

        // app running aspace
        // SBI(0x80000000) -> App <- Kernel(0x80200000)
        // 0xffff_ffc0_0000_0000
        const RUN_START: usize = 0xffff_ffc0_8010_0000;
        let run_code = unsafe {
            core::slice::from_raw_parts_mut(RUN_START as *mut u8, self.app_size)
        };
        run_code.copy_from_slice(load_code);
        println!("run code {:?}; address [{:?}]", run_code, run_code.as_ptr());
        println!("Execute app ...");
        // execute app
        // 第一行将abi_table的地址存放到a7寄存器中
        // 第二行将app_code的地址存放到t2寄存器中
        // 第三行跳转到t2寄存器所指向的地址，然后执行代码
        unsafe { core::arch::asm!("
            la      a7, {abi_table} 
            li      t2, {run_start}
            jalr    t2
            ",
            run_start = const RUN_START,
            abi_table = sym ABI_TABLE,
        )}

    }
}

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    let app_addr = PLASH_START;
    register_multi_abi();
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
            app_info.excute_code();
            cur_addr += 12+app_info.app_size();
        }
    }else{//单个APP
        let mut app_info = AppInfo::new();
        app_info.init(PLASH_START);
        app_info.excute_code()
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