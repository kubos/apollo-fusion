#!/bin/sh
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

# Register all apps located in the `apps` directory with the apps service
#
# Note: Meant to be run on an OBC

APPS_DIR=$PWD/apps
IP="0.0.0.0"
PORT=8000
URL="${IP}:${PORT}"

for app in $(ls apps) ;
do
    curl ${URL} -H "Content-Type: application/json" -d "{\"query\":\"mutation{register(path:\\\"${APPS_DIR}/${app}\\\"){success,errors}}\"}" 
done