#/bin/sh
# This script will generate the code for the collector and database from the OpenAPI spec
# It will download the openapi-generator-cli if it is not already present
# The generated code will be placed in the collector/oapicode and database/oapicode directories

DIRECTORY="oapi-generator"

echo "Checking for openapi-generator-cli"

if [ -d "$DIRECTORY" ]; then
    echo "Directory $DIRECTORY found!"
else
    echo "Creating $DIRECTORY directory"
    mkdir "$DIRECTORY"
    cd "$DIRECTORY"
    export OPENAPI_GENERATOR_VERSION="7.11.0"
    curl https://raw.githubusercontent.com/OpenAPITools/openapi-generator/master/bin/utils/openapi-generator-cli.sh > openapi-generator-cli
    chmod u+x openapi-generator-cli
    cd ..
fi


./$DIRECTORY/openapi-generator-cli generate -g python -i $(pwd)/docs/specs/openapi.yml -o $(pwd)/collector/oapicode
./$DIRECTORY/openapi-generator-cli generate -g python -i $(pwd)/docs/specs/openapi.yml -o $(pwd)/webserver/oapicode

./$DIRECTORY/openapi-generator-cli generate -g rust-axum -i $(pwd)/docs/specs/openapi.yml -o $(pwd)/database/oapicode
