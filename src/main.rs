extern crate clap;
extern crate rusoto_core;
extern crate rusoto_sts;
#[macro_use]
extern crate quick_error;
extern crate ini;

use clap::{App, AppSettings, Arg};
use ini::Ini;
use rusoto_core::{region::Region, CredentialsError};
use rusoto_sts::{GetSessionTokenError, GetSessionTokenRequest, Sts, StsClient};
use std::str;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        CredentialsError(err: CredentialsError) {
            from()
            description(err.description())
            display("credentials error: {}", err)
            cause(err)
        }
        GetSessionTokenError(err: GetSessionTokenError) {
            from()
            description(err.description())
            display("STS Error: {}", err)
            cause(err)
        }
        IniError(err: ::ini::ini::Error) {
            from()
            description(err.description())
            display("Config Error: {}", err)
            cause(err)
        }
        IOError(err: ::std::io::Error) {
            from()
            description(err.description())
            display("I/O Error: {}", err)
            cause(err)
        }
    }
}

type Result<T> = ::std::result::Result<T, Error>;

fn main() -> Result<()> {
    let app = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author("Ferruccio Barletta")
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .setting(AppSettings::ColoredHelp)
        .setting(AppSettings::UnifiedHelpMessage)
        .arg(
            Arg::with_name("credentials")
                .value_name("CREDENTIALS")
                .help("AWS credentials file")
                .index(1)
                .required(true),
        )
        .arg(
            Arg::with_name("profile")
                .value_name("PROFILE")
                .help("AWS profile to update")
                .index(2)
                .required(true),
        )
        .get_matches();

    let credentials = app.value_of("credentials").unwrap_or("");
    let profile = Some(app.value_of("profile").unwrap_or(""));
    if profile.unwrap() == "default" {
        println!("cannot set default profile");
        return Ok(());
    }

    let sts = StsClient::new(Region::default());

    let req = GetSessionTokenRequest {
        ..Default::default()
    };
    match sts.get_session_token(req).sync() {
        Err(err) => match err {
            GetSessionTokenError::Unknown(unknown) => {
                println!("unknown error: {}", str::from_utf8(&unknown.body).unwrap());
            }
            _ => println!("{:?}", err),
        },
        Ok(rsp) => {
            if let Some(cred) = rsp.credentials {
                let mut conf = Ini::load_from_file(credentials)?;
                conf.set_to(profile, "aws_access_key_id".to_owned(), cred.access_key_id);
                conf.set_to(
                    profile,
                    "aws_secret_access_key".to_owned(),
                    cred.secret_access_key,
                );
                conf.set_to(profile, "aws_session_token".to_owned(), cred.session_token);
                conf.write_to_file(credentials)?;
                println!("session token expires: {}", cred.expiration);
            }
        }
    }

    Ok(())
}
