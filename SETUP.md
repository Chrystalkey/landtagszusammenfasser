# Setup for Development and Deployment

## Description

This is a little HowTo to give you a leg to stand on with regards to what you need to actually run and deploy this project.
This is intended for developers as well as deployers(?) or well, for the world that happens after development.

## Development Setup
### Requirements
To properly develop anything on this project, you need the dependencies not prefixed by a '+'. The ones with a '+' are optional, but make your life easier.
**General**
- Docker (for linux)
- Docker Compose

**Database**
- Cargo and Rust(>=1.83)
- libpq (as development dependency)
- diesel_cli (>=2.2)
- postgres
+ pgcli

**Webserver**
- Python(>=3.13)
- Poetry(>=1.4)
- Zola(>=0.9)

**Collector**
- Python(>=3.13)
- Poetry(>=1.4)
+ redis

You will notice, the Python dependencies are the same versions.

A way to setup everything that is requried on Ubuntu is:
```bash
sudo apt update && sudo apt install libpq-dev gcc python3 python3-poetry curl docker docker-compose-v2 postgres
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup # Now follow the default on-screen options
cargo install diesel_cli --features=postgres --no-default-features
```

A way to setup everything required on windows is:
```powershell
# scoop
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
Invoke-RestMethod -Uri https://get.scoop.sh | Invoke-Expression

scoop install mingw gcc vcpkg poetry python
vcpkg install --triplet=x64-mingw-static libpq

curl -O "https://win.rustup.rs/x86_64"
./rustup-init.exe # follow on-screen instructions
cargo install diesel_cli --features=postgres --no-default-features
```
- install docker somehow afterwards with the WSL backend


### Optional Extras
I always use postgres and redis as a docker container, just so I can delete / rerun everything in the blink of an eye. If you dont, you do you.

pgcli made my life a lot easier on linux, so you might consider it.

### Shortcuts

The `docker-compose.yml` on the top level sets up, compiles and runs everything in one ginourmeous step. For development, you may want to consider commenting out what you dont need and running the rest as-is with your software locally.

## Deployment
### Description
The `docker-compose.yml` on the top level sets up, compiles and runs everything in one ginourmeous step. 
run `docker compose up --build -d` to just make it all up in one step. The default Port for the webserver is 8081 on localhost.
