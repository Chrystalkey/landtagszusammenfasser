# Setup for Development and Deployment

## Description

This is a little HowTo to give you a leg to stand on with regards to what you need to actually run and deploy this project.
This is intended for developers as well as deployers(?) or well, for the world that happens after development.

## Development Setup
### Requirements
To properly develop anything on this project, you need the dependencies not prefixed by a '+'. The ones with a '+' are optional, but make your life easier.
Note that when cloning this project you **MUST** specify `--recurse-submodules` to also pull in the required repositories. 
In full: `git clone git@github.com:Chrystalkey/landtagszusammenfasser.git --recurse-submodules`.

**General (=needed by everyone)**
- Docker (for linux)
- Docker Compose (with the WSL backend if you're on windows)
+ [Swagger Editor](https://editor.swagger.io/) # If you want to do API testing and development

**ltzf-backend**
- Cargo and Rust(>=1.85)
- libpq (as development dependency)
- sqlx-cli (>=2.2)
- postgres
+ pgcli / pgadmin

**ltzf-website**
- Python(>=3.13)
- Poetry(>=1.4)
- Zola(>=0.9)

**ltzf-collector**
- Python(>=3.13)
- Poetry(>=1.4)
- tesseract(>=5.5)
- tesseract languages(>=4.1)
- redis

You will notice, the Python dependencies are the same versions.

A way to setup everything that is requried on Ubuntu is:
```bash
sudo apt update && sudo apt install libpq-dev gcc python3 python3-poetry curl docker docker-compose-v2 postgres tesseract-ocr pandoc
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup # Now follow the default on-screen options
cargo install sqlx-cli

## after all prerequisites are installed to get to a working state, do:
git clone git@github.com:Chrystalkey/landtagszusammenfasser.git --recurse-submodules
cd landtagszusammenfasser
sh oapi_gen.sh
```

A way to setup everything required on windows is:
1. install wsl
2. install docker for windows with the WSL backend
```powershell
# scoop
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
Invoke-RestMethod -Uri https://get.scoop.sh | Invoke-Expression

scoop install mingw gcc vcpkg poetry python tesseract tesseract-languages
vcpkg install --triplet=x64-mingw-static libpq

curl -O "https://win.rustup.rs/x86_64"
./rustup-init.exe # follow on-screen instructions
cargo install sqlx-cli

## after all prerequisites are installed to get to a working state, do:
git clone git@github.com:Chrystalkey/landtagszusammenfasser.git --recurse-submodules
.\oapi_gen.ps1
```

### After-Setup Care
To work with Collectors and the Database, you have to authenticate the collector with the database in some way. How you can do that in general is documented [here](docs/documentation/authentication.md), but the easiest route for development is to give the database a KEYADDER_KEY and using just that for your collector.

#### OAPI Generators
Furthermore, the very first step after installing all required components for your development experience, you have to run `oapi_gen.sh` or `oapi_gen.ps1`, depending on your system. This will generate the appropriate libraries from the openapi.yml specification in docs/specs. All components need these generated code modules.
Whenever you change the spec, you have to re-do this step.

The generation can also be done once per subproject, in which case (example: ltzf-collector) you have to
```bash
cd ltzf-collector
sh oapigen.sh # or .\oapigen.ps1 on windows
```
Which will result in the same code being generated in the same place.

### Known Oddities of the software setup
Poetry is a bit weird when it comes to working with a generated client that does not change the version after regeneration. To cope with this circumstance, to re-build the project environment of `ltzf-collector` as well as `ltzf-website` you have to delete and reinstall the poetry env in question. One way to do that is:  

```bash
sh oapi_gen.sh
cd ltzf-collector
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
