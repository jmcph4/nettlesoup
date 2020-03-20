extern crate clap;
use clap::{Arg, App};

fn main() {
    App::new("tftpd")
       .version("0.1.0")
       .about("The NettleSoup TFTP server")
       .author("Jack McPherson <jmcph4.github@gmail.com>")
       .arg(Arg::with_name("root")
            .required(true)
            .value_name("ROOT")
            .help("The root of the filesystem tree to confine requests to"))
       .arg(Arg::with_name("listen")
            .long("listen")
            .short('l')
            .value_name("address")
            .help("The local address to listen on")
            .takes_value(true))
       .arg(Arg::with_name("port")
            .long("port")
            .short('p')
            .value_name("port")
            .help("The local UDP port to listen on")
            .takes_value(true))
       .arg(Arg::with_name("verbose")
            .long("verbose")
            .short('v')
            .help("Enables verbose output to STDOUT"))
       .get_matches();
}

