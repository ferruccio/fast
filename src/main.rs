extern crate clap;
extern crate rusoto_core;
extern crate rusoto_sts;
#[macro_use] extern crate quick_error;
extern crate ini;

use clap::{App, Arg, AppSettings};
use rusoto_core::{region::Region, TlsError, CredentialsError};
use rusoto_sts::{Sts, StsClient, GetSessionTokenRequest, GetSessionTokenError};
use ini::Ini;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        TlsError(err: TlsError) {
            from()
            description(err.description())
            display("tls error: {}", err)
            cause(err)
        }
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
    let app =
        App::new(env!("CARGO_PKG_NAME"))
            .version(env!("CARGO_PKG_VERSION"))
            .author("Ferruccio Barletta")
            .about(env!("CARGO_PKG_DESCRIPTION"))
            .setting(AppSettings::ColoredHelp)
            .setting(AppSettings::UnifiedHelpMessage)
            .arg(Arg::with_name("config")
                .value_name("CONFIG")
                .help("git config file")
                .index(1)
                .required(true))
            .arg(Arg::with_name("profile")
                .value_name("PROFILE")
                .help("git profile")
                .index(2)
                .default_value("git")
                .required(false))
            .get_matches();

    let config = app.value_of("config").unwrap_or("");
    let profile = Some(app.value_of("profile").unwrap_or(""));

    let sts = StsClient::simple(Region::default());

    let req = GetSessionTokenRequest { ..Default::default() };
    let rsp = sts.get_session_token(&req).sync()?;

    if let Some(cred) = rsp.credentials {
        let mut conf = Ini::load_from_file(config)?;
        conf.set_to(profile, "aws_access_key_id".to_owned(), cred.access_key_id);
        conf.set_to(profile, "aws_secret_access_key".to_owned(), cred.secret_access_key);
        conf.set_to(profile, "aws_session_token".to_owned(), cred.session_token);
        conf.write_to_file(config)?;
        println!("session token expires: {}", cred.expiration);
    }

    Ok(())
}