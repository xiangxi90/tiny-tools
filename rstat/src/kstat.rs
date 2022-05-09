use std::fmt::Display;

use libc::lstat;
pub use libc::{stat,statfs};
use chrono::prelude::*;

#[allow(dead_code)]
pub struct FileStat{
    filename: String,               //文件名
    stat: stat,                     //文件信息
    statfs:statfs,                  //文件系统信息

    blinker : bool,                 //该文件是否是一个链接
    oriname : String,               //链接指向的源文件的名字

    alive : bool,                   //该信息是否属于一个成功解析的文件
    err_info : String,              //错误信息

//    gstat : libc::group
}
// //readlink
// enum FileSystem{
//     File(stat),
//     FSystem(statfs),
// }


#[derive(Debug)]
pub struct OptionSelected{
    blink : bool,           //是否需要查看链接过去的文件还是链接本身 windows下不可用
    bterse : bool,          //是否需要简短输出
    bfilter : bool,         //是否需要规范化输出
    bfilesystem : bool,     //是否要查看文件系统   windows下不可用
    bmhelper : bool,        //是否要输出更多帮助
    files : Vec<String>,    //要查看的文件的名称
    output_inf : String,     //如果要规范化输出，其就是这个
    needflag : NeedFlag,    //用于判别输入时使用
}
/// 用于判断是否还需要规范化输入的状态
#[derive(Debug,Eq, PartialEq)]
pub enum NeedFlag{
    Default,
    Need,
    Over,
}
///返回时可能出现的警告
#[derive(Debug)]
#[allow(dead_code)]
pub enum StatError{
    InvalidOption,          //不能识别的选项
    WrongOption,            //错误的选项关系，例如-c不能与-t同时出现
    UnknowFile,             //未知的文件
    InvalidFormat,          //不能识别的规范输入
    MissingFormat,          //规范输入丢失
}

impl OptionSelected{
    /// 提供默认选项结构体
    pub fn default() -> Self{
        OptionSelected {
            blink : false,
            bterse: false,
            bfilter: false,
            bfilesystem: false,
            bmhelper: false,
            files: vec![],
            needflag : NeedFlag::Default,
            output_inf : String::new(),
        }
    }
    /// 解析命令行输入的内容   并从string容器中读取选项信息到当前选项结构体
    pub fn readoption(&mut self , args : Vec<String>) -> Result<(),StatError>{
        //let mut opt : OptionSelected=OptionSelected::default();
        let mlen = args.len();
        // 遍历传来的string，获取信息，默认最前面一个数组丢弃（直接从env拿来的第一个无用）
        for x in 1..mlen{
            // 获取相应位置上的string
            let iter = match args.get(x){
                Some(e) => e,
                None => "",
            };
            let length=iter.len();

            if length<=0 {
                continue;
            }
            // 判定现在的输入模式即拥有的选项等
            if self.needflag==NeedFlag::Need{
                self.output_inf.push_str(iter);
                self.needflag=NeedFlag::Over;
            }
            else if iter.starts_with("--"){
                if length<=2{
                    continue;
                }
                let longoption=&iter[2..];
                match longoption{
                    "dereference" => self.blink=true,
                    "format=FORMAT" => {
                        self.bfilter = true;
                        self.needflag=NeedFlag::Need;
                    },
                    "terse" => {
                        self.bterse = true;
                    },
                    "more-help" => self.bmhelper=true,
                    "file-system" => self.bfilesystem=true,
                    _ => return Err(StatError::InvalidOption),
                };
            }
            else if iter.starts_with('-'){
                for ch in iter.chars(){
                    match ch{
                        'L' => self.blink=true,
                        'c' => {
                            self.bfilter = true;
                            self.needflag=NeedFlag::Need;
                        },
                        't' => {
                            self.bterse = true;
                        },
                        'H' => self.bmhelper=true,
                        'f' => self.bfilesystem=true,
                        '-' => {},
                        _ => return Err(StatError::InvalidOption),
                    };
                }
            }
            else{
                // 这里一定要注意,在Rust下String末尾是不带'\0'的,但是c的数组末尾以这个结束,那么我们没这个的话就会解析不出来
                self.files.push(format!("{}\0",iter));
            }
        }
        Ok(())
    }

    /// 根据现在的选项结构体来获取相应的输出方式及信息获取
    pub fn output(&self) -> Result<(),StatError>{
        let mut filestats:Vec<FileStat> = Vec::new();

        // 如果同时需要简化输出和格式化输出,我们选择直接返回错误,即不允许这种错误
        if self.bterse==true&&self.bfilter==true{
            return Err(StatError::WrongOption);
        }

        // 获取本地时间偏移量,即所处时区
        let dt = Local::now().offset().to_string();
        //println!("dt: {}", dt.offset());

        
        // 遍历所有文件,并按照选项拿出我们需要的信息
        for iter in self.files.iter(){     
            if self.bfilesystem==true{
                filestats.push(get_file_statfs(iter.clone()));
            }
            else if self.blink==true{
                filestats.push(get_file_stat_link(iter.clone()));
            }
            else{
                filestats.push(get_file_stat(iter.clone()));
            }
        }

        //根据选项中的值来选择合适的输出方式
        if self.bterse==true{
            if self.bfilesystem{
                OptionSelected::output_with_terse_filesystem(filestats);
            }
            else{
                OptionSelected::output_with_terse_file(filestats);
            }
        }
        else if self.bfilter==true{
            OptionSelected::output_with_fileter(self.output_inf.clone(),filestats);
        }
        else {
            if self.bfilesystem{
                OptionSelected::output_normal_filesystem(filestats,&dt);
            }
            else{
                OptionSelected::output_normal_file(filestats,&dt);
            }
        }
        Ok(())
    }

    pub fn output_with_fileter(command : String , filestat : Vec<FileStat>){
        println!("{}:",command);
        for iter in filestat{
            print!("{}   ",iter.filename);
        }print!("\n");
    }

    pub fn output_with_terse_file(filestat: Vec<FileStat>){
        for iter in filestat{
            if iter.alive{
                let filestat =&iter.stat;
                println!("{} {} {} {:x} {} {} {:x} {} {} {} {} {} {} {} {} {}",
                    iter.filename,
                    filestat.st_size,filestat.st_blocks,
                    filestat.st_mode,filestat.st_uid,filestat.st_gid,
                    filestat.st_dev,
                    filestat.st_ino,filestat.st_nlink,0,0,
                    filestat.st_atime,filestat.st_mtime,filestat.st_ctime,
                    0,filestat.st_blksize,
                );
            }
            else{
                println!("stat: cannot stat '{}': {}",iter.filename,iter.err_info);
            }
        }
    }

    pub fn output_with_terse_filesystem(filestat: Vec<FileStat>){
        for iter in filestat{
            if iter.alive{
                let filestatfs =&iter.statfs;
                println!("{}  {:x} {} {} {} {} {} {} {}",
                    iter.filename,
                    11111111,   //该字段本该是filestatfs.f_fsid,但是目前使用的该libc库并没有给予该结构体任何display的方法，甚至内容都是pri的，难搞
                    filestatfs.f_namelen,filestatfs.f_bavail,
                    filestatfs.f_bsize,filestatfs.f_frsize,filestatfs.f_ffree,
                    filestatfs.f_bfree,filestatfs.f_type
                );
            }
            else{
                println!("stat: cannot stat '{}': {}",iter.filename,iter.err_info);
            }
        }
    }

    pub fn output_normal_file(filestat: Vec<FileStat> , offset : &str){
        println!("{}:","normal  file");
        for iter in filestat{
            if !iter.alive{
                println!("stat: cannot stat '{}': {}",iter.filename,iter.err_info);
                continue;
            }
            let filestat =&iter.stat;
            let ftype = FileType::get_file_type(filestat.st_mode);
            let fpermission = FilePermission::new(filestat.st_mode);

            if iter.blinker{
                println!("  File: {} -> {}",iter.filename,iter.oriname);
            }
            else{
                println!("  File: {}",iter.filename);
            }
            println!("  Size: {:<16}Blocks: {:<11}IO Block: {:<7}{}"
                ,filestat.st_size,filestat.st_blocks,filestat.st_blksize,
                ftype);
            println!("Device: {:<16}Inode: {:<12}Links: {}",
                format!("{:x}h/{}d",filestat.st_dev,filestat.st_dev),
                filestat.st_ino,filestat.st_nlink);
            println!("Access: ({}/{}{})  Uid: ({:>5}/{:>8})   Gid: ({:>5}/{:>8})",
                fpermission.output_num(),ftype.output_short(),fpermission.output_char(),
                filestat.st_uid,get_username_with_id(filestat.st_uid),
                filestat.st_gid,get_groupname_with_id(filestat.st_gid)
            );

            println!("Access: {} {}\nModify: {} {}\nChange: {} {}\nBirth: -",
                    get_time_utc2local(filestat.st_atime, filestat.st_atime_nsec),offset,
                    get_time_utc2local(filestat.st_mtime, filestat.st_mtime_nsec),offset,
                    get_time_utc2local(filestat.st_ctime, filestat.st_ctime_nsec),offset,
                );

        }print!("\n");
    }

    pub fn output_normal_filesystem(filestat: Vec<FileStat> , offset : &str){
        println!("{}:{}","normal  filesys",offset);
        for _iter in filestat{
            
        }print!("\n");
    }
}

fn get_file_stat(filename : String) -> FileStat{
    unsafe{
        let mut kstat = get_new_stat();
        let ret = lstat(filename.as_ptr().cast(),(&mut kstat) as *mut stat);
        if ret==-1{
            FileStat{
                filename: filename,
                stat: get_new_stat(),
                statfs: get_new_statfs(),
                blinker: false, 
                oriname: String::new(), 
                alive: false,
                err_info: std::io::Error::last_os_error().to_string(),
            }
        }
        else{
            let blnk=is_symbolic_link(kstat.st_mode);
            FileStat{
                filename: filename.clone(),
                stat: kstat,
                statfs: get_new_statfs(),
                blinker: blnk, 
                oriname: match blnk{
                    true => get_link_oriname(filename),
                    false => String::new(),
                } ,
                alive: true,
                err_info: String::new(),
            }
        }
    }
}

fn get_file_stat_link(filename: String) ->FileStat{
    unsafe{
        let mut klstat = get_new_stat();
        let ret = stat(filename.as_ptr().cast(),(&mut klstat) as *mut stat);
        if ret==-1{
            FileStat{
                filename: filename,
                stat: get_new_stat(),
                statfs: get_new_statfs(),
                blinker: false, 
                oriname: String::new(), 
                alive: false,
                err_info: std::io::Error::last_os_error().to_string(),
            }
        }
        else{
            FileStat{
                filename: filename.clone(),
                stat: klstat,
                statfs: get_new_statfs(),
                blinker: false, 
                oriname: String::new(),
                alive: true,
                err_info: String::new(),
            }
        }
    }
}


fn get_file_statfs(filename : String) ->FileStat{
    unsafe{
        let mut kstatfs = get_new_statfs();
        let ret = statfs(filename.as_ptr().cast(),(&mut kstatfs) as *mut statfs);
        if ret==-1{
            FileStat{
                filename: filename,
                stat: get_new_stat(),
                statfs: get_new_statfs(),
                blinker: false,
                oriname: String::new(), 
                alive: false,
                err_info: std::io::Error::last_os_error().to_string(),
            }
        }
        else{
            FileStat{
                filename: filename,
                stat: get_new_stat(),
                statfs: kstatfs,
                blinker: false, //实际上这里有可能提供的本身是一个链接，但是文件系统中无关是不是链接，故而无需考虑这个项的值
                oriname: String::new(), 
                alive: true,
                err_info: String::new(),
            }
        }
    }
}

fn get_link_oriname(linkname: String) -> String{
    unsafe{
        let mut namebuf : [i8; 50] = [0;50];
        let charbuf = namebuf.as_mut_ptr().cast();
        let ret=libc::readlink(linkname.as_ptr().cast(),charbuf,50);
        if ret!=-1 && ret<50{
            let res=std::ffi::CStr::from_ptr(charbuf);
            return format!("{}",res.to_str().unwrap());
        }
        println!("err");
        return linkname;
    }
}

// fn get_pwd_win() -> String{
//     let cwd=std::env::current_dir().unwrap();
//     cwd.into_os_string().into_string().unwrap()
// }

#[inline]
pub fn get_new_stat() -> stat{
    unsafe{
        return std::mem::zeroed::<libc::stat>();
    }
}

#[inline]
pub fn get_new_statfs() -> statfs{
    unsafe{
        return std::mem::zeroed::<libc::statfs>();
    }
}

#[allow(dead_code)]
pub fn get_dev_major(devno: u64) -> u32{
    unsafe{
        return libc::major(devno);
    }
}

#[allow(dead_code)]
pub fn get_dev_minor(devno: u64) -> u32{
    unsafe{
        return libc::minor(devno);
    }
}

/// Encoding of the file mode.  
/// #define	__S_IFMT	0170000	/* These bits determine file type.  
/// File types. 
/// #define	__S_IFDIR	0040000	 Directory. 
// #define	__S_IFCHR	0020000	 Character device.  
// #define	__S_IFBLK	0060000	 Block device. 
// #define	__S_IFREG	0100000	 Regular file.  
// #define	__S_IFIFO	0010000	 FIFO.  
// #define	__S_IFLNK	0120000	 Symbolic link.  
// #define	__S_IFSOCK	0140000	 Socket.  

/// Protection bits.  

/// #define	__S_ISUID	04000	 Set user ID on execution.  
/// #define	__S_ISGID	02000	 Set group ID on execution.  
/// #define	__S_ISVTX	01000	 Save swapped text after use (sticky).  
///#define	__S_IREAD	0400	 Read by owner. 
/// #define	__S_IWRITE	0200	 Write by owner.  
/// #define	__S_IEXEC	0100	 Execute by owner.  
/// 

type mode_t = u32;
pub const S_IFIFO: mode_t = 4096;
pub const S_IFCHR: mode_t = 8192;
pub const S_IFBLK: mode_t = 24576;
pub const S_IFDIR: mode_t = 16384;
pub const S_IFREG: mode_t = 32768;
pub const S_IFLNK: mode_t = 40960;
pub const S_IFSOCK: mode_t = 49152;
pub const S_IFMT: mode_t = 61440;

//pub const  FILEPERMIS : mode_t = 4095;
// pub const S_IRWXU: mode_t = 448;
// pub const S_IXUSR: mode_t = 64;
// pub const S_IWUSR: mode_t = 128;
// pub const S_IRUSR: mode_t = 256;
// pub const S_IRWXG: mode_t = 56;
// pub const S_IXGRP: mode_t = 8;
// pub const S_IWGRP: mode_t = 16;
// pub const S_IRGRP: mode_t = 32;
// pub const S_IRWXO: mode_t = 7;
// pub const S_IXOTH: mode_t = 1;
// pub const S_IWOTH: mode_t = 2;
// pub const S_IROTH: mode_t = 4;

#[inline]
pub fn is_symbolic_link(mode: u32) ->bool{
    if mode & (S_IFMT) == S_IFLNK{
        return true;
    }
    return false;
}



enum FileType{
    Block,
    Character,
    Directory,
    Link,
    Regular,
    Socket,
    Pipe,
    Unknown,
}

impl Display for FileType{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self{
            FileType::Block => write!(f,"block device"),
            FileType::Character => write!(f,"character device"),
            FileType::Directory => write!(f,"directory"),
            FileType::Link => write!(f,"symbolic link"),
            FileType::Regular => write!(f,"regular file"),
            FileType::Socket => write!(f,"socket"),
            FileType::Pipe => write!(f,"pipe"),
            FileType::Unknown => write!(f,"unknow type"), 
        }
    }
}

impl FileType{
    pub fn output_short(&self) -> char{
        match self{
            FileType::Block => 'b',
            FileType::Character => 'c',
            FileType::Directory => 'd',
            FileType::Link => 'l',
            FileType::Regular => '-',
            FileType::Socket => 's',
            FileType::Pipe => 'p',
            FileType::Unknown => 'r', 
        }
    }
}

impl  FileType {
    pub fn get_file_type(mode: u32) -> Self{
        let cmp = mode & (S_IFMT);
        //println!("{}  {} ",mode,cmp);
        if cmp==S_IFBLK{
            FileType::Block
        }else if cmp==S_IFCHR{
            FileType::Character
        }else if cmp==S_IFDIR{
            FileType::Directory
        }else if cmp==S_IFIFO{
            FileType::  Pipe
        }else if cmp==S_IFLNK{
            FileType::Link
        }else if cmp==S_IFREG{
            FileType::Regular
        }else if cmp==S_IFSOCK{
            FileType::Socket
        }else{
            FileType::Unknown
        }
    } 
}
struct FilePermission{
    //filetype: FileType, 
    owner : &'static str,    //rwx
    group : &'static str,
    other : &'static str,
    //special : &'static str,
    nowner : i8,
    ngroup : i8,
    nother : i8,
    nspe : i8,
}


impl FilePermission{
    pub fn new(mode: u32) -> Self{
        let group_per: &str;
        let ngroup : i8;
        match mode&448{
            448 => {group_per="rwx";ngroup=7},
            384 => {group_per="rw-";ngroup=6},
            320 => {group_per="r-x";ngroup=5},    
            256 => {group_per="r--";ngroup=4},
            192 => {group_per="-wx";ngroup=3},
            128 => {group_per="-w-";ngroup=2},
            64  => {group_per="--x";ngroup=1},
            0   => {group_per="---";ngroup=0},
            _   => {panic!("wrong input mode in group")}
        };
        let owner_per: &str;
        let nowner : i8;
        match mode&56{
            56 => {owner_per="rwx";nowner=7},
            48 => {owner_per="rw-";nowner=6},
            40 => {owner_per="r-x";nowner=5},    
            32 => {owner_per="r--";nowner=4},
            24 => {owner_per="-wx";nowner=3},
            16 => {owner_per="-w-";nowner=2},
            8  => {owner_per="--x";nowner=1},
            0  => {owner_per="---";nowner=0},
            _   => {panic!("wrong input mode in owner")}
        };
        let other_per:&str;
        let nother : i8;
        match mode&7{
            7 => {other_per="rwx";nother=7},
            6 => {other_per="rw-";nother=6},
            5 => {other_per="r-x";nother=5},    
            4 => {other_per="r--";nother=4},
            3 => {other_per="-wx";nother=3},
            2 => {other_per="-w-";nother=2},
            1  => {other_per="--x";nother=1},
            0  => {other_per="---";nother=0},
            _   => {panic!("wrong input mode in other")}
        };
        //let spe_per: &str;
        let nspe : i8;
        match mode&3584{
            3584 => {nspe=7},
            3072 => {nspe=6},
            2560 => {nspe=5},    
            2048 => {nspe=4},
            1536 => {nspe=3},
            1024 => {nspe=2},
            512  => {nspe=1},
            0  => {nspe=0},
            _   => {panic!("wrong input mode in special")}
        };
        Self{
            owner: owner_per,
            group: group_per,
            other: other_per,
        //    special: spe_per,
            ngroup : ngroup,
            nother : nother,
            nspe : nspe,
            nowner : nowner,
        }
    }

    pub fn output_num(&self) -> String {
        format!("{}{}{}{}",self.nspe,self.ngroup,self.nowner,self.nother)
    }

    pub fn output_char(&self) -> String{
        format!("{}{}{}",self.group,self.owner,self.other)
    }
}

pub fn get_username_with_id(uid: u32) -> String{
    unsafe{
        //let userinfo=std::mem::zeroed::<libc::passwd>();
        let userinfo = libc::getpwuid(uid);
        if userinfo.is_null(){
            format!("nullptr")
        }
        else{
            let sde = std::ffi::CStr::from_ptr((*userinfo).pw_name);
            format!("{}",sde.to_str().unwrap())
        }
    }
}

pub fn get_groupname_with_id(gid: u32) -> String{
    unsafe{
        //let userinfo=std::mem::zeroed::<libc::passwd>();
        let groupinfo = libc::getgrgid(gid);
        if groupinfo.is_null(){
            format!("nullptr")
        }
        else{
            let sde = std::ffi::CStr::from_ptr((*groupinfo).gr_name);
            format!("{}",sde.to_str().unwrap())
        }
    }
}

pub fn get_time_utc2local(tc: i64, tn: i64) -> String{
    let da : DateTime<Utc> = DateTime::from_utc(NaiveDateTime::from_timestamp(tc, tn.try_into().unwrap()), Utc);
    let lc : DateTime<Local> = DateTime::from(da);
    lc.format("%Y-%m-%d %H:%M:%S").to_string()
}