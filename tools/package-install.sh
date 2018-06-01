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
# Copy the packages which need to be manually installed into the Apollo Fusion stack
# onto the board
#
# Usage: package-install.sh ip-addr
#
# - ip-addr: The IP address of the stack
#

pass = 'sshpass -p "Kubos123"'

$pass scp -r $HOME/.kubos/kubos/hal/python-hal/i2c kubos@($1):/home/kubos
$pass scp -r $HOME/.kubos/kubos/libs/kubos_service kubos@($1):/home/kubos
$pass scp -r $HOME/.kubos/kubos/apis/pumpkin_mcu_api kubos@($1):/home/kubos
$pass scp -r $HOME/.kubos/kubos/services/pumpkin-mcu-service-udp kubos@($1):/home/system/usr/sbin

$pass ssh kubos@($1) 'python /home/kubos/i2c/setup.py install; rm /home/kubos/i2c -R'
$pass ssh kubos@($1) 'python /home/kubos/kubos_service/setup.py install; /home/kubos/kubos_service -R'
$pass ssh kubos@($1) 'python /home/kubos/pumpkin_mcu_api/setup.py install; /home/kubos/pumpkin_mcu_api -R'