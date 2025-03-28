# Setup for Development and Deployment

## Description

This is a little HowTo to give you a leg to stand on with regards to what you need to actually run and deploy this project.
This is intended for developers as well as deployers(?) or well, for the world that happens after development.

## Development Setup
### Requirements
To properly develop anything on this project, you need the dependencies not prefixed by a '+'. The ones with a '+' are optional, but make your life easier.
**General**
- Docker (for linux)
- Docker Compose (with the WSL backend if you're on windows)
+ [Swagger Editor](https://editor.swagger.io/) # If you want to do API testing and development

**Database**
- Cargo and Rust(>=1.85)
- libpq (as development dependency)
- sqlx-cli (>=2.2)
- postgres
+ pgcli

**Webserver**
- Python(>=3.13)
- Poetry(>=1.4)
- Zola(>=0.9)

**Collector**
- Python(>=3.13)
- Poetry(>=1.4)
- redis

You will notice, the Python dependencies are the same versions.

A way to setup everything that is requried on Ubuntu is:
```bash
sudo apt update && sudo apt install libpq-dev gcc python3 python3-poetry curl docker docker-compose-v2 postgres
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup # Now follow the default on-screen options
cargo install sqlx-cli
```

A way to setup everything required on windows is:
1. install wsl
2. install docker for windows with the WSL backend
```powershell
# scoop
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
Invoke-RestMethod -Uri https://get.scoop.sh | Invoke-Expression

scoop install mingw gcc vcpkg poetry python
vcpkg install --triplet=x64-mingw-static libpq

curl -O "https://win.rustup.rs/x86_64"
./rustup-init.exe # follow on-screen instructions
cargo install sqlx-cli
```

### After-Setup Care
To work with Collectors and the Database, you have to authenticate the collector with the database in some way. How you can do that in general is documented [here](docs/documentation/authentication.md), but the easiest route for development is to give the database a KEYADDER_KEY and using just that for your collector.

Furthermore, the very first step after installing all required components for your development experience, you have to run `oapi_gen.sh` or `oapi_gen.ps1`, depending on your system. This will generate the appropriate libraries from the openapi.yml specification in docs/specs. All components need these generated clients/servers.

Whenevery you change the spec, you have to re-do this step.

### Known Oddities
Poetry is a bit weird when it comes to working with a generated client that does not change the version after regeneration. To cope with this circumstance, to re-build the project environment of `collector` as well as `webserver` you have to delete and reinstall the poetry env in question. One way to do that is:  

```bash
sh oapi_gen.sh
cd collector
poetry env list # this should output the name of an environment
poetry env remove <the environment from one command above>
poetry install
```

### Optional Extras
I always use postgres and redis as a docker container, just so I can delete / rerun everything in the blink of an eye. If you dont, you do you.

pgcli made my life a lot easier on linux, pgadmin4 on windows, so you might consider using them.

### Shortcuts

The `docker-compose.yml` on the top level sets up, compiles and runs everything in one ginourmeous step. For development, you may want to consider commenting out what you dont need and running the rest as-is with your software locally.
The command to build/setup any of the three sub-projects is `docker compose up -d --build <service name(s)>`

Note that the provided docker-compose file does _not_ contain any actual credentials (except for the test root api key which is irrelevant).
I personally use git stash to keep one version of it containing all relevant credentials on hand.
Remember to set the OPENAI_API_KEY and LTZF_API_KEY variables properly!
