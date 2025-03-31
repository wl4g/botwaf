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

#####################################################
### STAGE 1: Build Infra Libraries Tooling Layer. ###
#####################################################
FROM registry.cn-shenzhen.aliyuncs.com/wl4g/rust:1.85 AS base

ARG BUILD_MIRROR_URL
RUN echo "[STAGE 1] Using the BUILD_MIRROR_URL: ${BUILD_MIRROR_URL}"

# Set up fast apt sources and such as:
#   Debian Official Default: http://deb.debian.org
#   Alibaba Cloud Internal: http://mirrors.cloud.aliyuncs.com
#   Alibaba Cloud External: http://mirrors.aliyun.com
#
# Notice: If you use modsecurity-1.0.0, you must rely on libmodsecurity>=3.0.13. For Linux, 
# see: /usr/lib/x86_64-linux-gnu/pkgconfig/modsecurity.pc. Rust-1.85 is the latest version of 
# the default source of debian-12 and only supports libmodsecurity-3.0.9.
#
RUN echo > /etc/apt/sources.list && \
    echo "deb ${BUILD_MIRROR_URL}/debian bookworm main contrib non-free non-free-firmware" > /etc/apt/sources.list && \
    echo "deb-src ${BUILD_MIRROR_URL}/debian bookworm main contrib non-free non-free-firmware" >> /etc/apt/sources.list && \
    echo "deb ${BUILD_MIRROR_URL}/debian-security bookworm-security main contrib non-free non-free-firmware" >> /etc/apt/sources.list && \
    echo "deb-src ${BUILD_MIRROR_URL}/debian-security bookworm-security main contrib non-free non-free-firmware" >> /etc/apt/sources.list && \
    echo "deb ${BUILD_MIRROR_URL}/debian bookworm-updates main contrib non-free non-free-firmware" >> /etc/apt/sources.list && \
    echo "deb-src ${BUILD_MIRROR_URL}/debian bookworm-updates main contrib non-free non-free-firmware" >> /etc/apt/sources.list && \
    apt-get update && \
    apt-get install -y libmodsecurity3 libmodsecurity-dev pkg-config && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

#################################################
### STAGE 2: Build Dependencies Cached Layer. ###
#################################################
FROM base AS deps
WORKDIR /usr/src/botwaf
# Copy cargo configuration files and directory structure
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY benchmarks ./benchmarks
# Remove all files except Cargo.toml
# 1. Create minimal source structure for each crate
# 2. Create root main.rs
# 3. Build all dependencies
# 4. Clean up all source files
RUN find src -type f -not -name "Cargo.toml" -delete && \
    find src -type d -name src -o -path "*/src" | xargs -I{} mkdir -p {} && \
    find src -type d -name src | xargs -I{} touch {}/lib.rs && \
    mkdir -p src/cmd/src/bin && \
    echo "fn main() {}" > src/cmd/src/bin/botwaf.rs && \
    ls -R /usr/src/botwaf && \
    cargo build --release && \
    find . -name "*.rs" -delete

################################################
### STAGE 3: Build Application Source Layer. ###
################################################
FROM base AS builder

ARG BUILD_ARGS
RUN echo "[STAGE 3] Using the BUILD_ARGS: ${BUILD_ARGS}"

WORKDIR /usr/src/botwaf
# Copy the build cache of dependency Stage 2
COPY --from=deps /usr/src/botwaf/target target
COPY --from=deps /usr/local/cargo /usr/local/cargo
# Copy the rest source files of the project
COPY . .
# Compile the application source files.
RUN cargo install ${BUILD_ARGS} --path .

#################################################
### STAGE 4: Build Application Runtime Layer. ###
#################################################
FROM registry.cn-shenzhen.aliyuncs.com/wl4g/debian:bookworm

ARG BUILD_ARGS
ARG BUILD_REPO_URL
ARG BUILD_BRANCH
ARG BUILD_TAG
ARG BUILD_COMMIT_ID
ARG BUILD_TIME
ARG BUILD_MIRROR_URL
RUN echo "[STAGE 4] Using the BUILD_ARGS: ${BUILD_ARGS}, BUILD_MIRROR_URL: ${BUILD_MIRROR_URL}"

LABEL maintainer="James Wong<jameswong1376@gmail.com>" \
  description="Botwaf - A Mini Open Source AI Bot WAF written in Rust." \
  buildArgs=${BUILD_ARGS} \
  buildRepoUrl=${BUILD_REPO_URL} \
  buildBranch=${BUILD_BRANCH} \
  buildTag=${BUILD_TAG} \
  buildCommitId=${BUILD_COMMIT_ID} \
  buildTime=${BUILD_TIME} \
  buildMirrorURL=${BUILD_MIRROR_URL}

# Set up fast apt sources. (internal: http://mirrors.cloud.aliyuncs.com, external: http://mirrors.aliyun.com)
RUN echo "deb ${BUILD_MIRROR_URL}/debian bookworm main contrib non-free non-free-firmware" > /etc/apt/sources.list && \
    echo "deb ${BUILD_MIRROR_URL}/debian-security bookworm-security main contrib non-free non-free-firmware" >> /etc/apt/sources.list && \
    echo "deb ${BUILD_MIRROR_URL}/debian bookworm-updates main contrib non-free non-free-firmware" >> /etc/apt/sources.list && \
    apt-get update && \
    apt-get install -y tini libmodsecurity3 libssl3 && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# Copy the application build binary executable.
COPY --from=builder /usr/local/cargo/bin/botwaf /usr/local/bin/botwaf

# Set the run entrypoint of the container.
ENTRYPOINT ["/sbin/tini", "-s", "-g", "--", "botwaf"]