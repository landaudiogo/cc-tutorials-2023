# Overview

An observable system is one that collects data to provide insights as to how it
is performing. To achieve this, it is common to use monitoring and
visualization tooling, such as Prometheus for monitoring and metric collection
and Grafana for visualization.

*"Prometheus is an open-source systems monitoring and alerting toolkit"*
[link](https://prometheus.io/docs/introduction/overview/). A Prometheus server
scrapes the **targets** configured at a specific rate, and stores the data in a
time-series database. The timeseries data can then be queried resorting to
PromQL.

*"Grafana enables you to query, visualize, alert on, and explore your metrics,
logs, and traces wherever they are stored"*
[link](https://grafana.com/docs/grafana/latest/introduction/). It is common to
use grafana as a visualization tool of the data collected by the prometheus
server.

Throughout this tutorial we will setup 2 observability setups: 

- Local setup (docker & host)
- k8s setup

# Local Setup

This local setup is also the setup we will use throughout the demo to evaluate
the cost of your microservice infrastructure.

It consists of 3 components: 

1. Prometheus node_exporter: This service collects data from our node/VM/host
   and exposes the data through an HTTP API. 
1. Prometheus server: The prometheus server collects the data from a list of
   targets (node_exporter), and stores it in a timeseries database.
1. Grafana: Hosts a dashboard to visualize the infrastructure cost metrics.

To download the node exporter, we download the compressed binary file,
uncompress it, and execute the binary:
```bash
wget -O local-setup/node_exporter.tar.gz https://github.com/prometheus/node_exporter/releases/download/v1.6.1/node_exporter-1.6.1.linux-amd64.tar.gz
mkdir local-setup/node-exporter
tar xvfz local-setup/node_exporter.tar.gz --directory local-setup/node-exporter --strip-components=1
./local-setup/node-exporter/node_exporter "--web.listen-address=[0.0.0.0]:9100"
```

If we run the following command, we should see a list of metrics the
node_exporter exposes.
```bash
curl localhost:9100/metrics
```

To start our prometheus and grafana services, a `docker-compose.yml` file has
been created. Start the services:
```bash
cd local-setup && docker compose up -d && cd - 
```

Open your browser and insert the following link (change the `<your-vm-ip>`
placeholder to the ip you use to ssh into your VM):
```
<your-vm-ip>:5010
```

You should see a login page. Type `admin` for the username and password.

First, on the side panel, click on `Connections` > `Data Sources`. Click on `+
Add new data source` and click on `Prometheus`. Fill in the `Prometheus server
url` field with `http://prometheus:9090`. To finalize click on `Save & test`

Now open the dashboards view, and click on `New` > `Import`. You should see an
input box that has as label "Import via panel json". In that box, place the
following json content, and click `Load` > `Import`:
```json
{
  "annotations": {
    "list": [
      {
        "builtIn": 1,
        "datasource": {
          "type": "grafana",
          "uid": "-- Grafana --"
        },
        "enable": true,
        "hide": true,
        "iconColor": "rgba(0, 211, 255, 1)",
        "name": "Annotations & Alerts",
        "type": "dashboard"
      }
    ]
  },
  "editable": true,
  "fiscalYearStartMonth": 0,
  "graphTooltip": 0,
  "id": 1,
  "links": [],
  "liveNow": false,
  "panels": [
    {
      "datasource": {
        "type": "prometheus",
        "uid": "e5417d40-376e-49e3-b704-6fe313af120d"
      },
      "description": "This is computed as MemTotal - MemAvailable",
      "fieldConfig": {
        "defaults": {
          "color": {
            "mode": "continuous-GrYlRd"
          },
          "custom": {
            "axisCenteredZero": false,
            "axisColorMode": "text",
            "axisLabel": "",
            "axisPlacement": "auto",
            "barAlignment": 0,
            "drawStyle": "line",
            "fillOpacity": 20,
            "gradientMode": "scheme",
            "hideFrom": {
              "legend": false,
              "tooltip": false,
              "viz": false
            },
            "insertNulls": false,
            "lineInterpolation": "smooth",
            "lineWidth": 3,
            "pointSize": 5,
            "scaleDistribution": {
              "type": "linear"
            },
            "showPoints": "auto",
            "spanNulls": false,
            "stacking": {
              "group": "A",
              "mode": "none"
            },
            "thresholdsStyle": {
              "mode": "off"
            }
          },
          "mappings": [],
          "thresholds": {
            "mode": "absolute",
            "steps": [
              {
                "color": "green",
                "value": null
              },
              {
                "color": "red",
                "value": 80
              }
            ]
          },
          "unit": "bytes"
        },
        "overrides": []
      },
      "gridPos": {
        "h": 8,
        "w": 12,
        "x": 0,
        "y": 0
      },
      "id": 1,
      "options": {
        "legend": {
          "calcs": [],
          "displayMode": "list",
          "placement": "bottom",
          "showLegend": true
        },
        "tooltip": {
          "mode": "single",
          "sort": "none"
        }
      },
      "targets": [
        {
          "datasource": {
            "type": "prometheus",
            "uid": "e5417d40-376e-49e3-b704-6fe313af120d"
          },
          "editorMode": "code",
          "expr": "node_memory_MemTotal_bytes - node_memory_MemAvailable_bytes\n",
          "instant": false,
          "legendFormat": "__auto",
          "range": true,
          "refId": "A"
        }
      ],
      "title": "Memory Used",
      "type": "timeseries"
    },
    {
      "datasource": {
        "type": "prometheus",
        "uid": "e5417d40-376e-49e3-b704-6fe313af120d"
      },
      "fieldConfig": {
        "defaults": {
          "color": {
            "mode": "palette-classic"
          },
          "custom": {
            "axisCenteredZero": false,
            "axisColorMode": "text",
            "axisLabel": "",
            "axisPlacement": "auto",
            "barAlignment": 0,
            "drawStyle": "line",
            "fillOpacity": 0,
            "gradientMode": "none",
            "hideFrom": {
              "legend": false,
              "tooltip": false,
              "viz": false
            },
            "insertNulls": false,
            "lineInterpolation": "linear",
            "lineWidth": 1,
            "pointSize": 5,
            "scaleDistribution": {
              "type": "linear"
            },
            "showPoints": "auto",
            "spanNulls": false,
            "stacking": {
              "group": "A",
              "mode": "none"
            },
            "thresholdsStyle": {
              "mode": "off"
            }
          },
          "mappings": [],
          "thresholds": {
            "mode": "absolute",
            "steps": [
              {
                "color": "green",
                "value": null
              },
              {
                "color": "red",
                "value": 80
              }
            ]
          },
          "unit": "percent"
        },
        "overrides": []
      },
      "gridPos": {
        "h": 8,
        "w": 12,
        "x": 12,
        "y": 0
      },
      "id": 2,
      "options": {
        "legend": {
          "calcs": [],
          "displayMode": "list",
          "placement": "bottom",
          "showLegend": true
        },
        "tooltip": {
          "mode": "single",
          "sort": "none"
        }
      },
      "targets": [
        {
          "datasource": {
            "type": "prometheus",
            "uid": "e5417d40-376e-49e3-b704-6fe313af120d"
          },
          "editorMode": "code",
          "expr": "(sum(rate(node_cpu_seconds_total{mode!=\"idle\"}[2m])) / count(node_cpu_seconds_total{mode=\"idle\"})) * 100 \n",
          "instant": false,
          "legendFormat": "__auto",
          "range": true,
          "refId": "A"
        }
      ],
      "title": "CPU usage",
      "type": "timeseries"
    }
  ],
  "refresh": "",
  "schemaVersion": 38,
  "style": "dark",
  "tags": [],
  "templating": {
    "list": []
  },
  "time": {
    "from": "now-12h",
    "to": "now"
  },
  "timepicker": {},
  "timezone": "",
  "title": "Deployment Cost",
  "uid": "b2ce7b20-b098-485a-a8e9-e52541fd2e7e",
  "version": 6,
  "weekStart": ""
}
```

You should now see 2 visualizations, albeit with an error. To fix the error,
edit each of the visualizations, and change the data source to the one you just
created. Update the query by adding or removing any white-space, and click the
refresh button at the top right corner of the visualization and the graph data
should now be available.

# k8s Setup

Despite all the advantages of managing containerized applications in Kubernetes,
it also adds some additional complexity to infering the state of our
applications in this distributed environment. 

Luckily, thanks to prometheus-community, there are some automated deployment
**charts** that deploy the same monitoring infrastructure as in the previous
section in k8s. For this, we will use helm charts.   

For this part, make sure minikube is running: 
```bash
minikube start
```

First, let's install helm: 
```bash
curl https://raw.githubusercontent.com/helm/helm/main/scripts/get-helm-3 | bash
```

Test running helm with:
```bash
helm version
```

Add the `prometheus-community` repository:
```bash
helm repo add prometheus-community https://prometheus-community.github.io/helm-charts 
helm repo update
```

Deploy prometheus' helm chart in our k8s cluster:
```bash
helm install prometheus prometheus-community/prometheus
```

By default, the prometheus server is not exposed to processes running outside
our cluster. As such, we expose the prometheus-server on port 3001 of our host
with:
```bash
kubectl apply -f k8s-setup/expose-prometheus.yml
docker run -d --rm -it --network=host alpine ash -c "apk add socat && socat TCP-LISTEN:3001,reuseaddr,fork TCP:$(minikube ip):30900"
```

Try accessing your prometheus-server via your browser (don't
forget to change the `<your-vm-ip>` placeholder):
```
<your-vm-ip>:3001
```

Now we add the grafana helm repository:
```bash
helm repo add grafana https://grafana.github.io/helm-charts
helm repo update
```
and install the grafana helm chart in our k8s cluster:
```bash
helm install grafana grafana/grafana
```

Similarly, expose grafana over your host's 3002 port with:
```bash
kubectl apply -f k8s-setup/expose-grafana.yml
docker run -d --rm -it --network=host alpine ash -c "apk add socat && socat TCP-LISTEN:3002,reuseaddr,fork TCP:$(minikube ip):30910"
```

Copy the password provided by the following command:
```bash
kubectl get secrets grafana -o jsonpath='{.data.admin-password}' | base64 --decode
```

Now you can open your browser and access grafana on the link (don't forget to
change the `<your-vm-ip>` placeholder):
```
<your-vm-ip>:3002
```

With regards to the credentials, the username is `admin` and password is the
output of the last command you executed `kubectl get secrets ...`.

The following steps are similar to what we have done in the local-setup. Add a
prometheus datasource that with the `Prometheus server url` field filled to
`http://<your-vm-ip>:9090`.

We will now explore some of the different metrics exposed by the
prometheus-server deployed in our k8s cluster. Our goal will be to analyze how
one of our serverless functions is behaving while performing a load test.

Let's first create the service we will track:
```bash
kubectl apply -f ../openfaas/python-db/postgre-db.yml
faas-cli up -f ../openfaas/python-db.yml
```

Install our load generator:
```bash
arkade get hey
```

Now open another ssh session. On the first session, run the following read
load:
```bash
hey -t 10 -z 1m -c 20 -q 5 \
    "http://localhost:8080/function/python-db"
```
and on the second, run the write load generator:
```bash
hey -t 10 -z 1m -c 20 -q 5 -m POST \
    "http://localhost:8080/function/python-db?researcher=test@uu.nl"
```

Open your k8s grafana in your browser and select the `Explore` page on the side
view. Select your prometheus datasource, and let's start querying our
prometheus-server for some data related to the stress test we have just
executed.

To evaluate the performance of our services, there are 4 main resources we
might want to track: CPU; Memory; Disk; Network. CPU and memory related metrics
will have a direct impact on the total cost of your infrastructure, whereas
network and disk might have an impact on your service's latency.

I would recommend you try your own queries to start getting a feel of what sort
of metrics are available. However, here are a few queries for you to try out
(one per resource). For more information on container related metrics, read
[this](https://blog.freshtracks.io/a-deep-dive-into-kubernetes-metrics-part-3-container-resource-metrics-361c5ee46e66)
link.

- CPU - Show the amount of CPU used in seconds by our postgre-db pods:
  ```promql
  rate(container_cpu_usage_seconds_total{pod="postgre-database"}[3m])
  ```
- Memory - This first query includes the memory used by the process + OS level
  cache (e.g. filesystem cache). Some data in the cache entries can be evicted
  to make room in the case the container is under memory pressure. Here we are
  computing the rate at which the amount of memory in use increases.
  ```promql
  rate(container_memory_usage_bytes{pod="postgre-database"}[3m])
  ```

  The working set size is the metric tracked by the OOM (Out-of-memory) killer
  to determine whether a container has to be killed if it's exceeding its
  memory limits.
  ```promql
  rate(container_memory_working_set_bytes{pod="postgre-database"}[3m])
  ```

  For more information on these metrics, refer to
  [this](https://faun.pub/how-much-is-too-much-the-linux-oomkiller-and-used-memory-d32186f29c9d)
  link.
- Disk - This metric sums the amount of data written by the `postgre-database`
  container into all block devices:
  ```promql
  sum(rate(container_fs_writes_bytes_total{pod="postgre-database"}[3m]))
  ```

  Similarly, this query computes the amount of data read by our
  `postgre-database` container from all block devices:
  ```promql
  sum(rate(container_fs_reads_bytes_total{pod="postgre-database"}[3m]))
  ```

  **Note: Not all filesystem reads result in a read from disk. After reading
  the contents from a file, the operating system might cache its contents so
  that they are available in memory. This is also the cached filesystem content
  mentioned in the memory bullet point.**
- Network - Compute the rate at which data was received on each interface:
  ```promql
  rate(container_network_receive_bytes_total[3m])
  ```

  Compute the rate at which data was transmitted on each interface:
  ```promql
  rate(container_network_transmit_bytes_total[3m])
  ```
