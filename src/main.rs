mod libssh;

use libssh::{SSHSession, PubKey, ssh_keytypes as keytypes, ssh_auth as auth};

use clap::{Parser};

use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[command(flatten)]
    key: Key,

    #[arg(short, long, value_name="PORT", default_value="22")]
    port: String,

    #[arg(value_name="REMOTE_USER", required=true)]
    username: String,

    #[arg(value_name="HOST", required=true)]
    host: String,
}

#[derive(Parser, Debug, Clone)]
#[clap(group(clap::ArgGroup::new("key").required(true).multiple(false).args(&["b64", "file"])))]
struct Key {
    #[arg(short, long, value_name="KEY_B64", requires="keytype")]
    b64: Option<String>,

    #[arg(long="type")]
    keytype : Option<keytypes>,

    #[arg(short, long, value_name="KEY_FILE", conflicts_with="keytype")]
    file: Option<PathBuf>,
}

fn get_key(k: Key) -> Result<PubKey, libssh::Error> {
    match k.b64 {
        Some(b64) => PubKey::from_base64(&b64, k.keytype.unwrap()),
        None => {
            match k.file {
            Some(f) => PubKey::from_file(&k.file.unwrap()),
            None => panic!("IMPOSSIBLE"),
            }
        }
    }
}

fn main() {
    let args = Args::parse();

    let key = get_key(args.key).expect("Invalid key");
    let host = &args.host;
    let user = &args.username;
    let port = &args.port;

    let session = SSHSession::new();
    session.options_set_host(host);
    session.options_set_user(user);
    session.options_set_port_str(port);

    let session = session.connect().expect("Could not connect");
    let b = session
        .get_server_banner()
        .expect("Cannot get banner, Null");

    println!("{}", b);

    // import a public key


    // try it on the server
    match session.userauth_try_publickey(&key) {
        auth::SSH_AUTH_DENIED  => println!("Key not valid"),
        auth::SSH_AUTH_SUCCESS => println!("Key is valid"),

        auth::SSH_AUTH_ERROR => panic!("Serious SSH Error happened!"),
        _ => panic!("Impossible return value from try_publickey"),
    }
    
    session.silent_disconnect();
}
