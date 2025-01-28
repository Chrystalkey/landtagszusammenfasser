# This script will generate the code for the collector and database from the OpenAPI spec
# It will download the openapi-generator-cli if it is not already present
# The generated code will be placed in the collector/oapicode and database/oapicode directories

echo "Checking for openapi-generator-cli"

if (-Not (Test-Path -Path "oapi-generator" -PathType Container)) {
    Write-Host "Creating oapi-generator directory"
    New-Item -ItemType Directory -Path "oapi-generator" -Force | Out-Null
    Set-Location -Path "oapi-generator"
    
    # Download the openapi-generator-cli script
    & Invoke-WebRequest -OutFile openapi-generator-cli.jar https://repo1.maven.org/maven2/org/openapitools/openapi-generator-cli/7.11.0/openapi-generator-cli-7.11.0.jar
    
    Set-Location -Path ".."
}


# Generate Python client code
& java -jar "./oapi-generator/openapi-generator-cli.jar" generate -g python -i "$(Get-Location)/docs/specs/openapi.yml" -o "$(Get-Location)/collector/oapicode"
# Twice, once for the website and once for the collector
& java -jar "./oapi-generator/openapi-generator-cli.jar" generate -g python -i "$(Get-Location)/docs/specs/openapi.yml" -o "$(Get-Location)/webserver/oapicode"

# Generate Rust Axum server code
& java -jar "./oapi-generator/openapi-generator-cli.jar" generate -g rust-axum -i "$(Get-Location)/docs/specs/openapi.yml" -o "$(Get-Location)/database/oapicode"