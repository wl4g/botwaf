# Build and Deploy on Docker

## Building the image

- Build the execuable and initdb All images.

```bash
./run.sh build-image -A
```

- Push the All images to the registry.

```bash
./run.sh push-image -A
```

## Run with Docker Componse

- Start up the All services.

```bash
./run.sh deploy-docker -U
```
