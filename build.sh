#!/bin/bash

set -e -o pipefail

buildroot_tar="buildroot-2017.02.8.tar.gz"
buildroot_url="https://buildroot.uclibc.org/downloads/$buildroot_tar"

echo "Building for Board: apollo-fusion"

cd .. #cd out of the apollo-fusion directory

echo "Getting kubos-linux-build"

git clone https://github.com/kubos/kubos-linux-build

latest_tag=`git tag --sort=-creatordate | head -n 1`
sed -i "s/0.0.0/$latest_tag/g" kubos-linux-build/common/linux-kubos.config

echo "Getting buildroot"

wget $buildroot_url && tar xzf $buildroot_tar && rm $buildroot_tar

cd ./buildroot*

make BR2_EXTERNAL=../kubos-linux-build:../apollo-fusion apollo-fusion_defconfig

echo "STARTING BUILD"

make