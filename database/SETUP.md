# Setting up the Environment for developing this project
## Database Setup
The project currently only works with postgres. Thus, set up a postgres database (or run the docker-compose.yml file included here)  and specify the environment variables as seen in the .env file, with changes as pertaining to your specific database setup.

## Setting up Rust
Set up rustc, cargo etc as described [here](https://rustup.rs/).

## Setting up Diesel
Diesel is the tool we use to set up, manage and connect to the database.
Because diesel is not only a crate, but also a command line tool to run database setup, migration and all those things, you need to install the tool seperately from the
default cargo build process.

To set up diesel, run `cargo install diesel_cli --no-default-features --features postgres`. If there is an error like 
```
note: ld: library not found for -lpq
clang: error: linker command failed with exit code 1 (use -v to see invocation)
```
Install the Postgres C libraries and rerun the client. On debian based systems this could for example be: `sudo apt install libpq-dev`.
More information on this process can be found [here](https://diesel.rs/guides/getting-started).


## Building and running the project
First, make sure you are in the `./database` directory of the project.
Then, run `cargo build` to build the project and `cargo run` ro run it.

NOTE that it is currently necessary to specify all options, refer to the program help to see which ones exactly. This is a TODO item because currently it is quite annoying.
An example run of the server with all necessary options could be:
`cargo run -- --db-url="postgres://postgres:postgres@localhost/postgres" --mail-server whaterver.example.com --mail-user user --mail-password pw --mail-sender test@example.com --mail-recipient mail@example.com`
NOTE that currently no check is done on whether the mail server is working.