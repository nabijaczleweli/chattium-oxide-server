use clap::{App as Clapp, Arg as Clarg};
use std::io::{self, Write};
use std::str::FromStr;
use std::fmt::Arguments as FormatArguments;
use std::path::PathBuf;
use std::error::Error;
use yaml_file_handler::Yaml;
use yaml_file_handler::yaml_handler::FileHandler as YamlFileHandler;


#[derive(Debug)]
pub struct Options {
    pub port: u16,
    pub ssl: Option<(String, String)>,
}


impl Options {
    /// Parses commandline arguments into an [`Options`](#) instance
    ///
    /// Optionally reads from a config file in [YAML](http://yaml.org) format, however commandline arguments take preference thereover.
    /// The config file format is non-trivial, see `"example/config.yml"`.
    pub fn parse() -> Options {
        const PORT_USAGE: &'static str = "-p --port [PORT]          'Specifies the port the \
                                          server will run on, will prompt if not specified'";
        const USAGE: &'static str = "--ssl [SSL_CERT;SSL_KEY]  'Sets SSL cert and key to use
		                                  \
                                     -c --config=[CONFIG_FILE] 'Sets config file to load, values \
                                     will be overriden by commandline args'";

        let matches = Clapp::new("chattium-oxide-server")
                          .version(env!("CARGO_PKG_VERSION"))
                          .author("nabijaczleweli <nabijaczleweli@gmail.com>")
                          .about("Chat server for chattium-oxide-client")
                          .args_from_usage(USAGE)
                          .arg(Clarg::from_usage(PORT_USAGE).validator(|val| {
                              match u16::from_str(&*&val) {
                                  Ok(_) => Ok(()),
                                  Err(error) => Err(error.description().to_string()),
                              }
                          }))
                          .get_matches();
        let mut port: Option<u16> = None;
        let mut ssl_cert: Option<String> = None;
        let mut ssl_key: Option<String> = None;

        if let Some(config) = matches.value_of("config") {
            let mut yaml = YamlFileHandler::new();
            if yaml.add_files(vec![config]) {
                if let Some(yaml) = yaml.read_all_files().as_ref().map(|all| {
                    let mut b = PathBuf::from(config);
                    b.set_extension("");
                    &all[b.file_name().unwrap().to_str().unwrap()]
                }) {
                    if let Some(map) = yaml["ssl"].as_hash() {
                        ssl_cert = if let Some(cert) = map.get(&Yaml::String("cert".to_string())) {
                            cert.as_str().map(|c| c.to_string())
                        } else {
                            None
                        };
                        ssl_key = if let Some(key) = map.get(&Yaml::String("key".to_string())) {
                            key.as_str().map(|k| k.to_string())
                        } else {
                            None
                        };
                    }
                    port = yaml["port"].as_i64().map(|p| p as u16);
                }
            }
        }

        if let Some(cport) = matches.value_of("port") {
            port = Some(u16::from_str(cport).unwrap())
        }  // Validated using Arg::validator
        if let Some(cssl) = matches.values_of("ssl") {
            let cssl = cssl.collect::<Vec<_>>().join(";");
            let mut paths = cssl.split(';');
            ssl_cert = Some(paths.next().unwrap().to_string());
            ssl_key = Some(paths.next().unwrap().to_string());
        }

        if port.is_none() {
            let mut tport: Option<u16> = None;
            while tport.is_none() {
                match read_prompted(format_args!("No port specified.\nPlease type in the port \
                                                  now: ")) {
                    Ok(rport) => {
                        match rport {
                            Some(rport) => {
                                match u16::from_str(&*&rport) {
                                    Ok(port) => tport = Some(port),
                                    Err(error) => println!("Invalid port value: {}", error),
                                }
                            }
                            None => (),
                        }
                    }
                    Err(error) => println!("Couldn't read port: {}", error),
                }
            }
            port = tport;
        }

        assert!(port.is_some());
        Options {
            port: port.unwrap(),
            ssl: match (ssl_cert, ssl_key) {
                (Some(cert), Some(key)) => Some((cert, key)),
                (None, Some(key)) => {
                    println!("SSL key specified (\"{}\"), but certificate not specified",
                             key);
                    None
                }
                (Some(cert), None) => {
                    println!("SSL key not specified, but certificate specified (\"{}\")",
                             cert);
                    None
                }
                (None, None) => None,
            },
        }
    }
}


fn read_prompted(prompt: FormatArguments) -> io::Result<Option<String>> {
    try!(io::stdout().write_fmt(prompt));
    try!(io::stdout().flush());

    let mut obuf = String::new();
    try!(io::stdin().read_line(&mut obuf));  // Don't match on this directly, because it returns with "\r\n"
    let obuf = obuf.trim();
    Ok(match obuf.len() {
        0 => None,
        _ => Some(obuf.trim().to_string()),
    })
}
