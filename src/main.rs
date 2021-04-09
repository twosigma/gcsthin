//  Copyright 2020 Two Sigma Investments, LP.
//
//  Licensed under the Apache License, Version 2.0 (the "License");
//  you may not use this file except in compliance with the License.
//  You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.

use structopt::{StructOpt, clap::AppSettings};
use anyhow::{anyhow, Result};
use std::io::{self, stdin, stdout, BufReader, BufWriter};

mod auth;
use auth::get_auth;

const BASE_URL: &str = "https://www.googleapis.com/storage/v1";
const INSERT_BASE_URL: &str = "https://www.googleapis.com/upload/storage/v1";

const KB: usize = 1024;
// These values seem to have a sweet spot for optimizing CPU utilization
const STDIN_BUF_SIZE: usize = 256*KB;
const STDOUT_BUF_SIZE: usize = 256*KB;

/// Wrapper that sets request connect/read/write timeouts to 30s.
pub fn ureq_request(method: &str, url: &str) -> ureq::Request {
    let mut req = ureq::request(method, url);
    req.timeout_connect(30_000);
    req.timeout_read(30_000);
    req.timeout_write(30_000);
    req
}

struct BucketFile {
    bucket: String,
    path: String,
}

impl BucketFile {
    fn new(url_str: &str) -> Self {
        let url = url::Url::parse(url_str).expect("Invalid URL");
        assert!(url.scheme() == "gs", "Invalid URL. It must start with gs://");
        let bucket = url.host_str().expect("Incomplete URL");
        let mut path = url.path();

        // Skip the leading slash of the path
        if path.chars().nth(0) == Some('/') {
            path = &path[1..];
        }

        let bucket = bucket.to_string();
        let path = path.to_string();

        Self { bucket, path }
    }
}

#[test]
fn test_bucket_file() {
    let BucketFile { bucket, path } = BucketFile::new("gs://bucket_name/dir/file");
    assert_eq!(bucket, "bucket_name");
    assert_eq!(path, "dir/file");
}


fn upload(dst: &BucketFile) -> Result<()> {
    let stdin = BufReader::with_capacity(STDIN_BUF_SIZE, stdin());

    let url = format!("{}/b/{}/o", INSERT_BASE_URL, dst.bucket);
    let res = ureq_request("POST", &url)
        .set("Transfer-Encoding", "chunked")
        .set("Authorization", &get_auth("devstorage.read_write")?)
        .query("uploadType", "media")
        .query("name", &dst.path)
        .send(stdin);

    if res.ok() {
        Ok(())
    } else {
        Err(anyhow!("Failed to upload: {}", res.into_string().unwrap()))
    }
}

fn download(src: &BucketFile) -> Result<()> {
    let mut stdout = BufWriter::with_capacity(STDOUT_BUF_SIZE, stdout());

    let path = src.path.replace("/", "%2F");
    let url = format!("{}/b/{}/o/{}", BASE_URL, src.bucket, path);
    let res = ureq_request("GET", &url)
        .set("Authorization", &get_auth("devstorage.read_only")?)
        .query("alt", "media")
        .call();

    if res.ok() {
        io::copy(&mut res.into_reader(), &mut stdout)?;
        Ok(())
    } else {
        Err(anyhow!("Failed to download: {}", res.into_string().unwrap()))
    }
}

#[derive(StructOpt, PartialEq, Debug)]
struct Cp {
    /// Source. Can be - or a gs:// URL
    #[structopt(name="SRC")]
    src: String,

    /// Destination. Can be - or a gs:// URL
    #[structopt(name="DST")]
    dst: String,
}

impl Cp {
    fn run(self) -> Result<()> {
        match (&*self.src, &*self.dst) {
            ("-", url) => upload(&BucketFile::new(url)),
            (url, "-") => download(&BucketFile::new(url)),
            _ => panic!("One of SRC or DST should be -"),
        }
    }
}

#[derive(StructOpt, PartialEq, Debug)]
#[structopt(about,
    // When showing --help, we want to keep the order of arguments defined
    // in the `Opts` struct, as opposed to the default alphabetical order.
    global_setting(AppSettings::DeriveDisplayOrder),
    // help subcommand is not useful, disable it.
    global_setting(AppSettings::DisableHelpSubcommand),
    // subcommand version is not useful, disable it.
    global_setting(AppSettings::VersionlessSubcommands),
)]
struct Opts {
    #[structopt(subcommand)]
    operation: Operation,
}

#[derive(StructOpt, PartialEq, Debug)]
enum Operation {
    /// Transfer a single file
    Cp(Cp),
}

fn main() -> Result<()> {
    let opts: Opts = Opts::from_args();
    match opts.operation {
        Operation::Cp(opts) => opts.run(),
    }
}
