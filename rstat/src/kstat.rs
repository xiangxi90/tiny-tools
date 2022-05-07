pub use libc::{stat,statfs};
//use chrono::prelude::*;

#[allow(dead_code)]
pub struct FileStat{
    filename: String,               //文件名
    stat: stat,                     //
    statfs:statfs,                  //

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
                        'l' => self.blink=true,
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
        // let dt = Local::now();
        // println!("dt: {}", dt.offset());

        
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
            OptionSelected::output_normal(filestats);
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
                println!("{} {} {} {:x} {} {} {}{} {} {} {} {} {} {} {} {} {}",
                    iter.filename,
                    filestat.st_size,filestat.st_blocks,
                    filestat.st_mode,filestat.st_uid,filestat.st_gid,
                    get_dev_major(filestat.st_dev),get_dev_minor(filestat.st_dev),
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

    pub fn output_normal(filestat: Vec<FileStat>){
        println!("{}:","normal");
        for iter in filestat{
            print!("{}   ",iter.filename);
        }print!("\n");
    }
}

fn get_file_stat(filename : String) -> FileStat{
    unsafe{
        let mut kstat = get_new_stat();
        let ret = stat(filename.as_ptr().cast(),(&mut kstat) as *mut stat);
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
                stat: kstat,
                statfs: get_new_statfs(),
                blinker: false, 
                oriname: String::new(), 
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
                blinker: true, 
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
                blinker: true, 
                oriname: get_link_oriname(filename),
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
        let mut namebuf : [char; 50] = [' ';50];
        let ret=libc::readlink(linkname.as_ptr().cast(),namebuf.as_mut_ptr().cast(),50);
        if ret!=-1 && ret<50{
            let mut lname=String::new();
            for i in namebuf.iter(){
                lname.push(*i);
            }
            return lname;
        }
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

pub fn get_dev_major(devno: u64) -> u32{
    unsafe{
        return libc::major(devno);
    }
}

pub fn get_dev_minor(devno: u64) -> u32{
    unsafe{
        return libc::minor(devno);
    }
}
