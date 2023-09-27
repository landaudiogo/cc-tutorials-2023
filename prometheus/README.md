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
