# ssscan
`ssscan` checks if a list of servers accept a login from a given public-private key pair,
without requiring the user to know the associated private key.
`ssscan` is focused on asking many servers about one specific public key,
rather than asking a single server about many keys.
For example, giving `ssscan` the public key left as a backdoor by malware
searches for compromised machines with the same backdoor.

As this is relatively simple program to write,
and I wanted an excuse to try out `Rust`,
`ssscan` is implemented in Rust with `C` bindings:

&nbsp;&nbsp;&nbsp;&nbsp;
![Made with Rust](.github/images/made-with-rust.svg)
&nbsp;&nbsp;&nbsp;&nbsp;
![Bindings unsafe](.github/images/bindings-unsafe.svg)

## Usage
### Compilation
`ssscan` requires the development version of the [libssh](https://www.libssh.org/) library in order to compile.
`libssh` can be installed via your system's package manager or via downloads on the previously linked website.
`ssscan` has been tested with versions `0.9.5` and `0.9.7`, but will likely work with any unless there is/was a breaking change to the public key authentication flow.
After installing the library, `ssscan` can be installed via running:
```bash
cargo install --path .
```
in the root directory or built and run with:
```bash
cargo build
cargo run -- -h
```

### Testing A Server
`ssscan` needs a public key and a username to function.  
As an example, we'll take one of the keys tied to my [Github profile](https://github.com/baricus.keys):
```
ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIEmSKzdXduwYgD2ICYWrOo1xMiZW5RUK2MqxgvEJmVn9
```
We'd like to confirm that this key is authorized to log in to `git@github.com` via ssh.
`ssscan` requires us to pass the public key and username on the command line. 
The username is always the last (and only) positional argument,
while the key can be either read in from a public key file or base 64.
The base 64 option is expressed on the command line and requires we specify the type of key:
```bash
ssscan -b AAAAC3NzaC1lZDI1NTE5AAAAIEmSKzdXduwYgD2ICYWrOo1xMiZW5RUK2MqxgvEJmVn9 --type ssh-keytype-ed25519-cert01 git
```
while the public key file option just specifies the file:
```bash
FILE=$(mktemp) # or any other file if you prefer
curl https://github.com/baricus.keys | head -n 1 > $FILE
ssscan -f $FILE git
```
In either case, the program will wait for IPs to test entered via user input.
As Github uses various IPs, I'd recommend running `dig github.com` to get an up to date IP,
but for example we'll use the IP I had at time of writing:
```bash
$ ssscan -f $FILE git
> 140.82.114.4
< 140.82.114.4    SSH-2.0-babeld-fc59fe75
```
Note that `>` means user input and `<` means program output.
Here, we see the IP and the server banner of the github machine we connected to,
which means the key was accepted!
If we try another ip:
```bash
$ ssscan -f $FILE git
> 1.1.1.1

```
we get no output, signifying that the key was not accepted.

### Testing Many Servers
`ssscan` acts as a filter, taking in IPs on standard input
and writing those IPs to standard output only if they accept the provided public key.
This approach allows for handling large quantities of IPs,
either in pre-existing lists or generated via tools such as [zmap](https://github.com/zmap/zmap).
For example, running:
```bash
KEY=AAAAB3NzaC1yc2EAAAADAQABAAABAQCl0kIN33IJISIufmqpqg54D6s4J0L7XV2kep0rNzgY1S1IdE8HDef7z1ipBVuGTygGsq+x4yVnxveGshVP48YmicQHJMCIljmn6Po0RMC48qihm/9ytoEYtkKkeiTR02c6DyIcDnX3QdlSmEqPqSNRQ/XDgM7qIB/VpYtAhK/7DoE8pqdoFNBU5+JlqeWYpsMO+qkHugKA5U22wEGs8xG2XyyDtrBcw10xz+M7U8Vpt0tEadeV973tXNNNpUgYGIFEsrDEAjbMkEsUw+iQmXg37EusEFjCVjBySGH3F+EQtwin3YmxbB9HRMzOIzNnXwCFaYU5JjTNnzylUBp/XB6B
sudo zmap -p 22 -B 1M | ssscan -b "$KEY" --type ssh-keytype-rsa -t 20 root > compromised
```
should give you an ip in the output file within a few minutes (unless you're unlucky or it's been a very long time since this was written).  

Notably, for situations like a list of IPs piped into `ssscan` and especially using tools like `zmap`,
it is recommended to use the `-t` parameter to set a number of threads to parallelize testing machines.
These threads are extremely network bound and do very little work, 
so you can have far more than you have physical threads on the machine with little issue.  

For specifically the use case of piping a network scanner directly into `ssscan`,
around double the average receive rate is a good number of threads;
testing a single server often takes around 1-2 seconds, so double the receiving rate
usually provides enough of a buffer to prevent a large task queue from building up.  
This may require some tuning to find the best number of threads for your use case.  

## How `ssscan` Works
SSH public key authentication is described in [RFC4252](https://www.rfc-editor.org/rfc/rfc4252).
In order to prevent needlessly prompting the user for their passphrase
and to avoid needless computation,
the protocol specifies a message of the format:
```
      byte      SSH_MSG_USERAUTH_REQUEST
      string    user name in ISO-10646 UTF-8 encoding [RFC3629]
      string    service name in US-ASCII
      string    "publickey"
      boolean   FALSE
      string    public key algorithm name
      string    public key blob
```
(found near the top of [page 9](https://www.rfc-editor.org/rfc/rfc4252#page-9))

The server is required to respond with `SSH_MSG_USERAUTH_PK_OK`
if the key would be accepted
and `SSH_MSG_USERAUTH_FAILURE` otherwise.
Since this message does not require any signature (as having one would defeat the point of the message)
we can send the message without knowing the private key.
`ssscan` connects to a SSH server and uses this message to test the given key.
We don't need to (and cannot) finish the authentication flow,
so `ssscan` immediately disconnects afterwards to save bandwidth.
