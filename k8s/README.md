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
cd k8s
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
      args: ["--topic", "{{TOPIC}}", "{{GROUP_ID}}"]
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
kubectl create configmap kafka-auth --from-file=../kafka/auth
```

The command creates a configMap wherein each key is the name of the file in the
`../kafka/auth` directory, and the content is the file itself. As a
confirmation, run the following:
```bash
kubectl describe configmap kafka-auth
```

We can now use this configmap in our pod manifest above. To create our pod we
will first have to change the `{{TOPIC}}` template value which is unique to
each of us. To do so, change the `<your-topic>` in the command, and start your
experiment producer: 
```bash
sed 's/{{TOPIC}}/<your-topic>/g' < pods/pod-template.yml | kubectl apply -f -
```

Delete your pod:
```bash
kubectl delete -f pods/pod-template.yml
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

Disconnect from the database: 
```bash
\q
```
and from the container: 
```bash
exit
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

Disconnect from the database: 
```bash
\q
```
and from the container: 
```bash
exit
```

Delete the resources we just created: 
```bash
kubectl delete -f persistent_volumes/pv-pvc.yml
```

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

Delete the job: 
```bash
kubectl delete -f jobs/job-template.yml
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

Delete the cronjob: 
```bash
kubectl delete -f jobs/cronjob-template.yml
```

# Deployment

A deployment is a k8s resource that manages a stateless set of pods. With
deployments. By defining our deployment's state, the deployment controller will
always try to uphold this contract. When dealing with deployments, we can
deploy our service by increasing the number of replicas (horizontal scaling).
This can either be done manually, or resorting to the horizontal pod
autoscaler (HPA). We will look into HPA further into this tutorial.

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: experiment-consumer-deployment
  labels:
    app: experiment-consumer
spec:
  replicas: 2
  selector:
    matchLabels:
      app: experiment-consumer
  template:
    metadata:
      labels:
        app: experiment-consumer
    spec:
      containers:
        - name: experiment-consumer
          image: dclandau/assignment-consumer:1.0.1
          args: ["consumer.py", "{{TOPIC}}"]
          volumeMounts:
            - name: config-vol
              mountPath: /usr/src/app/auth
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

This deployment indicates it will manage pods that have the structure defined
in the `.spec.template.spec` attribute. Each pod will host a single container
based on the image `dclandau/assignment-consumer:1.0.0` pulled from my docker
hub image repository. This image creates a consumer instance that will read
data from the topic we pass into it as an argument.

Deployments require pods to always restart in case they terminate. As such,
this type of k8s resource is more appropriate for non-terminating services.
Also, this manifest also states that we want our deployment to hold 2 instances
of our pod.

To create our deployment, we run (don't forget to change the value of `<your-topic>`): 
```bash
sed \
    -e 's/{{TOPIC}}/<your-topic>/g' \
    -e "s/{{GROUP_ID}}/"$(cat /proc/sys/kernel/random/uuid | tr -d "-")"/g" < deployment/deployment-template.yml \
    | kubectl apply -f -
```

Now inspect the number of pods that our deployment has created: 
```bash
kubectl get pods
```

To test our consumers, we can run our job once again (don't forget to change
the `<your-topic>` value):
```bash
sed 's/{{TOPIC}}/<your-topic>/g' < jobs/job-template.yml | kubectl apply -f -
watch kubectl get pods
```

Manually up- or down-scaling our deployment is as simple as running:
```bash
kubectl scale --replicas 3 deployment/experiment-consumer-deployment
```
or changing the replicas attribute in our deployment manifest and re-applying
the file.

Delete the resources we just created: 
```bash
kubectl delete -f deployment/deployment-template.yml
kubectl delete -f jobs/job-template.yml
```

# Service

"In Kubernetes, a Service is a method for exposing a network application that
is running as one or more Pods in your cluster."
[link](https://kubernetes.io/docs/concepts/services-networking/service/) 

To illustrate k8s service resource, our goal is to load-balance our requests
between a set of 2 notifications-service instances. Since all pods contained in
a deployment are stateless, our communication requirements can usually be
satisfied by any of the pods in our deployment.

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: notifications-service
  labels:
    app: tutorial-k8s
spec:
  replicas: 2
  selector:
    matchLabels:
      app: notifications-service
  template:
    metadata:
      labels:
        app: notifications-service
    spec:
      containers:
        - name: notifications-service
          image: dclandau/notifications-service:1.0.0
          ports: 
          - containerPort: 3000

---
apiVersion: v1
kind: Service
metadata:
  name: notifications-service
spec:
  type: NodePort
  selector:
    app: notifications-service
  ports:
    - port: 3000
      targetPort: 3000
      nodePort: 30674
```

Our service manifest creates a deployment with 2 replicas of our
notifications-service. We then expose our pods via the NodePort service.

To fully expose our Node to external traffice, we still have to port-forward
one of our host's ports to our service's port. To do so, we run: 

We can start our deployment and service with the following command:
```bash
sed 's/{{EXTERNAL_IP}}/<your-vm-ip>/g' < service/service.yml | kubectl apply -f -
```

Since minikube is an experimental k8s cluster, it does not provide any simple
facility to expose the service to external traffic, without losing the
load-balancing characteristic of services (see
[this](https://stackoverflow.com/a/59941521) stackoverflow thread). With this
command, we set up our reverse proxy:
```bash
docker run \
    --rm -d \
    --name nginx-proxy \
    -v $(pwd)/service/conf.d:/etc/nginx/conf.d \
    -p 3000:3000 \
    --network minikube \
    nginx:alpine 
```

You may now open your browser on your computer and visit the link
`<your-vm-ip>:3000`.

If we now run 5 of our requests: 
```bash
for i in $(seq 1 5); do 
    curl -X 'POST' \
        'http://localhost:3000/api/notify' \
        -H 'accept: text/plain; charset=utf-8' \
        -H 'Content-Type: application/json; charset=utf-8' \
        -d '{
            "notification_type": "OutOfRange",
            "researcher": "d.landau@uu.nl",
            "measurement_id": "1234",
            "experiment_id": "5678",
            "cipher_data": "D5qnEHeIrTYmLwYX.hSZNb3xxQ9MtGhRP7E52yv2seWo4tUxYe28ATJVHUi0J++SFyfq5LQc0sTmiS4ILiM0/YsPHgp5fQKuRuuHLSyLA1WR9YIRS6nYrokZ68u4OLC4j26JW/QpiGmAydGKPIvV2ImD8t1NOUrejbnp/cmbMDUKO1hbXGPfD7oTvvk6JQVBAxSPVB96jDv7C4sGTmuEDZPoIpojcTBFP2xA"
        }'; 
done
```
we can verify that these requests were load-balanced between the different pods
selected by our service.

List the pods you have available:
```bash
kubectl get pods
```

Copy the name of the first pod, and show its logs:
```bash
kubectl logs -f "<pod-1>"
```

Copy the name of the second pod, and show its logs:
```bash
kubectl logs -f "<pod-2>"
```

Delete the resources just created:
```bash
kubectl delete -f service/service.yml
docker stop nginx-proxy
```

# Horizontal Pod Autoscaling

*"In Kubernetes, a HorizontalPodAutoscaler automatically updates a workload
resource (such as a Deployment or StatefulSet), with the aim of automatically
scaling the workload to match demand."*
[link](https://kubernetes.io/docs/tasks/run-application/horizontal-pod-autoscale/)

Horizontal means adding more instances instead of adding more resources (which
would be vertical scaling).

For this part of the tutorial, we will stress a notifications-service
deployment to see how the number of instances in our deployment changes over
time. Our procedure will look something like: 

1. Create our notifications-service deployment & service
1. Stress the notifications-service
1. Check the number of notifications-service instances
1. Stop the stress test
1. Check the number of notifications-service instances

We would expect that while stressing our application, the number of instances
should increase, but never above the limit we set in the HPA manifest. And when
the service is no longer being stressed, the number of instances should
decrease to the minimum.

Let's have a look at our manifest: 
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: notifications-service
  labels:
    app: tutorial-k8s
spec:
  replicas: 1
  selector:
    matchLabels:
      app: notifications-service
  template:
    metadata:
      labels:
        app: notifications-service
    spec:
      containers:
        - name: notifications-service
          image: dclandau/notifications-service:1.0.0
          args: ["--external-ip", "{{EXTERNAL_IP}}"]
          ports: 
          - containerPort: 3000
          resources:
            requests:
              cpu: 50m

---
apiVersion: v1
kind: Service
metadata:
  name: notifications-service
spec:
  type: NodePort
  selector:
    app: notifications-service
  ports:
    - port: 3000
      targetPort: 3000
      nodePort: 30674

---
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: notifications-service
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: notifications-service
  maxReplicas: 5
  minReplicas: 1
  metrics:
  - resource:
      name: cpu
      target:
        averageUtilization: 20
        type: Utilization
    type: Resource
```

The deployment and the service resources should now be familiar to you, as
these are similar to the ones used in the `service` part of this demo. The only
difference is that in this deployment, we are defining resources for each
container. Here, we define that our container requests `50m` which mean 50
millicpu, i.e., 0.05 share of a vCPU's resources.

The last resource in our manifest is the HPA. Here we define that we want to
scale our notifications-service deployment. We also want to set upper and lower
bound limits on the number of pods the HPA can scale, to 5 and 1 respectively.
We then define that the resource we want our HPA to monitor is the vCPU usage,
and we want an average usage of 20%. 

Based on the rules defined in the HPA's manifest, the autoscaler will collect
metrics from our deployment and calculate the number of pods required to
satisfy our conditions. The algorithm the HPA autoscaler uses to determine the number of pods required is as follows: 
```
desiredReplicas = ceil[currentReplicas * ( currentMetricValue / desiredMetricValue )]
```
So, if we have 2 pods, with an average usage of 30%, and we want our pods with
an average usage of 20%, the `desiredReplicas` will be `2*0.3/0.2` which is
equal to `3` replicas (pods).

First, enable the metrics-server addon on minikube:
```bash
minikube addons enable metrics-server
```

Create the resources:
```bash
kubectl apply -f hpa/hpa.yml
```

And now wait for the pod to get into a `Running` status:
```bash
watch kubectl get pods
```

To stress the notifications-service, run the following command:
```bash
for i in $(seq 1 100000); do 
    curl -X 'POST' \
        'http://192.168.49.2:30674/api/notify' \
        -H 'accept: text/plain; charset=utf-8' \
        -H 'Content-Type: application/json; charset=utf-8' \
        -d '{
            "notification_type": "OutOfRange",
            "researcher": "d.landau@uu.nl",
            "measurement_id": "1234",
            "experiment_id": "5678",
            "cipher_data": "D5qnEHeIrTYmLwYX.hSZNb3xxQ9MtGhRP7E52yv2seWo4tUxYe28ATJVHUi0J++SFyfq5LQc0sTmiS4ILiM0/YsPHgp5fQKuRuuHLSyLA1WR9YIRS6nYrokZ68u4OLC4j26JW/QpiGmAydGKPIvV2ImD8t1NOUrejbnp/cmbMDUKO1hbXGPfD7oTvvk6JQVBAxSPVB96jDv7C4sGTmuEDZPoIpojcTBFP2xA"
        }'; 
    sleep 0.00001
done
```

Open 2 new ssh sessions (in addition to the one running the stress test) to
your VM and in the first run:
```bash
watch kubectl get pods
```
and on the second run 
```bash
watch kubectl get hpa
```

You should verify that the HPA is evaluating our deployment's current load, and
increasing/decreasing the number of replicas.

You can terminate the stress test with `Ctrl-C` and delete the resources with:
```bash
kubectl delete -f hpa/hpa.yml
```
