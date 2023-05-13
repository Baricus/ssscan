mod libssh;

use scoped_thread_pool::Pool;

use libssh::{SSHSession, PubKey, ssh_keytypes as keytypes, ssh_auth as auth};

use clap::{Parser};

use std::{path::PathBuf, io::stdin};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[command(flatten)]
    key: Key,

    #[arg(short, long, value_name="NUM_THREADS", default_value="1")]
    threads: usize,

    #[arg(short, long, value_name="PORT", default_value="22")]
    port: String,

    #[arg(value_name="REMOTE_USER", required=true)]
    username: String,
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
            Some(f) => PubKey::from_file(&f),
            None => panic!("IMPOSSIBLE"),
            }
        }
    }
}

fn test_host(host: String, port: &str, user: &str, key: &PubKey) {
    let session = SSHSession::new();
    session.options_set_host(&host);
    session.options_set_user(user);
    session.options_set_port_str(port);

    // if we connected, try the key
    match session.connect() {
        Ok(session) => {
            let b = session
                .get_server_banner()
                .unwrap_or(""); // if we can't get the banner, don't print it

            // actually send the key packet
            match session.userauth_try_publickey(&key) {
                auth::SSH_AUTH_DENIED  => (),
                auth::SSH_AUTH_SUCCESS => println!("{} {}", &host, &b),

                auth::SSH_AUTH_ERROR => panic!("Serious SSH Error happened!"),
                _ => panic!("Impossible return value from try_publickey"),
            };
                
            session.silent_disconnect();
        }
        Err(_) => (), // we don't care here since this is still a failure
    };
}

fn main() {
    let args = Args::parse();

    let key  = get_key(args.key).expect("Invalid key");
    let user = args.username;
    let port = args.port;

    let mut pool = Pool::new(args.threads);
    let stdin = stdin();

    // wrap this all in a scope so this doesn't break
    // this is an annoying pain point
    // since we can't share the values, even
    // if we wait for the threads to finish at the end
    //
    // I'd love not to need this specific kind of thread pool, but at least it works
    pool.scoped(|s| {
        let mut l = true;
        while l {
            let line = {
                let mut buff = String::new();
                let r = stdin.read_line(&mut buff);
                match r {
                    Ok(..)  => buff.trim().to_string(),
                    Err(..) => " ".to_owned(),
                }
            };

            if line.is_empty() {
                l = false;
            }
            else {
                s.execute(|| test_host(line, &port, &user, &key));
            }
        }
        
        // wait for all jobs to end before we exit
        s.join();
    });
}
