/*
    Usage: rhi [options...] <url>

    Options:
    -n  Number of requests to run. Default is 200.
    -c  Number of requests to run concurrently. Total number of requests cannot
    be smaller than the concurrency level. Default is 50.
    -q  Rate limit, in seconds (QPS).
    -o  Output type. If none provided, a summary is printed.
    "csv" is the only supported alternative. Dumps the response
    metrics in comma-separated values format.

    -m  HTTP method, one of GET, POST, PUT, DELETE, HEAD, OPTIONS.
    -H  Custom HTTP header. You can specify as many as needed by repeating the flag.
    For example, -H "Accept: text/html" -H "Content-Type: application/xml" .
    -t  Timeout for each request in seconds. Default is 20, use 0 for infinite.
    -A  HTTP Accept header.
    -d  HTTP request body.
    -D  HTTP request body from file. For example, /home/user/file.txt or ./file.txt.
    -T  Content-type, defaults to "text/html".
    -a  Basic authentication, username:password.
    -x  HTTP Proxy address as host:port.
    -h2 Enable HTTP/2.

    -host HTTP Host header.

    -disable-compression  Disable compression.
    -disable-keepalive    Disable keep-alive, prevents re-use of TCP
    connections between different HTTP requests.
    -cpus                 Number of used cpu cores.
    (default for current machine is 8 cores)
    -more                 Provides information on DNS lookup, dialup, request and response timings.
*/


extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate tokio_core;
extern crate clap;
extern crate url;
extern crate core;

use futures::{Future,Stream};
use hyper::{Method, Request, Client, Uri};
use hyper::header::{ContentLength, Accept, QualityItem, Authorization, Basic};
use tokio_core::reactor::{Core, Interval};
use clap::{Arg, App, ArgMatches};
use core::str::FromStr;
use std::time::{Duration};
use std::fmt;
use core::num::ParseIntError;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const DEFAULT_NREQ: u32 = 100;
const DEFAULT_CREQ: u32 = 10;
const DEFAULT_RPS: u32 = 10;

/*
Lifetime of aninstance of Options should not exceed lifetime of ArgMatches.
*/
struct Options<'a>{

    // #requests to run. DEFAULT_NREQ.
    nreq: u32, 
    // #concurrent reqs.  DEFAULT_CREQ.
    creq: u32, 
    // rps DEFAULT_RPS.
    rps: u32, 

    // timeout per request.
    timeout: Duration, 

    matches: ArgMatches<'a>,
}

impl<'a> Options<'a>{
    
    fn get_request(&self) -> Request {
        let uri = match self.matches.value_of("url") {
            Some(x) => x,
            None => panic!("url is required by rhi."),
        };
        let uri = Uri::from_str(uri).unwrap();

        let method = match self.matches.value_of("method") {
            Some("GET") => Method::Get,
            Some("POST") => Method::Post,
            Some("PUT") => Method::Put,
            Some("DELETE") => Method::Delete,
            Some("HEAD") => Method::Head,
            Some("OPTIONS") => Method::Options,
            _  => Method::Get
        };

        let mut req = Request::new(method, uri);

        // Accept header.
        if self.matches.is_present("accept_header") {
            let qi = QualityItem::from_str(self.matches.value_of("accept_header").unwrap()).unwrap();
            req.headers_mut().set(Accept(vec![qi]));
        }

        // Basic authorization.
        if self.matches.is_present("a") {
            let v: Vec<&str> = self.matches.value_of("a").unwrap().split(':').collect();
            assert_eq!(v.len(), 2);
            let username = v[0];
            let password = v[1];
            req.headers_mut().set(Authorization(
                Basic {
                    username: username.to_owned(),
                    password: Some(password.to_owned()),
                }
            ))
        }

        // Body.
        if self.matches.is_present("d") {
            let body = self.matches.value_of("d").unwrap().to_owned();
            let blen = body.len();
            req.set_body(body);
            req.headers_mut().set(ContentLength(blen as u64));
        }

        req
    }

}

impl<'a> fmt::Display for Options<'a> {

    fn fmt(&self, f: &mut  fmt::Formatter) -> fmt::Result {
        write!(f, "(nreq:{} creq:{} rps:{})", self.nreq, self.creq, self.rps)
    }
    
}

fn main() {

    let opts = get_options().unwrap();

    let mut core = Core::new().unwrap();
    let core_handle = core.handle();
    let client = Client::new(&core_handle);

    let ticks = Interval::new(Duration::new(1, 0), &core_handle).unwrap();
    let ticks_future = ticks.for_each( move |_| {

        // Send creq requests to server per second.
        let mut c = 0;
        while c < opts.creq {
            c += 1;
            let req = opts.get_request();
            let post = client.request(req).and_then(|res| {
                println!("response: {}", res.status());
                res.body().concat2()
            }).then(|_| Ok(()) );
            core_handle.spawn(post);
        }

        Ok(())

    });
    core.run(ticks_future).unwrap();
}

fn get_options<'a>() -> Result<Options<'a>, ParseIntError> {

    let app = App::new("rhi").version(VERSION)
                    .about("HTTP load generator (like hey by @rakyll)")
                    .author("lafolle")
                    .arg(Arg::with_name("n")
                        .short("n")
                        .default_value("200")
                        .help("Number of requests to run."))
                    .arg(Arg::with_name("c")
                    .short("c")
                    .default_value("50")
                    .help("Number of requests to run concurrently. Total number of requests cannot
be smaller than the concurrency level."))
                    .arg(Arg::with_name("q")
                        .short("q")
                        .takes_value(true)
                        .default_value("1")
                        .help("Rate limit, in seconds (QPS)"))
                    .arg(Arg::with_name("method")
                        .short("m")
                        .long("method")
                        .default_value("GET")
                        .takes_value(true)
                        .possible_values(&["GET", "POST", "PUT", "DELETE", "HEAD", "OPTIONS"])
                        .help("HTTP method for requests"))
                    .arg(Arg::with_name("H")
                        .takes_value(true)
                        .short("H")
                        .multiple(true)
                        .number_of_values(1)
                        .help("Custom HTTP header. You can specify as many as needed by repeating the flag."))
                    .arg(Arg::with_name("t")
                        .takes_value(true)
                        .short("t")
                        .default_value("20")
                        .help("Timeout for each request in seconds. Use 0 for infinite."))
                    .arg(Arg::with_name("d")
                        .short("d")
                        .takes_value(true)
                        .help("HTTP request body."))
                    .arg(Arg::with_name("a")
                        .short("a")
                        .takes_value(true)
                        .help("Basic authentication, username:password."))
                    .arg(Arg::with_name("disable compression")
                        .long("disable-compression")
                        .help("Disable compression."))
                    .arg(Arg::with_name("disable-keepalive")
                        .long("disable-keepalive")
                        .help("Disable keep-alive, prevents re-use of TCP connections between different HTTP requests."))
                    .arg(Arg::with_name("url")
                        .help("url to hit")
                        .required(true)
                        .index(1));

    let matches = app.get_matches();

    let nreq = match matches.value_of("n") {
        Some(t) => t.parse::<u32>()?,
        None => DEFAULT_NREQ,
    };
    let creq = match matches.value_of("c") {
        Some(t) => t.parse::<u32>()?,
        None => DEFAULT_CREQ,
    };
    let rps = match matches.value_of("q") {
        Some(t) => t.parse::<u32>()?,
        None => DEFAULT_RPS,
    };

    let options = Options{
        nreq: nreq,
        creq: creq,
        rps: rps,
        timeout: Duration::new(10,0),
        matches: matches,
    };

    Ok(options)
}
