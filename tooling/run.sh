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

set -e

BASE_DIR="$(cd "`dirname $0`"/../; pwd)"
# If run.sh is a soft link, it is considered to be $PROJECT_HOME/run.sh, no need to call back the path.
if [ -L "`dirname $0`/run.sh" ]; then
  BASE_DIR="$(cd "`dirname $0`"; pwd)"
fi

# eg1: log "error" "Failed to xxx"
# eg2: log "xxx complete!"
function log() {
  local logLevel=" \033[33mINFO\033[0m"
  local logContent=$1
  if [[ $# > 1 ]]; then
    logLevel=$1
    logContent=$2
  fi
  local logMsg="[$logLevel] $(date '+%Y-%m-%d %H:%M:%S') - $logContent"
  echo -e "$logMsg"
  echo -e "$logMsg" >> /tmp/run-builder.log
}

function logDebug() {
  log "\033[37mDEBUG\033[0m" "$@"
}

function logWarn() {
  log "\033[33mWARN \033[0m" "$@"
}

function logErr() {
  log "\033[31mERROR\033[0m" "$@"
}

function usages() {
    echo $"Botwaf Development and Building and Deployment fast Tooling.
# for examples
export GPG_PRIVATE_KEY='-----BEGIN PGP PRIVATE KEY BLOCK-----\n...'
export GPG_PASSPHRASE='<YOUR_GPG_PASSPHRASE>'
export IMAGE_REGISTRY='docker.io/wl4g' # eg: docker.io/wl4g(default), ghcr.io/wl4g, registry.cn-shenzhen.aliyuncs.com/wl4g, ccr.ccs.tencentyun.com/wl4g
export IMAGE_USERNAME='<YOUR_REGISTRY_USER>'
export IMAGE_TOKEN='<YOUR_REGISTRY_TOKEN>'

Usage: ./$(basename $0) [OPTIONS] [arg1] [arg2] ...
    version                                         Print APP project version.
    gpg-verify                                      Installing and Verifying GPG keys on Linux only.
    build-on-host                                   Build executable binary file on Host.
    #build-deploy                                   Build and deploy to Crate central.
    build-image                                     Build component images.
                        -b,--backend                Build image for Backend component.
                        -f,--frontend               Build image for Frontend component.
                        -d,--initdb                 Build image for Init DB.
                        -A,--all                    Build image for All Artifacts.
    push-image                                      Push component images.
                        -b,--backend                Push image for Backend component.
                        -f,--frontend               Push image for Frontend component.
                        -d,--initdb                 Push image for Init DB.
                        -A,--all                    Push image for All components.
    build-push                                      Build with Crate and push images for All components.
    prune-image                                     Prune unused all images. (tag=none)
    deploy-docker                                   Deploy all services with docker compose mode.
                        -S,--status                 Display status for all services.
                        -U,--up                     Startup to all services.
                           --prune-all-volumes      Remove all data volumes before per initial deploy. (Note: be careful! development only)
                        -D,--down                   Shuwdown to all services.
                           --prune-all-volumes      Remove all data volumes after per destory deploy. (Note: be careful! development only)
"
}

function tools::get_host_ip() {
    echo $(ip addr | grep -E '^[0-9]+: (em|eno|enp|ens|eth|wlp|en)+[0-9]' -A2 | grep inet | awk -F ' ' '{print $2}' | cut -f1 -d/ | xargs echo)
}

function tools::get_project_version() {
    echo $(cat ${BASE_DIR}/Cargo.toml | sed 's/ //g' | grep -E '^version="(.+)"$' | awk -F '=' '{print $2}' | sed 's/"//g')
}

function tools::gpg_install_verify() {
    # GPG checks are supported only on Linux, such as when pushing a compilation library to a central repository.
    if [[ ! -f "/bin/bash" ]]; then
        logWarn "The Linux OS is not detected, skip GPG verification."
        return
    fi
    if [ -z $(command -v gpg) ]; then
        logErr "The GPG is not installed, please install GPG first."; exit 1
    fi

    log "Checking for GPG version ..."
    gpg_version=$(gpg --version | head -1 | grep -iEo '(([0-9]+)\.([0-9]+)\.([0-9]+))') # eg: 2.2.19
    gpg_version_major=$(echo $gpg_version | awk -F '.' '{print $1}')
    gpg_version_minor=$(echo $gpg_version | awk -F '.' '{print $2}')
    gpg_version_revision=$(echo $gpg_version | awk -F '.' '{print $3}')
    if [[ ! ("$gpg_version_major" -ge 2 && "$gpg_version_minor" -ge 1) ]]; then
      logErr "The GPG version must >= $gpg_version_major.$gpg_version_minor.x"; exit 1
    fi

    # If the GPG key has already been generated, it is skipped.
    log "Configuring for GPG keys ..."
    if [[ ! -f "$HOME/.gnupg/pubring.kbx" ]]; then
        if [[ -z "$GPG_PRIVATE_KEY" ]]; then
            logErr "The environment variable GPG_PRIVATE_KEY is missing."; exit 1
        fi

        \rm -rf ~/.gnupg/; mkdir -p ~/.gnupg/private-keys-v1.d/; chmod -R 700 ~/.gnupg/
        echo -n "$GPG_PRIVATE_KEY" > /tmp/private.key

        #logDebug "----- Print GPG secret key (debug) -----"
        #cat /tmp/private.key

        # FIXED:https://github.com/keybase/keybase-issues/issues/2798#issue-205008630
        #export GPG_TTY=$(tty) # Notice: github action the VM instance no tty.

        # FIXED:https://bbs.archlinux.org/viewtopic.php?pid=1691978#p1691978
        # FIXED:https://github.com/nodejs/docker-node/issues/922
        # Note that since Version 2.0 this passphrase is only used if the option --batch has also
        # been given. Since Version 2.1 the --pinentry-mode also needs to be set to loopback.
        # see:https://www.gnupg.org/documentation/manuals/gnupg/GPG-Esoteric-Options.html#index-allow_002dsecret_002dkey_002dimport
        gpg2 -v --pinentry-mode loopback --batch --secret-keyring ~/.gnupg/secring.gpg --import /tmp/private.key

        logDebug "Cleanup to /tmp/private.key ..."
        \rm -rf /tmp/private.key
        ls -al ~/.gnupg/

        logDebug "----- Imported list of GPG secret keys -----"
        gpg2 --list-keys
        gpg2 --list-secret-keys
    fi

    # Notice: Test signing should be performed first to ensure that the gpg-agent service has been 
    # pre-started (gpg-agent --homedir /root/.gnupg --use-standard-socket --daemon), otherwise
    # an error may be reported : 'gpg: signing failed: Inappropriate ioctl for device'
    if [[ -z "$GPG_PASSPHRASE" ]]; then
        logErr "The environment variable GPG_PASSPHRASE is missing."; exit 1
    fi
    logDebug "Prepare verifying the GPG signing ..."
    echo "test" | gpg2 -v --pinentry-mode loopback --passphrase $GPG_PASSPHRASE --clear-sign
}

function tools::get_fast_mirror_url() {
    # Detect to running on Cloud providers.
    # Alibaba Cloud: http://100.100.100.200/latest/meta-data/instance-id
    # Google Cloud: http://metadata.google.internal/computeMetadata/v1/instance/zone
    # AWS: http://169.254.169.254/latest/meta-data/instance-id
    # Azure: http://169.254.169.254/metadata/instance/compute/location
    if curl -s --max-time 1 http://100.100.100.200/latest/meta-data/instance-id > /dev/null 2>&1; then
        MIRROR_URL="http://mirrors.cloud.aliyuncs.com"
    else
        # Check if the machine is in China.
        if [ -z "$(curl -s --max-time 5 https://ipinfo.io | sed 's/ //g' | grep '"country":"CN"' > /dev/null 2>&1)" ]; then
            MIRROR_URL="http://mirrors.aliyun.com"
        else
            MIRROR_URL="http://deb.debian.org" # Use the default debian mirrors.
        fi
    fi
    echo $MIRROR_URL
}

function tools::prune_none_image() {
    local prune_images=$(docker images | grep none | awk -F ' ' '{print $3}')
    for pi in `echo $prune_images`; do
        echo "Removing image the $pi "
        docker rmi -f $pi
    done
}

function build::binary_on_host() {
    local build_args="$@"
    log "Build binary with args: $build_args..."
    SWAGGER_UI_DOWNLOAD_URL=file:$BASE_DIR/deps/swagger-ui-5.17.14.zip && \
    RUSTFLAGS="-C debug-prefix-map=$(pwd)=." && \
    cargo build --features deadlock_detection,profiling-mem-prof,profiling-pprof,profiling-tokio-console,profiling-pyroscope $build_args
    log "Finished build binary!"
}

function build::docker_image() {
    local image_name="$1"
    local dockerfile="$2"
    local image_tag="$(tools::get_project_version)"
    local mirror_url=$(tools::get_fast_mirror_url)
    if [ -z "$image_name" ]; then
        logErr "The arg 0 IMAGE_NAME is missing."; exit 1
    fi
    if [ -z "$dockerfile" ]; then
        logErr "The arg 1 DOCKER_FILE is missing."; exit 1
    fi
    log "Docker building to wl4g/${image_name}:${image_tag} with ${BASE_DIR}, ${mirror_url} ..."
    docker buildx build \
        -t wl4g/${image_name}:${image_tag} \
        -f ${BASE_DIR}/tooling/build/docker/${dockerfile} \
        --platform=amd64 \
        --build-arg BUILD_ARGS="--features deadlock_detection,profiling-mem-prof,profiling-pprof,profiling-tokio-console,profiling-pyroscope" \
        --build-arg BUILD_REPO_URL=$(git remote -v | head -1 | awk -F ' ' '{print $2}') \
        --build-arg BUILD_COMMIT_ID=$(git log | head -1 | awk -F ' ' '{print $2}' | cut -c 1-12) \
        --build-arg BUILD_BRANCH=$(git rev-parse --abbrev-ref HEAD) \
        --build-arg BUILD_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "Unkown") \
        --build-arg BUILD_TIME=$(date +'%Y%m%dT%H%M%S') \
        --build-arg BUILD_MIRROR_URL=${mirror_url} \
        ${BASE_DIR}/
}

function build:push_image() {
    local image_registry="$IMAGE_REGISTRY"
    local image_name="$1"
    local image_tag="$(tools::get_project_version)"

    if [ -z "$image_registry" ]; then
        image_registry="docker.io/wl4g"
    fi
    if [ "$(docker login >/dev/null 2>&1; echo $?)" -ne 0 ]; then
        if [[ -z "$IMAGE_USERNAME" || -z "$IMAGE_TOKEN" ]]; then
            logWarn "The environment variable IMAGE_USERNAME or IMAGE_TOKEN is missing."; exit 1
        else
            logDebug "Login to $image_registry ..."
            docker login -u $IMAGE_USERNAME -p $IMAGE_TOKEN $image_registry
        fi
    else
        logDebug "Already login docker hub."
    fi

    logDebug "Pushing image to $image_registry/$image_name:$image_tag ..."
    docker tag wl4g/$image_name:$image_tag $image_registry/$image_name:$image_tag
    docker push $image_registry/$image_name:$image_tag
    
}

function deploy::docker_compose() {
    local compose_args="$1"
    local prune_args="$2"
    [ -z "$compose_args" ] && logErr "docker compose args is missing." && exit 1 || echo -n

    local compose_cmd=$(which docker-compose)
    [ -z "$compose_cmd" ] && logErr "The docker-compose not installed yet" && exit 1 || echo -n

    ## Make the Compose environment.
    local node_ip=$(tools::get_host_ip)
    local env_file="/tmp/$(cat /dev/urandom | head -1 | sha1sum | cut -f1 -d-)/.env"
    mkdir -p $(dirname $env_file)
    logDebug "Make the docker compose environment file: $env_file ..."
    echo "BASE_DIR=${BASE_DIR}" > $env_file
    echo "NODE_IP=${node_ip}" >> $env_file

    ## Download the Compose file.
    local compose_file="${BASE_DIR}/tooling/deploy/compose/docker-compose.yml"
    if [ ! -f "$compose_file" ]; then
        local remote_compose_file="https://raw.githubusercontent.com/wl4g/botwaf/main/tooling/deploy/compose/docker-compose.yml"
        log "Downloading compose file from $remote_compose_file ..."
        curl -k -o $compose_file $remote_compose_file
    fi

    ## Before the initial deploy prune.
    if [[ "$compose_args" == up* && "$prune_args" == "--prune-all-volumes" ]]; then
        log "Pruning a previously deployed all volumes..."
        deploy::docker_compose::prune_all_volumes $compose_cmd $compose_file
    fi

    set +e
    $compose_cmd --env-file $env_file -f ${compose_file} $compose_args
    set -e

    ## After destory prune.
    if [[ "$compose_args" == down* && "$prune_args" == "--prune-all-volumes" ]]; then
        log "Pruning deployed all volumes..."
        deploy::docker_compose::prune_all_volumes $compose_cmd $compose_file
    fi
}

function deploy::docker_compose::prune_all_volumes() {
    local compose_cmd="$1"
    local compose_file="$2"
    [ -z "$compose_cmd" ] && logErr "The docker-compose not installed yet" && exit 1 || echo -n
    [ -z "$compose_file" ] && logErr "The docker-compose yaml is missing" && exit 1 || echo -n

    ## Remove the volumes only if it is the first deploy.
    if [ -z "$($compose_cmd -f $compose_file ls | grep botwaf)" ]; then
        set +e
        #docker volume rm botwaf_zookeeper_data 2>/dev/null
        #docker volume rm botwaf_kafka_data 2>/dev/null
        #docker volume rm botwaf_minio_data 2>/dev/null
        #docker volume rm botwaf_mongodb_data 2>/dev/null
        docker volume rm botwaf_redis_data_0 2>/dev/null
        docker volume rm botwaf_redis_data_1 2>/dev/null
        docker volume rm botwaf_redis_data_2 2>/dev/null
        docker volume rm botwaf_redis_data_3 2>/dev/null
        docker volume rm botwaf_redis_data_4 2>/dev/null
        docker volume rm botwaf_redis_data_5 2>/dev/null
        set -e
    else
        logErr "Unable to remove data volumes, please shutdown all containers before, Or remove the arg '--prune-all-volumes'"
        exit 1
    fi
}

# --- Main. ---
case $1 in
  version)
    tools::get_project_version
    ;;
  gpg-verify)
    tools::gpg_install_verify
    ;;
  build-on-host)
    build::binary_on_host "${@:2}"
    ;;
  build-image)
    case $2 in
        -b|--backend)
            build::docker_image "botwaf" "Dockerfile"
            ;;
        -d|--initdb)
            build::docker_image "botwaf-initdb" "Dockerfile.initdb"
            ;;
        -A|--all)
            build::docker_image "botwaf" "Dockerfile"
            build::docker_image "botwaf-initdb" "Dockerfile.initdb"
            ;;
        *)
            usages; exit 1
    esac
    ;;
  push-image)
    case $2 in
        -b|--backend)
            build::push_image "botwaf"
            ;;
        -d|--initdb)
            build::push_image "botwaf-initdb"
            ;;
        -A|--all)
            build::push_image "botwaf"
            build::push_image "botwaf-initdb"
            ;;
        *)
            usages; exit 1
    esac
    ;;
  build-push)
    build::docker_image "botwaf" "Dockerfile"
    build::docker_image "botwaf-initdb" "Dockerfile"
    #build::docker_image "botwaf-ui" "Dockerfile.ui"
    build:push_image "botwaf"
    build:push_image "botwaf-initdb"
    ;;
  prune-image)
    tools::prune_none_image
    ;;
  deploy-docker)
    case $2 in
        -S|--status)
            docker ps --format "table {{.ID}} {{.Names}}\t{{.Image}}\t{{.Status}}\t{{.Ports}}" | grep botwaf_
            ;;
        -U|--up)
            deploy::docker_compose "up -d" "$3"
            ;;
        -D|--down)
            deploy::docker_compose "down" "$3"
            ;;
        *)
            usages; exit 1
    esac
    ;;
  *)
    usages; exit 1
    ;;
esac
