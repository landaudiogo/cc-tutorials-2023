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
echo '
export PATH="$PATH:$HOME/.arkade/bin/"
export OPENFAAS_PREFIX="localhost:5000"
' >> ~/.bashrc
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

# Serverless Hello World

This part of the tutorial is for the most part, based on this
[link](https://docs.openfaas.com/tutorials/first-python-function/).

First, change directory into your tutorials repository followed by:
```bash
cd openfaas
```

To create a new function, run: 
```bash
faas-cli new --lang python hello-world
```

This should add 2 directories, a `hello-world` and `template` directory.
Additionally, you should also see a `hello-world.yml` file.

Now open the `hello-world/handler.py` file with your favourite text editor.
```bash
vi hello-world/handler.py
```

The `req` variable passed in the handler corresponds to what is passed in the
body of th request. Let's modify the return from this: 
```python
    # ... 
    return req 
```
To this: 
```python
    # ... 
    return "Hello from ECC " + req 
```

We can now, build, push and deploy our functions to our k8s cluster:
```bash
faas-cli build -f hello-world.yml
faas-cli push -f hello-world.yml
faas-cli deploy -f hello-world.yml
```
or you may run the combination of these commands as a single command:
```bash
faas-cli up -f hello-world.yml
```

You may now make a request to your service:
```bash
curl http://127.0.0.1:8080/function/hello-world
```

# Installing Additional Software Dependencies

> This part of the tutorial is based on
> [this](https://github.com/openfaas/python-flask-template#example-with-postgresql)
> example provided in an openfaas template github repository.

Upon receiving a request, a function might have additional software
dependencies other than the ones install in its environment by default. For
this example, we will create a service that: 
1. Creates a user in its database when its `POST` method is called
1. Lists the users in its database when its `GET` method is called

First, we have to install the `python3-http` template from the openfaas
template store: 
```bash
faas-cli template store pull python3-http
```

You can also see the available templates with:
```bash
faas-cli template store list
```

The code for this function is available in the `python-db` directory, and the
`python-db.yml` file. By adding our software dependencies in our function's
`requirements.txt` file:
```
# pythond-db/requirements.txt
psycopg2==2.9.3
```
and to our function's stack file, through the
`functions.python-db.ADDITIONAL_PACKAGE` attribute:
```yml
version: 1.0
provider:
  name: openfaas
  gateway: http://127.0.0.1:8080
functions:
  python-db:
    lang: python3-http
    handler: ./python-db
    build_args:
      ADDITIONAL_PACKAGE: "libpq-dev gcc python3-dev build-base"
    image: localhost:5000/python-db:latest
```
we may now use this package at runtime.

Our function will have the following code deployed:
```python
# python-db/handler.py
import psycopg2

def handle(event, context):
    try: 
        conn = psycopg2.connect("dbname='postgresdb' user='admin' port=5432 host='postgre-svc.default.svc.cluster.local' password='psltest'")
        cur = conn.cursor()
    except Exception as e:
        print("DB error {}".format(e))
        return {
            "statusCode": 500,
            "body": e
        }

    if event.method == "GET": 
        cur.execute(f"""SELECT * FROM experiment.researcher;""")
        rows = cur.fetchall()
        return {
            "statusCode": 200,
            "body": rows
        }
    elif event.method == "POST": 
        researcher = event.query["researcher"]
        query = f"INSERT INTO experiment.researcher (email) VALUES (%s);"
        cur.execute(query, (researcher,))
        conn.commit()
        return {
            "statusCode": 200,
            "body": f"Created researcher {researcher}" }
```
Our function starts by connecting to our database. This is then followed by
verifying whether the request's method is of type `POST` or `GET`. For the
former, it creates a user in the database, based on a query parameter passed in
our request. Otherwise, it will simply query the database to list the existing
users.

Before deploying our function, let's create our database (wait for it to get
into a `Running` status):
```bash
kubectl apply -f python-db/postgre-db.yml
```

We may now build and deploy our function:
```bash
faas-cli up -f python-db.yml
```

We may now query the function with the following command: 
```bash
curl http://127.0.0.1:8080/function/python-db
```
You should see an empty list of users.

We can now insert a user with: 
```bash
curl -X POST -G -d 'researcher=n.saurabh@uu.nl' http://127.0.0.1:8080/function/python-db
```

If you now query the function for the list of users, you should now see the new
user you created with the previous request:
```bash
curl http://127.0.0.1:8080/function/python-db
```

You can now remove the resources just created: 
```bash
faas-cli remove python-db
kubectl delete -f python-db/postgre-db.yml
```
