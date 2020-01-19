#!/bin/bash

# Setup on macOS
# brew install gnu-sed
# ln -s /usr/local/bin/gsed /usr/local/bin/sed

# Requires
# sudo bash -c 'echo -e "127.0.0.1 keycloak" >> /etc/hosts'

init=$(curl --verbose "http://localhost:3000/health" -L 2>&1)

requestUri=$(echo "$init" | sed -n -r 's/< Set-Cookie: (request_uri=.*); Path.*/\1/gp')
oauthTokenRequestState=$(echo "$init" | sed -n -r 's/< Set-Cookie: (OAuth_Token_Request_State=.*); Path.*/\1/gp')

authSessionId=$(echo "$init" | sed -n -r 's/< Set-Cookie: (AUTH_SESSION_ID=.*); Version.*/\1/gp')
kcRestart=$(echo "$init" | sed -n -r 's/< Set-Cookie: (KC_RESTART=.*); Version.*/\1/gp')

authenticateUrl=$(echo "$init" | sed -n -r 's/.*action="(http:\/\/keycloak:8080.*)" method.*/\1/gp')
authenticateUrlSessionCode=$(echo "$authenticateUrl" | sed -n -r 's/.*(session_code=.*)&amp;execution.*/\1/gp')
authenticateUrlExecution=$(echo "$authenticateUrl" | sed -n -r 's/.*(execution=.*)&amp;client_id.*/\1/gp')
authenticateUrlTabId=$(echo "$authenticateUrl" | sed -n -r 's/^.*(tab_id=.*)$/\1/gp')

login=$(curl --verbose "http://localhost:8080/auth/realms/demorealm/login-actions/authenticate?$authenticateUrlSessionCode&$authenticateUrlExecution&client_id=ingress-gatekeeper-client&$authenticateUrlTabId" -X POST --data "username=testuser&password=testuser&login=Log+In" -H "Content-Type: application/x-www-form-urlencoded" -b "$authSessionId; $kcRestart;" 2>&1)

keycloakIdentity=$(echo "$login" | sed -n -r 's/< Set-Cookie: (KEYCLOAK_IDENTITY=.*); Version.*/\1/gp')
keycloakSession=$(echo "$login" | sed -n -r 's/< Set-Cookie: (KEYCLOAK_SESSION=.*); Version.*/\1/gp')
gatekeeperLocation=$(echo "$login" | sed -n -r 's/< Location: (.*)/\1/gp' | sed -n -r 's/http:\/\/ingress-gatekeeper:3000/http:\/\/localhost:3000/gp' | tr -d '\r') # remove '%0D'

callback=$(curl --verbose "$gatekeeperLocation" -b "$requestUri; $oauthTokenRequestState;" 2>&1)
kcAccess=$(echo "$callback" | sed -n -r 's/< Set-Cookie: (kc-access=.*); Path.*/\1/gp')
kcState=$(echo "$callback" | sed -n -r 's/< Set-Cookie: (kc-state=.*); Path.*/\1/gp')

myResource=$(curl --verbose "http://localhost:3000/health?next=http://ingress-gatekeeper:3000/health" -H "X-Auth-Username: myUsername" -b "$kcAccess;" 2>&1)

# printf "\ninit >>>>>\n"
# echo "$init"
# printf "<<<<< init\n"

# echo requestUri "$requestUri"
# echo oauthTokenRequestState "$oauthTokenRequestState"
# echo authSessionId "$authSessionId"
# echo kcRestart "$kcRestart"

# echo authenticateUrl "$authenticateUrl"
# echo authenticateUrlSessionCode "$authenticateUrlSessionCode"

# printf "\nlogin >>>>>\n"
# echo "$login"
# printf "<<<<< login\n"

# printf "\ncallback >>>>>\n"
# echo "$callback"
# printf "<<<<< callback\n"

# echo kcAccess "$kcAccess"
# echo kcState "$kcState"

echo "$myResource"
