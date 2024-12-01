#/bin/sh
# This script will generate the code for the collector and database from the OpenAPI spec
# It will download the openapi-generator-cli if it is not already present
# The generated code will be placed in the collector/oapicode and database/oapicode directories

echo "Checking for openapi-generator-cli"

if [! -d "oapi-generator"]; then
    echo "Creating oai-generator directory"
    mkdir -p oapi-generator 
    cd oapi-generator
    curl https://raw.githubusercontent.com/OpenAPITools/openapi-generator/master/bin/utils/openapi-generator-cli.sh > openapi-generator-cli
    chmod u+x openapi-generator-cli
    cd ..
fi

if (-Not (Test-Path -Path "oapi-generator" -PathType Container)) (
    echo "Creating oai-generator directory"
    mkdir -p oapi-generator 
    cd oapi-generator
    curl https://raw.githubusercontent.com/OpenAPITools/openapi-generator/master/bin/utils/openapi-generator-cli.sh > openapi-generator-cli
    chmod u+x openapi-generator-cli
    cd ..
)


./oapi-generator/openapi-generator-cli generate -g python -i $(pwd)/docs/specs/openapi.yml -o $(pwd)/collector/oapicode
./oapi-generator/openapi-generator-cli generate -g rust-axum -i $(pwd)/docs/specs/openapi.yml -o $(pwd)/database/oapicode