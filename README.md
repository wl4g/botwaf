# Botwaf

> Botwaf - A Mini Open Source AI Bot WAF written in Rust.

## Prerequisites

### Rust 1.85+

```bash
rustup toolchain install stable
```

### [Install rustfmt VsCode plugin](https://marketplace.visualstudio.com/items?itemName=statiolake.vscode-rustfmt)

### Run models with ollama

- Run embeddding model

```bash
export OLLAMA_HOST='0.0.0.0'
nohup ollama start >/dev/null &
```

- Verify embeddding model. [See more ollama API docs](https://github.com/ollama/ollama/blob/main/docs/api.md#generate-a-completion)

```bash
curl -X POST http://localhost:11434/api/embed \
-H "Content-Type: application/json" \
-d '{"model": "bge-m3:latest", "input": "Hello, world!"}'
```

## Quick Start

### Building & Run with native

```bash
git clone git@github.com:wl4g-private/book-playground.git
cd rust-playground/playground-waf-modsecurity-ai

# Show targets supported.
rustup target list | grep -iE 'apple-darwin|x86_64-unknown-linux-gnu'
#aarch64-apple-darwin (installed)
#x86_64-apple-darwin
#x86_64-unknown-linux-gnu
#x86_64-unknown-linux-gnux32

# Build for X86 MacOS
cargo build --target x86_64-apple-darwin

# Build for ARM MacOS
cargo build --target aarch64-apple-darwin

# Build for Generic Linux (Unknown means vendor-less bound)
cargo build --target x86_64-unknown-linux-gnu

# Run Botwaf
./target/debug/botwaf
```

### Building & Run with image

```bash
docker build --platform=amd64 -t registry.cn-shenzhen.aliyuncs.com/wl4g/botwaf:latest .
```

### Run with Docker

```bash
docker run -d \
--name botwaf \
--restart unless-stopped \
--security-opt seccomp=unconfined \
-p 9999:9999 \
-e RUST_BACKTRACE=full \
registry.cn-shenzhen.aliyuncs.com/wl4g/botwaf:latest
```

### Verify Botwaf via directly

```bash
curl -I -H 'User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36' \
http://localhost:9999/admin/get
#HTTP/1.1 403 Forbidden
#content-length: 23
#date: Sun, 09 Mar 2025 10:07:27 GM
#x-waf-blocked: 1001    <== Botwaf Intercepted

# Corresponding to the Botwaf ModSecurity rule: ↓↓↓
#SecRuleEngine On
#SecRule REQUEST_URI "@rx admin" "id:1000,phase:1,deny,status:403,msg:'Forbidden Admin Path Detected'"
```

### Generate Nginx configuration

```bash
export LOCAL_IP="$(ip a | grep -E '(em|eno|enp|ens|eth|wlp|en)+[0-9]' -A2 | grep inet | awk -F ' ' '{print $2}' | cut -f 1 -d / | tail -n 1)"
cat <<EOF > /tmp/myapp.conf
server {
    listen       8888;
    #listen       443 ssl;

    server_name  blogs.myapp.com;
    #ssl_certificate cert.d/myapp.fullchain.letencrypt.pem;
    #ssl_certificate_key cert.d/myapp.letencrypt.pem;
    #ssl_session_timeout 5m;
    #ssl_ciphers ECDHE-RSA-AES128-GCM-SHA256:ECDHE:ECDH:AES:HIGH:!NULL:!aNULL:!MD5:!ADH:!RC4;
    #ssl_protocols TLSv1 TLSv1.1 TLSv1.2;
    #ssl_prefer_server_ciphers on;
    ## Notice: The port should be included in Host, otherwise it may cause the backend
    ## service to not resolve properly, resulting in infinite redirects, such as: wordpress

    proxy_set_header Host \$host:\$server_port;
    proxy_set_header X-Real-IP \$remote_addr;
    proxy_set_header X-Forwarded-Proto \$scheme;
    proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
    proxy_set_header X-Forwarded-Host \$host;
    proxy_set_header X-Forwarded-Server \$host;
    proxy_set_header X-Forwarded-Port \$server_port;
    proxy_set_header Referer \$http_referer;
    #include /etc/nginx/default.d/*.conf;

    location / {
        proxy_set_header X-Upstream-Destination "http://${LOCAL_IP}:8080";
        proxy_pass http://${LOCAL_IP}:9999; # Proxy to Botwaf
        error_page 433 = @handle_waf_block;
        proxy_intercept_errors off;
    }

    location @handle_waf_block {
        # If BOTWAF returns a rejection status, it will usually carry a header such as 'X-WAF-Block: 1001'.
        # If we need to respond to the client (without using the Nginx special status code 444), it is recommended to remove it for security reasons.
        proxy_hide_header X-Waf-Block;
        return 444;
    }
}
EOF
```

### Run nginx

```bash
docker run -d \
--name myapp_nginx \
--restart unless-stopped \
--security-opt seccomp=unconfined \
-p 8888:8888 \
-v /tmp/myapp.conf:/etc/nginx/conf.d/myapp.conf \
registry.cn-shenzhen.aliyuncs.com/wl4g/nginx:1.27.3-alpine3.20
```

### Verify Botwaf via Nginx

```bash
curl -I -H 'User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36' \
http://localhost:8888/admin/get
#HTTP/1.1 403 Forbidden
#Server: nginx/1.27.3
#Date: Sun, 09 Mar 2025 10:01:52 GMT
#Content-Length: 23
#Connection: keep-alive
#x-waf-blocked: 1001    <== Botwaf Intercepted
```

## FAQ

### How solve to build with `cargo build --target x86_64-unknown-linux-gnu` in macOS M3 aarch64 failure

- Operation

```bash
# Prerequisites.
rustup target add x86_64-unknown-linux-gnu
# Build to linux x86_64
cargo build --target x86_64-unknown-linux-gnu
```

- Error

```log
...
pkg-config has not been configured to support cross-compilation.
 Install a sysroot for the target platform and configure it via
 PKG_CONFIG_SYSROOT_DIR and PKG_CONFIG_PATH, or install a
 cross-compiling wrapper for pkg-config and set it via
 PKG_CONFIG environment variable.
 cargo:warning=Could not find directory of OpenSSL installation, and this -sys crate cannot proceed without this knowledge. If OpenSSL is installed and this crate had trouble findin
g it, you can set the OPENSSL_DIR environment variable for the compilation process. See stderr section below for further information.
 --- stderr
 Could not find directory of OpenSSL installation, and this -sys crate cannot
 proceed without this knowledge. If OpenSSL is installed and this crate had
 trouble finding it, you can set the OPENSSL_DIR environment variable for the
 compilation process.
 Make sure you also have the development packages of openssl installed.
 For example, libssl-dev on Ubuntu or openssl-devel on Fedora.
 If you're in a situation where you think the directory should be found
 automatically, please open a bug at https://github.com/sfackler/rust-openssl
 and include information about your system as well as this message.
 $HOST = aarch64-apple-darwin
 $TARGET = x86_64-unknown-linux-gnu
 openssl-sys = 0.9.106
warning: build failed, waiting for other jobs to finish...
```

- Resolve

```bash
brew install FiloSottile/musl-cross/musl-cross
brew install pkg-config openssl
```
