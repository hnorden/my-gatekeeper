# My gatekeeper

A keycloak gatekeeper test setup with docker-compose.

# Run instructions

## Start containers

Start and setup keycloak:

```
docker-compose up -d keycloak-setup
docker-compose logs --follow keycloak keycloak-setup
```

Start gatekeeper instances running in different modes:

```
docker-compose up -d ingress-gatekeeper gatekeeper-auth-proxy
docker-compose logs --follow ingress-gatekeeper gatekeeper-auth-proxy
```

Build rest-service with rust:

```
docker-compose up -d rust-builder
docker-compose logs --follow rust-builder
```

Run the rust-service (built with previous step):

```
docker-compose up -d rust-service
docker-compose logs --follow rust-service
```

## Test the setup

Simulate keycloak login and call REST-Endpoint:

```
bash auth-flow_osx.sh
```

You should get this response:

```
{
  "next": {
    "remote": {
      "status": "OK",
      "username": "auth-proxy-user"
    },
    "url": "http://ingress-gatekeeper:3000/health"
  },
  "status": "OK",
  "username": "testuser"
}
```

You call the ingress-gatekeeper with your token and the header `X-Auth-Username: myUsername`. Gatekeeper extracts details from your token und sets/overrides predefined headers. The request is forwarded to the rust-service which in turn calls itself again (`?next=http://...&with_proxy=http://...`). This time in proxy mode, though.

