function checkHealth() {
  const baseUrl =
    "http://ingress-gatekeeper:3000/health?next=http://ingress-gatekeeper:3000/health&with_proxy=http://gatekeeper-auth-proxy:4000";

  return fetch(baseUrl, {
    method: "GET"
  })
    .then(response => {
      if (response.ok) {
        return response.json();
      } else {
        throw new Error(response.statusText);
      }
    })
    .then(response => JSON.stringify(response, null, "\t"))
    .catch(error => error);
}

checkHealth().then(response => {
  const el = document.getElementById("response");
  el.innerHTML = response;
});
