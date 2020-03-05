use ofxtools::OFXClient;

use futures::executor::ThreadPool;
use log::info;
use std::collections::HashMap;

use clap::{App, AppSettings, Arg, ArgMatches};
use ini::Ini;
use itertools::iproduct;

fn read_config() -> Ini {
    Ini::load_from_file("src/bin/fi.ini").unwrap()
}

fn main() {
    // let app = App::new("ofxget").about("Download OFX financial data");
    // let app = new
    let matches = App::new("ofxget")
        .about("Download OFX financial data")
        .subcommand(new_subcommand("list", "List known reachable OFX servers"))
        .subcommand(new_subcommand(
            "scan",
            "Probe OFX server for working connection parameters",
        ))
        // .subcommand(
        //     App::new("push")
        //         .about("pushes things")
        //         .setting(AppSettings::SubcommandRequiredElseHelp)
        //         .subcommand(
        //             App::new("remote") // Subcommands can have thier own subcommands,
        //                 // which in turn have their own subcommands
        //                 .about("pushes remote things")
        //                 .arg(
        //                     Arg::with_name("repo")
        //                         .required(true)
        //                         .help("The remote repo to push things to"),
        //                 ),
        //         )
        //         .subcommand(App::new("local").about("pushes local things")),
        // )
        // .subcommand(
        //     App::new("add")
        //         .about("adds things")
        //         .author("Someone Else") // Subcommands can list different authors
        //         .version("v2.0 (I'm versioned differently") // or different version from their parents
        //         .setting(AppSettings::ArgRequiredElseHelp) // They can even have different settings
        //         .arg(
        //             Arg::with_name("stuff")
        //                 .long("stuff")
        //                 .help("Stuff to add")
        //                 .takes_value(true)
        //                 .multiple(true),
        //         ),
        // )
        .get_matches();

    match matches.subcommand() {
        ("list", Some(args)) => list_fis(args),
        ("scan", Some(args)) => scan_profile(args),
        _ => unreachable!(),
    }
}

fn new_subcommand<'a>(cmd: &'static str, help: &'static str) -> App<'a> {
    let mut server = cmd == "scan";
    let mut format = cmd == "prof";
    let mut signon = cmd == "acctinfo";
    let mut stmtend = cmd == "stmtend";
    let mut stmt = cmd == "stmt";
    let mut tax = cmd == "tax1099";

    if stmt {
        stmtend = true;
    }
    if stmtend || tax {
        signon = true;
    }
    if signon {
        format = true;
    }
    if format {
        server = true;
    }

    let mut app = App::new(cmd)
        .about(help)
        .arg(
            Arg::with_name("server")
                .help("OFX server nickname")
                .required(false),
        )
        .arg(
            Arg::with_name("--verbose")
                .short('v')
                .help("Give more output (option can be repeated)")
                .multiple_occurrences(true),
        );

    if server {
        app = app
            .arg(
                Arg::with_name("url")
                    .long("url")
                    .help("OFX server URL")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("ofxhome")
                    .long("ofxhome")
                    .help("FI id# on http://www.ofxhome.com/")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("write")
                    .long("write")
                    .short('w')
                    .help("Write working parameters to config file"),
            )
            .arg(
                Arg::with_name("unsafe")
                    .long("unsafe")
                    .help("Skip SSL certificate verification"),
            );
    }

    app
}

// CLI METHODS

/// Report working connection parameters
fn scan_profile(args: &ArgMatches) {
    if args.is_present("dryrun") {
        panic!("Can't reasonably show a dry run for a profile scan");
    }

    let url = args.value_of("url").unwrap();
    let org = args.value_of("org");
    let fid = args.value_of("fid");

    let scan_results = _scan_profile(url, org, fid, None, None);
    // scan_results = _scan_profile(url, org, fid)

    // v1, v2, signoninfo = scan_results
    // if (not v2["versions"]) and (not v1["versions"]):
    // msg = f"Scan found no working formats for {url}"
    // print(msg)
    // else:
    // print(json.dumps(scan_results))

    // if args["write"]:
    //     extra_args = _best_scan_format(scan_results)
    //     write_config(ChainMap(extra_args, dict(args)))
}

// PROFILE SCAN

/// Report permutations of OFX version/prettyprint/unclosedelements that
/// successfully download OFX profile from server.
/// Returns a 3-tuple of (OFXv1 results, OFXv2 results, signoninfo), each
/// type(dict).  OFX results provide ``ofxget`` configs that will work to
/// make a basic OFX connection. SIGNONINFO reports further information
/// that may be helpful to authenticate successfully.
fn _scan_profile(
    url: &str,
    org: Option<&str>,
    fid: Option<&str>,
    max_workers: Option<i32>,
    timeout: Option<f32>,
) {
    println!(
        "Scanning url={} org={} fid={}",
        url,
        org.unwrap_or("None"),
        fid.unwrap_or("None")
    );
    let client = OFXClient::new(
        url.to_string(),
        org.map(|x| x.to_string()),
        fid.map(|x| x.to_string()),
    );
    let futures = _queue_scans(&client, max_workers, timeout);
}
// def _scan_profile(
//     url: str,
//     org: Optional[str],
//     fid: Optional[str],
//     max_workers: Optional[int] = None,
//     timeout: Optional[float] = None,
// ) -> ScanResults:
//     logger.info(
//         (
//             f"Scanning url={url} org={org} fid={fid} "
//             f"max_workers={max_workers} timeout={timeout}"
//         )
//     )
//     client = OFXClient(url, org=org, fid=fid)
//     futures = _queue_scans(client, max_workers, timeout)

//     # The primary data we keep is actually the metadata (i.e. connection
//     # parameters - OFX version; prettyprint; unclosedelements) tagged on
//     # the Future by _queue_scans() that gave us a successful OFX connection.
//     success_params: FormatMap = defaultdict(list)
//     # If possible, we also parse out some data from SIGNONINFO included in
//     # the PROFRS.
//     signoninfo: SignoninfoReport = {}

//     # Assume that SIGNONINFO is the same for each successful OFX PROFRS.
//     # Tell _read_scan_response() to stop parsing out SIGNONINFO once
//     # it's successfully extracted one.
//     for future in concurrent.futures.as_completed(futures):
//         version, format = futures[future]
//         valid, signoninfo_ = _read_scan_response(future, not signoninfo)

//         if not valid:
//             continue
//         if not signoninfo and signoninfo_:
//             signoninfo = signoninfo_

//         logger.debug((f"OFX connection success, version={version}, format={format}"))
//         success_params[version].append(format)

//     v1_result, v2_result = [
//         collate_scan_results(ver)
//         for ver in utils.partition(lambda it: it[0] >= 200, success_params.items())
//     ]

//     # V2 always has closing tags for elements; just report prettyprint
//     for fmt in v2_result["formats"]:
//         assert not fmt["unclosedelements"]
//         del fmt["unclosedelements"]

//     results = (v1_result, v2_result, signoninfo)
//     logger.info(f"Scan results: {results}")
//     return results

fn _queue_scans(client: &OFXClient, max_workers: Option<i32>, timeout: Option<f32>) {
    let ofxv1 = vec![102, 103, 151, 160];
    let ofxv2 = vec![200, 201, 202, 203, 210, 211, 220];
    let BOOLS = vec![false, true];

    let executor = ThreadPool::builder()
        .pool_size(max_workers)
        .create()
        .unwrap();
    for (version, pretty, close) in iproduct!(ofxv1, BOOLS, BOOLS) {
        // future =
    }
}
// def _queue_scans(
//     client: OFXClient, max_workers: Optional[int], timeout: Optional[float]
// ) -> Mapping[concurrent.futures.Future, ScanMetadata]:
//     ofxv1 = [102, 103, 151, 160]
//     ofxv2 = [200, 201, 202, 203, 210, 211, 220]

//     BOOLS = (False, True)

//     futures = {}
//     with concurrent.futures.ThreadPoolExecutor(max_workers) as executor:
//         for version, pretty, close in itertools.product(ofxv1, BOOLS, BOOLS):
//             future = executor.submit(
//                 client.request_profile,
//                 version=version,
//                 prettyprint=pretty,
//                 close_elements=close,
//                 timeout=timeout,
//             )
//             futures[future] = (
//                 version,
//                 {"pretty": pretty, "unclosedelements": not close},
//             )

//         for version, pretty in itertools.product(ofxv2, BOOLS):
//             future = executor.submit(
//                 client.request_profile,
//                 version=version,
//                 prettyprint=pretty,
//                 close_elements=True,
//                 timeout=timeout,
//             )
//             futures[future] = (
//                 version,
//                 {"pretty": pretty, "unclosedelements": not close},
//             )

//     return futures

// CLI UTILITIES
fn list_fis(args: &ArgMatches) {
    let usercfg = read_config();

    let server = args.value_of("server");
    match server {
        None => {
            let mut entries: Vec<_> = fi_index(usercfg.clone())
                .iter()
                .map(|srv| format!("{:<40}{:<30}{:<8}", srv.0, srv.1, srv.2))
                .collect();
            entries.insert(
                0,
                vec!["=".repeat(39), "=".repeat(29), "=".repeat(8)].join(" "),
            );
            entries.insert(
                0,
                format!("{:^40}{:^30}{:^8}", "Name", "Nickname", "OFX Home"),
            );

            for entry in entries {
                println!("{}", entry);
            }
        }
        Some(server) => {
            let mut usercfg = usercfg.into_iter();
            let names = usercfg.next().unwrap();

            let servercfg = usercfg
                .filter(|(srv, _)| srv.unwrap() == server)
                .next()
                .expect(&format!("Unknown server {}", server)[..]);

            let ofxhome = servercfg.1.get("ofxhome").unwrap_or("");
            let name = names.1.get(ofxhome);
            let config = servercfg.1.iter().map(|x| format!("{} = {}", x.0, x.1));
            println!("");
            if let Some(name) = name {
                println!("{}", name);
            }
            println!("{}", config.collect::<Vec<_>>().join("\n"));
            println!("");
        }
    }
}

/// All FIs known to ofxget
fn fi_index(usercfg: Ini) -> Vec<(String, String, String)> {
    let mut usercfg = usercfg.into_iter();

    let mut names = HashMap::new();
    let (sec, prop) = usercfg.next().unwrap();
    let sec = sec.unwrap();
    assert_eq!(sec, "NAMES");
    for (id, name) in prop.iter() {
        names.insert(id, name);
    }

    let mut servers: Vec<_> = usercfg
        .map(|(nick, sct)| {
            (
                names
                    .remove(sct.get("ofxhome").unwrap_or(&""))
                    .unwrap_or(&"")
                    .to_string(),
                nick.unwrap().to_string(),
                sct.get("ofxhome").unwrap_or(&"--").to_string(),
            )
        })
        .collect();
    servers.sort_by_key(|srv| {
        let key = srv.0.to_lowercase();
        if key.starts_with("the ") {
            return key[4..].to_string();
        }
        key
    });
    servers
}
