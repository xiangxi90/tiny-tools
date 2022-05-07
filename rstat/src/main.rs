mod kstat;

use std::env;
use clap::{Command,arg};

fn main() {
    let mut cli = cli();
    let mut _command = cli.clone().get_matches();
    let args:Vec<String> = env::args().collect();

    let mut filesoption=kstat::OptionSelected::default();

    match filesoption.readoption(args){
        Ok(_) =>{} ,
        Err(e) => {
            println!("{:?}",e);
            cli.print_help().unwrap();
            std::process::exit(0);
        },
    };

    //println!("{:?}",filesoption);


    match filesoption.output(){
        Ok(_) => {},
        Err(e) => println!("{:?}",e),
    };
}



fn cli() -> Command<'static>{
    Command::new("rstat")
        .version("0.01")
        .author("kuze\t kuzehibiki@126.com")
        .about("stat rebuild by rust")
        .args(&[
            arg!(dereference: -L --"dereference"  "follow links"),
            arg!(filesystem: -f --"file-system"  "display file system status instead of file status"),
            //windows下没有文件系统可以调用，故而不存在这个选项
            arg!(filter: -c [format] "use the specified FORMAT instead of the default;\noutput a newline after each use of FORMAT"),
            arg!(pfilter: --"printf=FORMAT"   "like --format, but interpret backslash escapes,\nand do not output a mandatory trailing newline;\nif you want a newline, include \\n' in FORMAT"),
            arg!(terse: -t  --"terse"       "print the information in terse form"),
            arg!(mhelper: -H --"more-help" "print more help information"),
            arg!(<filename> ... "the files you want to stat"),
        ])
}

