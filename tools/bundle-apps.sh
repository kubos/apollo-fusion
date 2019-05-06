#! /bin/bash
#
# Copyright (C) 2019 Kubos Corporation
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

# Build and bundle all the apps into a tar file
#
# The tar file also includes `install-apps.sh` which will run the necessary app service commands
# on the OBC to install each app

set -e

DIR=$PWD
OUTPUT_DIR=${DIR}/apps
APPS_DIR=${DIR}/../apps
TARGET_DIR=${APPS_DIR}/target/arm-unknown-linux-gnueabihf/release

mkdir -p ${OUTPUT_DIR}
cd ${APPS_DIR}

### Build all the Rust-based apps

# Get the latest version of the Kubos repo to build with
#cargo update

for app in "beacon-app" "deploy-app" "telem-app" "obc-hs" ;
do
    # Create the final output directory
    mkdir -p ${OUTPUT_DIR}/${app}
    # Build the app
    cd ${APPS_DIR}/${app}
    PKG_CONFIG_ALLOW_CROSS=1 CC=/usr/bin/bbb_toolchain/usr/bin/arm-linux-gcc cargo build --release --target arm-unknown-linux-gnueabihf
    # Shrink it down
    arm-linux-strip ${TARGET_DIR}/${app}
    # Copy the final files to the output directory
    cp manifest.toml ${OUTPUT_DIR}/${app}
    cp ${TARGET_DIR}/${app} ${OUTPUT_DIR}/${app}
done

### Copy all the Python-based apps

for app in "adcs-hs" ;
do 
    cd ${APPS_DIR}/${app}
    # Create the final output directory
    mkdir -p ${OUTPUT_DIR}/${app}
    # Copy everything to the output directory
    cp -r * ${OUTPUT_DIR}/${app}
done

### Tar everything up for easy transportation

cd ${DIR}
tar -czf apps-$(date +%Y.%m.%d).tar.gz apps install-apps.sh

### Cleanup

rm ${OUTPUT_DIR} -R