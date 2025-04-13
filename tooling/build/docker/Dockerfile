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
### STAGE 1: Build Base with System Dependencies  ###
#####################################################
FROM registry.cn-shenzhen.aliyuncs.com/wl4g/rust:1.85 AS base

ARG BUILD_MIRROR_URL=http://mirrors.aliyun.com
ARG DEBIAN_FRONTEND=noninteractive

# Set up fast APT sources and such as:
#   Debian Official Default: http://deb.debian.org
#   Alibaba Cloud Internal: http://mirrors.cloud.aliyuncs.com
#   Alibaba Cloud External: http://mirrors.aliyun.com
#
# Notice: If you use modsecurity-1.0.0, you must rely on libmodsecurity>=3.0.13. For Linux, 
# see: /usr/lib/x86_64-linux-gnu/pkgconfig/modsecurity.pc. Rust-1.85 is the latest version of 
# the default source of debian-12 and only supports libmodsecurity-3.0.9.
RUN rm -f /etc/apt/sources.list.d/debian.sources && \
    echo "deb ${BUILD_MIRROR_URL}/debian bookworm main contrib non-free non-free-firmware" > /etc/apt/sources.list && \
    echo "deb ${BUILD_MIRROR_URL}/debian-security bookworm-security main contrib non-free non-free-firmware" >> /etc/apt/sources.list && \
    echo "deb ${BUILD_MIRROR_URL}/debian bookworm-updates main contrib non-free non-free-firmware" >> /etc/apt/sources.list && \
    apt-get update && \
    apt-get install -y --no-install-recommends \
        libmodsecurity3 \
        libmodsecurity-dev \
        pkg-config \
        ca-certificates \
        && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /usr/src/app

#################################################
### STAGE 2: Build Dependencies Cached Layer  ###
#################################################
FROM base AS deps

ARG PROJECT_NAME=botwaf

# Copy only files needed for dependency resolution. (Excluded: 'rust-toolchain.toml' due to containing dev components.)
COPY benchmarks ./benchmarks
COPY src ./src
COPY Cargo.toml Cargo.lock ./

# Setup dependency caching structure
RUN find src -type f -not -name "Cargo.toml" -delete && \
    find benchmarks -type f -not -name "Cargo.toml" -delete && \
    find src -type d -name src -o -path "*/src" | xargs -I{} mkdir -p {} && \
    find benchmarks -type d -name src -o -path "*/src" | xargs -I{} mkdir -p {} && \
    find src -type d -name src | xargs -I{} touch {}/lib.rs && \
    find benchmarks -type d -name src | xargs -I{} touch {}/lib.rs && \
    mkdir -p src/cmd/src/bin && \
    echo "fn main() {}" > src/cmd/src/bin/${PROJECT_NAME}.rs && \
    ls -R /usr/src/app

# Build dependencies only
RUN cargo build --release && \
    find . -name "*.rs" -delete

################################################
### STAGE 3: Build Application Binary        ###
################################################
FROM base AS builder

ARG PROJECT_NAME=botwaf
ARG BUILD_ARGS=""

# Copy cached dependencies
COPY --from=deps /usr/src/app/target target
COPY --from=deps /usr/local/cargo /usr/local/cargo

# Copy all source code and project files
COPY benchmarks ./benchmarks
COPY etc ./etc
COPY src ./src
COPY static ./static
COPY tooling ./tooling
COPY Cargo.toml Cargo.lock ./

# Check toolchain and build the application
RUN cargo install ${BUILD_ARGS} --path src/cmd

#################################################
### STAGE 4: Runtime Image                    ###
#################################################
FROM registry.cn-shenzhen.aliyuncs.com/wl4g/debian:bookworm

# Build project metadata labels.
ARG PROJECT_NAME=botwaf
ARG PROJECT_REPO="https://github.com/wl4g/botwaf"
ARG PROJECT_DESC="Botwaf - A Mini Open Source AI Bot WAF written in Rust."
ARG PROJECT_MAINTAINER="James Wong<jameswong1376@gmail.com>"
ARG BUILD_COMMIT_ID=""
ARG BUILD_BRANCH=""
ARG BUILD_TAG=""
ARG BUILD_TIME=""
# Build args configuration.
ARG DEBIAN_FRONTEND=noninteractive
ARG BUILD_MIRROR_URL=http://mirrors.aliyun.com
ARG BUILD_ARGS=""

LABEL maintainer="${PROJECT_MAINTAINER}" \
      description="${PROJECT_DESC}" \
      org.opencontainers.image.title=${PROJECT_NAME} \
      org.opencontainers.image.source=${PROJECT_REPO} \
      org.opencontainers.image.revision=${BUILD_COMMIT_ID} \
      org.opencontainers.image.version=${BUILD_TAG} \
      org.opencontainers.image.created=${BUILD_TIME} \
      build.repo.url=${PROJECT_REPO} \
      build.repo.commit=${BUILD_COMMIT_ID} \
      build.repo.branch=${BUILD_BRANCH} \
      build.repo.tag=${BUILD_TAG} \
      build.deps.mirror.url=${BUILD_MIRROR_URL}

# Install runtime dependencies
RUN rm -f /etc/apt/sources.list.d/debian.sources && \
    echo "deb ${BUILD_MIRROR_URL}/debian bookworm main contrib non-free non-free-firmware" > /etc/apt/sources.list && \
    echo "deb ${BUILD_MIRROR_URL}/debian-security bookworm-security main contrib non-free non-free-firmware" >> /etc/apt/sources.list && \
    echo "deb ${BUILD_MIRROR_URL}/debian bookworm-updates main contrib non-free non-free-firmware" >> /etc/apt/sources.list && \
    apt-get update && \
    apt-get install -y --no-install-recommends \
        tini \
        libmodsecurity3 \
        libssl3 \
        ca-certificates \
        && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# Create non-root user for security
RUN groupadd -r ${PROJECT_NAME} && \
    useradd -r -g ${PROJECT_NAME} ${PROJECT_NAME} && \
    mkdir -p /app/config /app/data /app/static && \
    chown -R ${PROJECT_NAME}:${PROJECT_NAME} /app

# Copy the application binary
COPY --from=builder /usr/local/cargo/bin/${PROJECT_NAME} /usr/local/bin/${PROJECT_NAME}

# Copy configuration and static files
COPY --from=builder /usr/src/app/etc /app/config
COPY --from=builder /usr/src/app/static /app/static

# Setup working directory
WORKDIR /app

# Set the run entrypoint using tini as init
ENTRYPOINT ["/usr/bin/tini", "-s", "--", "/usr/local/bin/${PROJECT_NAME}"]

# Default command if none provided
CMD ["--config", "/app/config/botwaf.yaml"]