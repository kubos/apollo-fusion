#!/bin/bash
#
# Copyright (C) 2017 Kubos Corporation
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

cd ../../buildroot-2017.02.8
sudo make BR2_EXTERNAL=../kubos-linux-build:../apollo-fusion apollo-fusion_defconfig
sudo make
./../kubos-linux-build/tools/kubos-package.sh -t pumpkin-mbm2 -o output -v kpack-base.itb -k
./../kubos-linux-build/tools/format-aux.sh -i kpack-base.itb

# TODO: zip the files so they're not massive...