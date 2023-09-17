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

