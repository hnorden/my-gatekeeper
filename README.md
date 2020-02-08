# My gatekeeper

A keycloak gatekeeper test setup with docker-compose.

# Run instructions

## Start containers

Start and setup keycloak:

```
docker-compose up -d keycloak-setup
docker-compose logs --follow keycloak keycloak-setup
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

Start dispatcher to serve frontend and backend with same domain (this step is implied by ingress-gatekeeper dependencies):

```
docker-compose up -d ingress-nginx-dispatcher
docker-compose logs --follow ingress-nginx-dispatcher frontend-nginx rust-service
```

Start gatekeeper instances running in different modes:

```
docker-compose up -d --scale ingress-gatekeeper=2 ingress-gatekeeper gatekeeper-auth-proxy
docker-compose logs --follow ingress-gatekeeper gatekeeper-auth-proxy ingress-nginx-dispatcher frontend-nginx rust-service
```

## Test the setup

### Browser test

Please add these hosts:

```
sudo bash -c 'echo -e "127.0.0.1 keycloak" >> /etc/hosts'
sudo bash -c 'echo -e "127.0.0.1 ingress-gatekeeper" >> /etc/hosts'
```

Open your browser and navigate to [http://ingress-gatekeeper:3000/](http://ingress-gatekeeper:3000/).

If you are not willing or able to alter your settings, you can fallback to the next section.

### Script test

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

> https://www.keycloak.org/docs/latest/securing_apps/index.html#upstream-headers

# What is really happening here?

See for yourself:

```
curl --head "http://localhost:8000/health"
HTTP/1.1 401 Unauthorized
```

```
curl "http://rust-service:8000/health" --proxy "http://localhost:4000" | jq
{
  "status": "OK",
  "username": ""
}
```

```
curl "http://ingress-gatekeeper:3000/health" --proxy "http://localhost:4000" | jq
{
  "status": "OK",
  "username": "auth-proxy-user"
}
```

```
curl "http://ingress-gatekeeper:3000/health?next=http://ingress-gatekeeper:3000/health" --proxy "http://localhost:4000" | jq
curl "http://ingress-gatekeeper:3000/health?next=http://ingress-gatekeeper:3000/health&with_proxy=http://gatekeeper-auth-proxy:4000" --proxy "http://localhost:4000" | jq
{
  "next": {
    "remote": {
      "status": "OK",
      "username": "auth-proxy-user"
    },
    "url": "http://ingress-gatekeeper:3000/health"
  },
  "status": "OK",
  "username": "auth-proxy-user"
}
```

This script first processes the keycloak login and token exchange (OpenID Connect Authorization Code Flow with PKCE) for a regular user with username and password. After that it calls the outfacing reverse proxy ingress-gatekeeper.

```
bash auth-flow_osx.sh
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

The final step the script does is just this:

```
curl "http://localhost:3000/health?next=http://ingress-gatekeeper:3000/health&with_proxy=http://gatekeeper-auth-proxy:4000" -H "X-Auth-Username: myUsername" -b "kc-access=eyJhb...;" | jq
```

To reproduce simply copy the token (`> Cookie: kc-access=...`).

Removing the proxy you receive this:

```
curl "http://localhost:3000/health?next=http://ingress-gatekeeper:3000/health" -H "X-Auth-Username: myUsername" -b "kc-access=eyJhb...;" | jq
{
  "next": {
    "remote": {
      "status": "OK",
      "username": "testuser"
    },
    "url": "http://ingress-gatekeeper:3000/health"
  },
  "status": "OK",
  "username": "testuser"
}
```

# Cluster Compatibility

To show cluster compatibility switch from port 3000 to 3001:

```
curl "http://localhost:3001/health?next=http://ingress-gatekeeper:3000/health" -H "X-Auth-Username: myUsername" -b "kc-access=eyJhb...;" | jq
{
  "next": {
    "remote": {
      "status": "OK",
      "username": "testuser"
    },
    "url": "http://ingress-gatekeeper:3000/health"
  },
  "status": "OK",
  "username": "testuser"
}
