# BDCS API Library and Server

[![Build Status](https://travis-ci.org/weldr/bdcs-api-rs.svg?branch=master)](https://travis-ci.org/weldr/bdcs-api-rs)
[![Coverage Status](https://coveralls.io/repos/github/weldr/bdcs-api-rs/badge.svg?branch=master)](https://coveralls.io/github/weldr/bdcs-api-rs?branch=master)

This codebase is the BDCS API server, it handles API requests for project
information, recipes, and image composing.

It depends on the BDCS metadata store generated by the [bdcs import
service](https://github.com/weldr/bdcs), and can be run directly via `cargo
run` if the system has Rust installed, or via Docker.

## Content Store Server

The bdcs-api-server now includes serving the ostree based content store. It can be accessed at
`/api/bdcs/` and by default serves up the files from `/mddb/cs.repo`. This path can be changed
by passing `--bdcs <path>` when starting the server.

You can add a remote repo to ostree like this:
`ostree --repo=bdcs-repo remote --no-gpg-verify add upstream http://URL-TO-API/api/bdcs/`

And then pull the bdcs reposity updates using the mirror operation:
`ostree --repo=bdcs-repo pull --depth=-1 --mirror upstream`

NOTE an ostree build later than 2017.6 is needed so that the --depth=-1 pulls all of the history.

## Running bdcs-api-server directly on the host

The server requires the nightly version of Rust (1.17 or later)
which is best installed by
following the instructions at https://rustup.rs, and then overriding the
default compiler to use for the bdcs-api-rs project by running this in the repo
directory:

`rustup override set nightly`

You can install and run rustup as a user, no root access needed.

If building fails try setting the override to the same version listed in the `Dockerfile` instead of `nightly`.

### Required Host Packages
The host will need to have the following packages installed before building or running the bdcs-api-server
* openssl-devel
* sqlite-devel
* cmake
* development-tools

eg. on Fedora run this to setup the system:

`dnf install @development-tools openssl-devel sqlite-devel cmake`

Running it directly on port 4000, using /var/tmp/recipes/ for recipe storage
looks like this:

`cargo run --bin bdcs-api-server -- --host 0.0.0.0 --port 4000 metadata.db /var/tmp/recipes/`

This will download any required crates, build them, build bdcs-api-server and run it.

To see what command line options are available execute:

`cargo run --bin bdcs-api-server -- --help`

You may want to use `--log` to configure the path to the log file and update
`/var/tmp/recipes` to a directory where you have write permissions!

If you want to use the /api/mock/ service you can point it to a directory of
json mock api files by adding `--mockfiles /path/to/files/`


## Running the API Server in Docker

The docker image depends on a base image, named weld/fedora:25, which needs
have been previously built. If it is not available it can be built from
the welder-deployment repository by running `make weld-f25`.

Build the docker image by running:

`sudo docker build -t weld/bdcs-api .`

To run the API it requires access to a copy of the metadata.db created by the
[bdcs import service](https://github.com/weldr/bdcs) and to a directory of
recipes. The recipes directory is initialized at runtime from the
./examples/recipes/ directory.

Create `~/tmp/mddb/` and copy metadata.db into it, and create an empty
`~/tmp/recipes/` directory. You can then run the API server like this:

`docker run -it --rm -v ~/tmp/mddb/:/mddb/:Z -v ~/tmp/recipes/:/bdcs-recipes/:Z -p 4000:4000 weld/bdcs-api`

You can then access the UI at `http://localhost:4000`, try `http://localhost:4000/api/v0/test` to
make sure you get a response like `API v0 test` from the server.

If you want to use a local directory named mock-api for the `/api/mock/`
service you would add this to the commandline (before weld/bdcs-api):
`-v ~/tmp/mock-api/:/mockfiles/:Z`

The files in tests/results/v0/ are suitable to use with the `/api/mock/` service.

See the [documentation on the mock api](src/api/mock.rs) for more information.

## Testing

All available tests are executed in Travis CI. To execute them type:

`cargo test`

Use `RUST_TEST_THREADS=1` to force sequential execution and debug failing tests!

Information about depclose integration testing can be found at
[tests/depclose-integration/README.md](blob/master/tests/depclose-integration/)

General information about testing in Rust can be found at

* [The Rust Testing Guide](http://aturon.github.io/stability-dashboard/guide-testing.html)
* [Official Testing Documentation](https://doc.rust-lang.org/book/testing.html)
* [Testing|Rust by Example](http://rustbyexample.com/meta/test.html)
