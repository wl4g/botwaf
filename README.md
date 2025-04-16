# Botwaf

> Botwaf - A Mini Open Source AI-driven Bot WAF written in Rust.

## Introduction

BotWAF is a lightweight, Rust-based AI-driven WAF that uses ModSecurity to intercept malicious requests and dynamically generate rules via LangChain. It consists of components - server, forwarder, updater, verifier - for robust intelligent security.

## Features

- Support for HTTP/1.1 and HTTP/2 protocols high-performance forwarding;
- Dynamic rule generation using LangChain-based AI;
- Real-time threat detection and response;
- Robust security through __ModSecurity__ Engine;
- Lightweight and efficient design written in Rust async Axum;

## Development

- [Prerequisites for locally Development](./docs/devel/1.prerequisites-for-dev.md)
- [Build and Test for locally Development](./docs/devel/2.build-and-test-for-dev.md)

## Deployment

- [Deploy on Docker](./docs/deploy/build-and-deploy-on-docker.md)
- [Deploy on Kubernetes](./docs/deploy/deploy-on-kubernetes.md)
