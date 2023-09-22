# Overview

# Installation & Setup

Install arkade: 
```bash
curl -sSL https://get.arkade.dev | sudo -E sh
```

Install `faas-cli`:
```bash
arkade get faas-cli
```

Add the directory where the `faas-cli` binary was installed to the list of
searchable paths for binary files:
```bash
echo "export PATH=$PATH:$HOME/.arkade/bin/" >> ~/.bashrc
source ~/.bashrc
```

Install openfaas: 
```bash
arkade install openfaas
```

Run the following commands to deploy our first function:
```bash
# Forward the gateway to your machine
kubectl rollout status -n openfaas deploy/gateway
kubectl port-forward -n openfaas svc/gateway 8080:8080 &
```

```bash
PASSWORD=$(kubectl get secret -n openfaas basic-auth -o jsonpath="{.data.basic-auth-password}" | base64 --decode; echo)
echo -n $PASSWORD | faas-cli login --username admin --password-stdin

faas-cli store deploy figlet
faas-cli list
```

To make sure our figlet function has been deployed correctly, run the following
command:
```bash
curl -X POST http://127.0.0.1:8080/function/figlet -d "test"
```

Openfaas requires pushing our images to a container registry to deploy our
serverless functions. For now, we will setup  a local registry setup in our
minikube cluster by running the following commands: 
```bash
minikube addons enable registry
```
followed by:
```bash
docker run --rm -it --network=host alpine ash -c "apk add socat && socat TCP-LISTEN:5000,reuseaddr,fork TCP:$(minikube ip):5000"
```

If you'd like to use your personal container registry in dockerhub for example,
you can skip this step. You do however have to run `docker login` to connect
your local docker to your docker hub registry.


