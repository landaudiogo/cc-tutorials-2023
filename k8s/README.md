# Overview

> Kubernetes is a system that automates deploying, scaling and management of
> containerized applications. [link](https://kubernetes.io/)

A kubernetes cluster is composed of a set of nodes where containers are
executed and a control-plane which makes global decisions regarding the cluster
such as scheduling.

To learn more about the k8s architecture, please refer to
[this](https://kubernetes.io/docs/concepts/overview/components/) link.

Our goal will be to learn and understand the use of the following kubernetes
resources and concepts: 

- pods
- jobs
- cronjobs
- deployment
- service
- scaling deployments
- presistent volumes and persistent volume claims
- ingress

We will distribute the tutorial on k8s between 2 days, the 20th and the 22nd of
September.

# Installing Minikube & kubectl

To install minikube run the following commands:
```bash
curl -LO https://storage.googleapis.com/minikube/releases/latest/minikube-linux-amd64
sudo install minikube-linux-amd64 /usr/local/bin/minikube
```

```bash
rm minikube-linux-amd64
```

```bash
minikube start
```

> The commands to install and start minikube were taken from: 
>
> https://minikube.sigs.k8s.io/docs/start/

We will now install kubectl with the next commands: 
```bash
sudo apt-get update
# apt-transport-https may be a dummy package; if so, you can skip that package
sudo apt-get install -y apt-transport-https ca-certificates curl
```

```bash
curl -fsSL https://pkgs.k8s.io/core:/stable:/v1.28/deb/Release.key | sudo gpg --dearmor -o /etc/apt/keyrings/kubernetes-apt-keyring.gpg
```

```bash
# This overwrites any existing configuration in /etc/apt/sources.list.d/kubernetes.list
echo 'deb [signed-by=/etc/apt/keyrings/kubernetes-apt-keyring.gpg] https://pkgs.k8s.io/core:/stable:/v1.28/deb/ /' | sudo tee /etc/apt/sources.list.d/kubernetes.list
```

```bash 
sudo apt-get update
sudo apt-get install -y kubectl
```

Now to check whether we have successfully installed kubectl we should see
kubectl's version when we run the following command: 
```bash
kubectl version
```

> These commands were extracted from:
>
> https://kubernetes.io/docs/tasks/tools/install-kubectl-linux/

# Pods

A pod is the smallest unit you can deploy on kubernetes (k8s). Similar to
docker this unit manages containers, but differs in the fact that a pod can
host more than one container.

Provided a kubernetes cluster, nodes are machines that are allocated to the
cluster where resources can be deployed to. Pods are no exception. When we
deploy a pod, we are actually asking k8s to schedule our pod into one of our
nodes in our k8s cluster.

To exemplify deploying pods, we are going to use an application we are already
familiar with. What we will be deploying in this section is similar to some of
the APIs we interacted with in our docker tutorial, however, because k8s is
directed toward a multi-node cluster there is some additional complexity.

## Demo

Within the tutorials repository, change directory into:
```bash
cd k8s/pods
```

Let's start by analyzing our experiment-producer pod manifest: 
```yaml
# experiment-producer.yml
apiVersion: v1
kind: Pod
metadata:
  name: experiment-producer
spec:
  containers:
    - name: c-experiment-producer
      image: image/experiment-producer
      imagePullPolicy: Never
      args: ["--topic", "{{TOPIC}}"]
      volumeMounts:
        - name: config-vol
          mountPath: /usr/src/cc-assignment-2023/experiment-producer/auth
  volumes:
    - name: config-vol
      configMap:
        name: kafka-auth
        items:
          - key: kafka.truststore.pkcs12
            path: kafka.truststore.pkcs12
          - key: kafka.keystore.pkcs12
            path: kafka.keystore.pkcs12
          - key: ca.crt
            path: ca.crt
```

This manifest simply describes what we require to deploy our
experiment-producer in k8s.

But before doing so, we will have to create our configMap for our pod to have
access to our personal kafka credentials. Given you are in the `k8s` tutorial
directory, run the following command:
```bash
kubectl create configmap kafka-auth --from-file=../../kafka/auth
```

The command creates a configMap wherein each key is the name of the file in the
`../kafka/auth` directory, and the content is the file itself. As a
confirmation, run the following:
```bash
kubectl describe configmap kakfa-auth
```

We can now use this configmap in our pod manifest above. To create our pod we
will first have to change a templated value in the file `{{TOPIC}}` which is
different for each of us. To do so, first change the `<your-topic>` value in
the command, and start your experiment producer: 
```bash
sed 's/{{TOPIC}}/<your-topic>/g' < pod-template.yml | kubectl apply -f -
```

# Persistent Volumes and Persistent Volume Claims

Similar to how we used volumes to persist data in docker, kubernetes also
provides persistence, with persistent_volumes and persistent_volume_claims. 

To illustrate this functionality, we will start our own postgre database as a
pod, insert some data into it, delete the pod, and lastly restart the database
to determine whether the data was persisted with the use of volumes.

The manifest below contains all the Kubernetes resources we will require to
deploy our database.

- The PersistentVolume creates the physical resources on our node, where the
  data is persisted. I.e. the directory `/data/db` on our node will be where
  the data will be persisted.
- A PersistentVolumeClaim is a request for a type of resources, with access
  type and capacity. This is the resource that will be associated with our Pod.
- Similar to the previous exercise, we also create a ConfigMap which will hold
  our database's configuration variables.
- Laslty, the Pod references our ConfigMap with the `configmapRef` and the
  persistentVolumeClaim with the `claimName` attribute.

```yaml
# persistent_volumes/pv-pvc.yml
kind: PersistentVolume
apiVersion: v1
metadata:
  name: postgres-pv-volume  # Sets PV's name
spec:
  storageClassName: manual
  capacity:
    storage: 1Gi # Sets PV Volume
  accessModes:
    - ReadWriteOnce
  hostPath:
    path: "/data/db"

---
kind: PersistentVolumeClaim
apiVersion: v1
metadata:
  name: postgres-pv-claim  # Sets name of PVC
spec:
  storageClassName: manual
  accessModes:
    - ReadWriteOnce  # Sets read and write access
  resources:
    requests:
      storage: 1Gi  # Sets volume sizeapiVersion: v1

---
kind: ConfigMap
apiVersion: v1
metadata:
  name: postgres-config
  labels:
    app: postgres
data:
  POSTGRES_DB: postgresdb
  POSTGRES_USER: admin
  POSTGRES_PASSWORD: psltest

---
kind: Pod
apiVersion: v1
metadata:
  name: postgre-database
spec:
  containers:
    - name: postgres
      image: postgres:10.1 # Sets Image
      imagePullPolicy: "IfNotPresent"
      ports:
        - containerPort: 5432  # Exposes container port
      envFrom:
        - configMapRef:
            name: postgres-config
      volumeMounts:
        - mountPath: /var/lib/postgresql/data
          name: postgredb
  volumes:
    - name: postgredb
      persistentVolumeClaim:
        claimName: postgres-pv-claim
  restartPolicy: Always
```

To create our kubernetes resources, we can run: 
```bash
kubectl apply -f persistent_volumes/pv-pvc.yml
watch kubectl get pods 
```

We will now connect to our pod with: 
```bash
kubectl exec -it postgre-database -- /bin/bash
```

And now to connect to the database server:
```bash
psql -U admin -p psltest -d postgresdb -h localhost -p 5432
```

Run the following lines to create data within our database: 
```
CREATE SCHEMA experiment;
CREATE TABLE experiment.researcher (id TEXT PRIMARY KEY, email TEXT);
INSERT INTO experiment.researcher (id, email) VALUES ('1234', 'd.landau@uu.nl');
SELECT * FROM experiment.researcher;
```

We now remove our pod with:
```bash
kubectl delete pod postgre-database
```

If persistent data is not configured, when we delete our database, when
starting our pod again, our database would no longer have the data we
generated.

Lets confirm whether this is the case. Recreate the postgre-database pod:
```bash
kubectl apply -f persistent_volumes/pv-pvc.yml
watch kubectl get pods
```

And connect to the pod:
```bash
kubectl exec -it postgre-database -- /bin/bash
```

Connect to the database server:
```bash
psql -U admin -p psltest -d postgresdb -h localhost -p 5432
```

And lastly, query the server for the data contained in the `researcher` table:
```bash
SELECT * FROM experiment.researcher;
```

If everything went according to plan, the query should show the line for the
researcher we created before.

# Jobs & CronJobs

> https://kubernetes.io/docs/concepts/workloads/controllers/job/
> 
> *A Job creates one or more Pods and will continue to retry execution of the
> Pods until a specified number of them successfully terminate.*

A job is a resource that runs pods (possibly in parallel) until a specified
number of pods have terminated successfully.

Our job manifest is requesting 4 successful completions of our
`experiment-producer` while only allowing 2 producers to run in parallel:
```yaml
# jobs/job-template
apiVersion: batch/v1
kind: Job
metadata:
  name: experiment-producer-job
spec:
  completions: 4
  parallelism: 2
  backoffLimit: 4
  template:
    # the same spec as we used for our pod
```

To start our job, run:
```bash
sed 's/{{TOPIC}}/<your-topic>/g' < jobs/job-template.yml | kubectl apply -f -
watch kubectl get pods
```

We can also check thet state of our job with:
```bash
kubectl get jobs
```

A CronJob, is a wrapper around Kubernetes Jobs, which allows starting a job
based on a cron expression. 

```yaml
# jobs/cronjob-template.yml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: experiment-producer-cron
spec:
  schedule: "* * * * *"
  jobTemplate:
    # the job configurations
```

As shown in the previous cronjob manifest, the `schedule` attribute indicates
we want a job with the `jobTemplate` value to be created every minute. You can
create more elaborate rules, such as every 5 minutes, every 10 minutes, every
hour, every 10th minute, and so on...

Visit this [link](https://crontab.guru/) to check other cron expressions.

Create the CronJob:
```bash
sed 's/{{TOPIC}}/<your-topic>/g' < jobs/cronjob-template.yml | kubectl apply -f -
watch kubectl get pods
```

