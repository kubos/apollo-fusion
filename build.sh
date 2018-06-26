#!/bin/bash

set -e -o pipefail

buildroot_tar="buildroot-2018.02.3.tar.gz"
buildroot_url="https://buildroot.uclibc.org/downloads/$buildroot_tar"

echo "Building for Board: apollo-fusion"

cd .. #cd out of the apollo-fusion directory

kubos update

echo "Getting kubos-linux-build"

git clone https://github.com/kubos/kubos-linux-build -b br-upgrade

echo "Getting buildroot"

wget $buildroot_url && tar xzf $buildroot_tar && rm $buildroot_tar

cd ./buildroot*

make BR2_EXTERNAL=../kubos-linux-build:../apollo-fusion apollo-fusion_defconfig

echo "STARTING BUILD"

make

echo "Creating Aux SD image"

cd ../kubos-linux-build/tools
./kubos-package.sh -t pumpkin-mbm2 -o output -v kpack-base.itb -k
sudo ./format-aux.sh -i kpack-base.itb
tar -czf aux-sd.tar.gz aux-sd.img
# Delete the .img file to free disk space back up
rm aux-sd.img