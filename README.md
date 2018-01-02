# rhi (WIP)
```
rhi 0.0.2
lafolle
HTTP load generator (like hey by @rakyll)

USAGE:
    rhi [FLAGS] [OPTIONS] <url>

FLAGS:
        --disable-compression    Disable compression.
        --disable-keepalive      Disable keep-alive, prevents re-use of TCP connections between
                                 different HTTP requests.
    -h, --help                   Prints help information
    -V, --version                Prints version information

OPTIONS:
    -H <H>...                Custom HTTP header. You can specify as many as needed by repeating the
                             flag.
    -a <a>                   Basic authentication, username:password.
    -c <c>                   Number of requests to run concurrently. Total number of requests cannot
                             be smaller than the concurrency level. [default: 50]
    -d <d>                   HTTP request body.
    -m, --method <method>    HTTP method for requests [default: GET]  [values: GET, POST, PUT,
                             DELETE, HEAD, OPTIONS]
    -n <n>                   Number of requests to run. [default: 200]
    -q <q>                   Rate limit, in seconds (QPS) [default: 1]
    -t <t>                   Timeout for each request in seconds. Use 0 for infinite. [default: 20]

ARGS:
    <url>    url to hit

```

## INSTALL
Install rust: https://www.rust-lang.org/en-US/install.html
```
git clone git@github.com:lafolle/rhi.git && cd rhi && cargo build
```
To run binary (from rhi directory): ./target/debug/rhi
