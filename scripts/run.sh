#!/bin/bash

export BASE_DIR=$(cd "`dirname $0`"; pwd; cd -)
export SWAGGER_UI_DOWNLOAD_URL=file:$BASE_DIR/etc/swagger-ui-5.17.14.zip
export RUSTFLAGS="-C debug-prefix-map=$(pwd)=."

cargo build
