#!/bin/bash
# SPDX-License-Identifier: GNU GENERAL PUBLIC LICENSE Version 3
#
# Copyleft (c) 2024 James Wong. This file is part of James Wong.
# is free software: you can redistribute it and/or modify it under
# the terms of the GNU General Public License as published by the
# Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
#
# James Wong is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License
# along with James Wong.  If not, see <https://www.gnu.org/licenses/>.
#
# IMPORTANT: Any software that fully or partially contains or uses materials
# covered by this license must also be released under the GNU GPL license.
# This includes modifications and derived works.
#
export BASE_DIR=$(cd "`dirname $0`"; pwd)

echo "Gzip unpack the raw/* ..."
gzip -d ${BASE_DIR}/raw/*

echo "Extract normal threats logs..."

cat ${BASE_DIR}/raw/access.log* | \
grep -i -E "\
Mozilla/5\.0 \(Macintosh; Intel Mac OS X|\
\+0800\] \"GET|\
HTTP/1\.1\" 200|\
SSTP_DUPLEX_POST|\
blogs\.wl4g\.com|\
excalidraw\.wl4g\.com|\
mqtt\.wl4g\.com|\
iam\.wl4g\.com|\
im\.wl4g\.com|\
unami\.wl4g\.com" | \
grep -v -i -E "\
Mozilla/5\.0 \(compatible; CensysInspect|\
Expanse, a Palo Alto Networks company|\
spider|\
Custom-AsyncHttpClient|\
\+0800\] \"PRI|\
\+0800\] \"HEAD /|\
\+0800\] \"HEAD /old|\
\+0800\] \"HEAD /new|\
\+0800\] \"HEAD /main|\
\+0800\] \"HEAD /home|\
\+0800\] \"HEAD /wp|\
\+0800\] \"HEAD /bc|\
\+0800\] \"HEAD /bk|\
\+0800\] \"HEAD /backup|\
\+0800\] \"HEAD /wordpress|\
GET /my_env|\
GET /mytest|\
GET /appsettings.json|\
GET /\.travis.yml|\
GET /aws\.yml|\
GET /sms\.py|\
GET /config/|\
GET /main\.yml|\
GET /server/s3\.js|\
GET /s3\.js|\
GET /\.aws|\
passport\.wl4g\.com|\
\.com\.wl4g\.com|\
sunwuu\.wl4g\.com|\
mail\.wl4g\.com|\
online\.wl4g\.com|\
pm\.wl4g\.com|\
console\.wl4g\.com|\
wp\.wl4g\.com|\
static\.wl4g\.com|\
dev\.wl4g\.com|\
api\.wl4g\.com" > ${BASE_DIR}/samples/access.valid.log

echo "Extract if possible threats logs..."

cat ${BASE_DIR}/raw/access.log* | \
grep -i -E "\
Mozilla/5\.0 \(compatible; CensysInspect|\
Expanse, a Palo Alto Networks company|\
spider|\
Custom-AsyncHttpClient|\
\+0800\] \"PRI|\
\+0800\] \"HEAD /|\
\+0800\] \"HEAD /old|\
\+0800\] \"HEAD /new|\
\+0800\] \"HEAD /main|\
\+0800\] \"HEAD /home|\
\+0800\] \"HEAD /wp|\
\+0800\] \"HEAD /bc|\
\+0800\] \"HEAD /bk|\
\+0800\] \"HEAD /backup|\
\+0800\] \"HEAD /wordpress|\
GET /my_env|\
GET /mytest|\
GET /appsettings.json|\
GET /\.travis.yml|\
GET /aws\.yml|\
GET /sms\.py|\
GET /config/|\
GET /main.yml|\
GET /server/s3\.js|\
GET /s3\.js|\
GET /\.aws|\
passport\.wl4g\.com|\
\.com\.wl4g\.com|\
sunwuu\.wl4g\.com|\
mail\.wl4g\.com|\
online\.wl4g\.com|\
pm\.wl4g\.com|\
console\.wl4g\.com|\
wp\.wl4g\.com|\
static\.wl4g\.com|\
dev\.wl4g\.com|\
api\.wl4g\.com" > ${BASE_DIR}/samples/access.invalid.log

echo "Gzip pack the raw/* ..."
gzip ${BASE_DIR}/raw/*

echo "Gzip pack the samples/* ..."
gzip ${BASE_DIR}/samples/*
