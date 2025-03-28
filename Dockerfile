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
### Stage 1: Infrastructure libraries installation. ###
FROM registry.cn-shenzhen.aliyuncs.com/wl4g/rust:1.85 as base

# Set up fast apt sources. (internal: http://mirrors.cloud.aliyuncs.com, external: http://mirrors.aliyun.com)
#
# Notice: If you use modsecurity-1.0.0, you must rely on libmodsecurity>=3.0.13. For Linux, 
# see: /usr/lib/x86_64-linux-gnu/pkgconfig/modsecurity.pc. Rust-1.85 is the latest version of 
# the default source of debian-12 and only supports libmodsecurity-3.0.9.
#
RUN echo > /etc/apt/sources.list && \
    echo 'deb http://mirrors.aliyun.com/debian bookworm main contrib non-free non-free-firmware' > /etc/apt/sources.list && \
    echo 'deb-src http://mirrors.aliyun.com/debian bookworm main contrib non-free non-free-firmware' >> /etc/apt/sources.list && \
    echo 'deb http://mirrors.aliyun.com/debian-security bookworm-security main contrib non-free non-free-firmware' >> /etc/apt/sources.list && \
    echo 'deb-src http://mirrors.aliyun.com/debian-security bookworm-security main contrib non-free non-free-firmware' >> /etc/apt/sources.list && \
    echo 'deb http://mirrors.aliyun.com/debian bookworm-updates main contrib non-free non-free-firmware' >> /etc/apt/sources.list && \
    echo 'deb-src http://mirrors.aliyun.com/debian bookworm-updates main contrib non-free non-free-firmware' >> /etc/apt/sources.list && \
    apt-get update && \
    apt-get install -y pkg-config && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

### Stage 2: Dependencies installation (for caching dependencies) ###
FROM base as deps
WORKDIR /usr/src/botwaf
# Just copy the files used to build the dependencies.
COPY Cargo.toml Cargo.lock ./
# Create an empty main.rs that triggers dependency install but does not build the actual project.
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

### Stage 3: Application build. ###
FROM base as builder
WORKDIR /usr/src/botwaf
# Copy the build cache of dependency Stage 2
COPY --from=deps /usr/src/botwaf/target target
COPY --from=deps /usr/local/cargo /usr/local/cargo
# Copy the rest source files of the project
COPY . .
# Compile the application source files.
RUN cargo install --path .

### Stage 4: Application runtime. ###
FROM registry.cn-shenzhen.aliyuncs.com/wl4g/debian:bookworm
# Set up fast apt sources. (internal: http://mirrors.cloud.aliyuncs.com, external: http://mirrors.aliyun.com)
RUN echo 'deb http://mirrors.aliyun.com/debian bookworm main contrib non-free non-free-firmware' > /etc/apt/sources.list && \
    echo 'deb http://mirrors.aliyun.com/debian-security bookworm-security main contrib non-free non-free-firmware' >> /etc/apt/sources.list && \
    echo 'deb http://mirrors.aliyun.com/debian bookworm-updates main contrib non-free non-free-firmware' >> /etc/apt/sources.list && \
    apt-get update && \
    apt-get install -y libssl3 && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# Copy the application build binary executable.
COPY --from=builder /usr/local/cargo/bin/botwaf /usr/local/bin/botwaf

# Set the run entrypoint of the container.
ENTRYPOINT ["botwaf"]
