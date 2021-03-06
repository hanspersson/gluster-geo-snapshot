/////////////////////////////////////////////////////////////////////////////////
//                                                                             //
//    ggsnap_utils, Common library for ggsnap and ggsnap_slave.                //
//    Copyright (C) 2018  Marcus Pedersén marcus.pedersen@slu.se               //
//                                                                             //
//    This program is free software: you can redistribute it and/or modify     //
//    it under the terms of the GNU General Public License as published by     //
//    the Free Software Foundation, either version 3 of the License, or        //
//    (at your option) any later version.                                      //
//                                                                             //
//    This program is distributed in the hope that it will be useful,          //
//    but WITHOUT ANY WARRANTY; without even the implied warranty of           //
//    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the            //
//    GNU General Public License for more details.                             //
//                                                                             //
//    You should have received a copy of the GNU General Public License        //
//    along with this program.  If not, see <http://www.gnu.org/licenses/>.    //
//                                                                             //
/////////////////////////////////////////////////////////////////////////////////

#[macro_use]
extern crate serde_derive;
extern crate toml;

use std::fs::File;
use std::io::prelude::*;
use std::process::Command;

static CONF_FILE: &'static str = "ggsnap.conf";
static CONF_ETC_DIR: &'static str = "/etc/ggsnap.conf";
static CONF_ETC_SUB_DIR: &'static str = "/etc/ggsnap/ggsnap.conf";

/// Struct that holds all information from config file  
/// Config file ggsnap.conf is interpretated with toml format  
#[derive(Deserialize)]
pub struct Config {
    pub general: General,
    pub snapshot: Snapshot,
    pub mail_from_master: Option<MailFromMaster>,
}

impl Config {
    /// If config file is missing a default
    /// Config struct is returned
    pub fn default_config() -> Config {
        Config {
            general: General {
                gluster_bin: String::from("/usr/sbin/gluster"),
                ggsnap_slave_bin: String::from("/root/ggsnap_slave")
            },
            snapshot: Snapshot {
                number_days_every_day: 10,
                number_months_with_two: 3,
                number_months_total: 12,
                master_volume: None,
                slave_volume: None,
                slave_hostname: None,
                slave_user: None
            },
            mail_from_master: None
        }
    }
}

/// Struct that holds information about sub section [general]  
/// in config file
#[derive(Deserialize)]
pub struct General {
    pub gluster_bin: String,
    pub ggsnap_slave_bin: String,
}

/// Struct that holds information about sub section [snapshot]  
/// in config file
#[derive(Deserialize)]
pub struct Snapshot {
    pub number_days_every_day: u32,
    pub number_months_with_two: u32,
    pub number_months_total: u32,
    pub master_volume: Option<String>,
    pub slave_volume: Option<String>,
    pub slave_hostname: Option<String>,
    pub slave_user: Option<String>,
}

/// Struct that holds information about sub section [mail_from_master]  
/// in config file
#[derive(Deserialize)]
pub struct MailFromMaster {
    pub smtp_server: String,
    pub authentification_mechanism: String,
    pub username: String,
    pub password: String,
    pub from_sender_address: String,
    pub to_addresses: Vec<String>,
    pub enable: bool,
}


/// Type to describe type of read error
#[derive(PartialEq, Debug)]
pub enum ConfigReadErr {
    ConfigNotFound,
    ReadFileErr,
    ConfigParseErr,
}

/// Function checks for config file in three locations:  
/// * same directory as binary file  
/// * /etc/  
/// * /etc/ggsnap/  
///
/// Config file is parsed with the toml configuration file format
/// and a Result containing Config struct is returned containing all
/// configuration.
///
/// # Example  
/// ```
/// // To get _number_months_total from [snapshot]
/// // let conf = get_config().unwrap();
/// // println!("Total months: {}", config.snapshot.number_months_total);
/// ```
///  
/// If file is not found or an error occur while
/// trying to read config file, an error is returned
/// containing description of error.  
pub fn get_config() -> Result<Config, (ConfigReadErr, String)> {
    let mut conf_content = String::new();

    if let Ok(mut current_exe) = std::env::current_exe() {
        current_exe.pop();
        current_exe.push(CONF_FILE);
        if let Ok(mut f) = File::open(current_exe) {
            match f.read_to_string(&mut conf_content) {
                Ok(_) => (),
                Err(e) => return Err((ConfigReadErr::ReadFileErr,
                                      format!("Error: Can not read {} in current directory\n{}",
                                              CONF_FILE, e.to_string()))),
            }
        }
        else if let Ok(mut f) = File::open(CONF_ETC_DIR) {
            match f.read_to_string(&mut conf_content) {
                Ok(_) => (),
                Err(e) => return Err((ConfigReadErr::ReadFileErr,
                                      format!("Error: Can not read config file: {}\n{}",
                                              CONF_ETC_DIR, e.to_string()))),
            }
        }
        else if let Ok(mut f) = File::open(CONF_ETC_SUB_DIR) {
            match f.read_to_string(&mut conf_content) {
                Ok(_) => (),
                Err(e) => return Err((ConfigReadErr::ReadFileErr,
                                      format!("Error: Can not read config file: {}\n{}",
                                              CONF_ETC_SUB_DIR, e.to_string()))),
            }
        }
        else {
            return Err((ConfigReadErr::ConfigNotFound, format!("Config file: {} is not found in current dir, /etc/ or /etc/ggsnap/", CONF_FILE)))
        }
    }
    else {
        if let Ok(mut f) = File::open(CONF_ETC_DIR) {
            match f.read_to_string(&mut conf_content) {
                Ok(_) => (),
                Err(e) => return Err((ConfigReadErr::ReadFileErr,
                                      format!("Error: Can not read config file: {}\n{}",
                                              CONF_ETC_DIR, e.to_string()))),
            }
        }
        else if let Ok(mut f) = File::open(CONF_ETC_SUB_DIR) {
            match f.read_to_string(&mut conf_content) {
                Ok(_) => (),
                Err(e) => return Err((ConfigReadErr::ReadFileErr,
                                      format!("Error: Can not read config file: {}\n{}",
                                              CONF_ETC_SUB_DIR, e.to_string()))),
            }
        }
        else {
            return Err((ConfigReadErr::ConfigNotFound, format!("Config file: {} is not found in current dir, /etc/ or /etc/ggsnap/", CONF_FILE)))
        }
    }

    parse_config(&conf_content)
}

/// Parses config string and returns a Config populated with
/// the content from string
fn parse_config(config_content: &String) -> Result<Config, (ConfigReadErr, String)> {
    match toml::from_str(config_content.as_str()) {
        Ok(c) => Ok(c),
        Err(e) => Err((ConfigReadErr::ConfigParseErr, format!("Error parse config file: {}", e)))
    }
}


/// Uses config file parameters in [snapshot]
/// to deside what to save and what to delete
/// On success a tring containing removed snapshots
/// will be returned. On error, error message will be returned
pub fn remove_old_snapshots(config: &Config) -> Result<String, String> {
    let mut snap_output: String = String::new();
    let cmd_out = Command::new(&config.general.gluster_bin)
                          .arg("snapshot")
                          .arg("list")
                          .output();

    match cmd_out {
        Ok(o) => {
            if o.status.success() {
                snap_output = format!("{}", String::from_utf8_lossy(&o.stdout));
            }
            else {
                return Err(format!("Error getting snapshots: {}{}", String::from_utf8_lossy(&o.stdout),
                            String::from_utf8_lossy(&o.stderr)))
            }
        },
        Err(e) => return Err(format!("Error executing command: gluster snapshot list\n{}", e.to_string())),
    }
    Ok(snap_output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_file_is_missing() {
        assert_eq!(get_config(),
                   Err(format!("Config file: {} is not found in current dir, /etc/ or /etc/ggsnap/",
                               CONF_FILE)));
    }

    #[test]
    fn parse_config_file() {
        let conf = String::from("
            [general]
            gluster_bin = '/usr/sbin/gluster'
            ggsnap_slave_bin = '/root/ggsnap_slave'

            [snapshot]
            number_days_every_day = 10
            number_months_with_two = 3
            number_months_total = 12

            [mail_from_master]
            smtp_server = 'mysmtp.server.com'
            authentification_mechanism = 'plain'
            username = 'foobar'
            password = 'noob'
            from_sender_address = 'aa@bb.cc'
            to_addresses = [ 'foobar@foobar.com', 'noob@noob.com' ]
            enable = true
            ");

        let c = parse_config(&conf).unwrap();
        assert_eq!(c.general.ggsnap_slave_bin, "/root/ggsnap_slave");
        assert_eq!(c.snapshot.number_months_with_two, 3);
        assert_eq!(c.mail_from_master.unwrap().enable, true);

        let conf = String::from("
            [general]
            gluster_bin = '/usr/sbin/gluster'
            ggsnap_slave_bin = '/root/ggsnap_slave'

            [snapshot]
            number_days_every_day = 10
            number_months_with_two = 3
            number_months_total = 12
            ");

        let c = parse_config(&conf).unwrap();
        assert_eq!(c.general.gluster_bin, "/usr/sbin/gluster");
        assert_eq!(c.mail_from_master.is_none(), true);
    }
}
