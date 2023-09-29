How do I get a pod's (milli)core CPU usage:
https://stackoverflow.com/questions/48872042/how-do-i-get-a-pods-millicore-cpu-usage-with-prometheus-in-kubernetes

Install Helm:
https://helm.sh/docs/intro/install/

Install Prometheus and Grafana:
https://www.youtube.com/watch?v=dk2-_DbWb80&list=PLVx1qovxj-anCTn6um3BDsoHnIr0O2tz3&index=1&ab_channel=Thetips4you

Tasks:

- [x] Research node-exporter CPU and Memory metrics
- [x] Test whether host is reachable from docker container
    - [x] Try --add-host host.docker.internal:host-gateway
    - [x] Try host IP
- [x] Create local setup
    - [x] PoC
    - [x] Prometheus server
    - [x] Grafana
    - [x] node-exporter
    - [x] Write-up
- [x] Create k8s setup
    - [x] PoC
    - [x] Helm Install kube-prometheus
    - [x] Grafana dashboard (optional)
    - [x] Write-up
- [x] Serverless python-db load read & write load generation
- [x] Find CPU related query
- [x] Find Memory related query
- [x] Find disk related query
- [x] Find network related query
- [x] Test the service on test tutorial VM
- [ ] Read through the tutorial

```
CPU: 
sum(node_cpu_seconds_total{mode!="idle"} @ 1695708537)
sum(node_cpu_seconds_total{mode!="idle"} @ 1695708537) - sum(node_cpu_seconds_total{mode!="idle"} @ 1695726537)
increase(node_cpu_seconds_total{mode!="idle"}[15m] @ 1695727818)

Memory: 
node_memory_MemTotal_bytes - node_memory_MemAvailable_bytes
```
