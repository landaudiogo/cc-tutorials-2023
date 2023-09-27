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


