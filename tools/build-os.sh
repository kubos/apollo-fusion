#!/bin/bash
#
# Copyright (C) 2018 Kubos Corporation
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
#
# Helper script to perform a full Kubos Linux build for the ApolloFusion stack
# Creates a tar.gz file containing the Kubos Linux and Aux SD images
# 
# Note: This script must be run from *this* folder due to the relative paths used

set -e

DATE=$(date +"%b-%d-%Y")
DIR=$PWD
ROOT_DIR=$DIR/../..
BR_DIR=$ROOT_DIR/buildroot-2017.02.8
KLB_DIR=$ROOT_DIR/kubos-linux-build

# Download BuildRoot and KLB_DIR
cd $ROOT_DIR
wget https://buildroot.uclibc.org/downloads/buildroot-2017.02.8.tar.gz && tar xvzf buildroot-2017.02.8.tar.gz && rm buildroot-2017.02.8.tar.gz
git clone https://github.com/kubos/kubos-linux-build

# Build the Kubos Linux image
cd $BR_DIR
sudo make BR2_EXTERNAL=../kubos-linux-build:../apollo-fusion apollo-fusion_defconfig
sudo make
sudo chmod +7 output/images/*
cp output/images/kubos-linux.tar.gz $DIR
# Delete the .img file to free disk space back up
rm output/images/kubos-linux.img

# Build the auxiliary SD card image
cd $KLB_DIR/tools
./kubos-package.sh -t pumpkin-mbm2 -o output -v kpack-base.itb -k
sudo ./format-aux.sh -i kpack-base.itb
tar -czf aux-sd.tar.gz aux-sd.img
cp aux-sd.tar.gz $DIR
# Delete the .img file to free disk space back up
rm aux-sd.img

# Package it all up in a nice small tar file
cd $DIR
tar -czf ApolloFusion-$DATE.tar.gz kubos-linux.tar.gz aux-sd.tar.gz

# Cleanup the temporary copies we made
rm kubos-linux.tar.gz
rm aux-sd.tar.gz

exit 0