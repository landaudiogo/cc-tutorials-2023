# Overview

This tutorials aims to show how distributed systems communicate with one
another through publish/subscribe communication protocols. We will specifically
use Kafka, which is a common message broker implementations used in the
industry due to its scalability, fault tolerance, availability, etc...

# Kafka Credentials

A development Kafka cluster is setup to which we will connect to throughout
this tutorial. To access the cluster, each student should have been provided a
OneDrive folder with the required credentials. We will now upload the folder to
our respective VMs.

## Demo

Let's start with downloading our folders into a known location in our
computers. This part of the tutorial will depend on your ssh client. In my
case, I am running an Ubuntu Linux distro with an openssh client. I will
download my folder to `~/Downloads` which will end up being the
`~/Downloads/client45` directory.

### openssh

If running and openssh client, you can run this command from 

```bash
scp -r -i <your-pem-file> ubuntu@<your-VMs-public-IP>:<tutorials-directory> <folder-path-just-downloaded>

E.g. scp -r -i ~/.config/keys/landau-cc-2023.pem ~/Downloads/client60 ubuntu@13.48.5.125:/home/ubuntu/repos/cc-2023-tutorials
```

### Other clients

If running other clients they might have a specific way of allowing files to be
transferred. If in doubt, raise your hand.

# Simple Producer Consumer

We will now create a simple data producer, that will write messages/events into
our kafka topic. At the same time, we will start a simple consumer that will
read the messages that are being published. These are the simples interactions
2 clients may have with the Kafka cluster. 

The producer, 
```python
# simple_producer_consumer/producer.py
import click

from confluent_kafka import Producer


p = Producer({
    'bootstrap.servers': '13.49.128.80:19093,13.49.128.80:29093,13.49.128.80:39093',
    'security.protocol': 'SSL',
    'ssl.ca.location': './auth/ca.crt',
    'ssl.keystore.location': './auth/kafka.keystore.pkcs12',
    'ssl.keystore.password': 'cc2023',
    'ssl.endpoint.identification.algorithm': 'none',
})

@click.command()
@click.argument('topic')
def produce(topic: str): 
    while True: 
        message = input()
        p.produce(topic, message.encode('utf-8'))
        p.flush()


produce()
```
is configured, reads in an argument passed in as the topic. The topic you will
subscribe to is your client `id`, e.g., in my case it is `client60`. It then
starts a loop where it asks for a user's input, and sends the message to kafka.

With regards to the consumer,
```python
# simple_producer_consumer/consumer.py
import click 

from confluent_kafka import Consumer


c = Consumer({
    'bootstrap.servers': '13.49.128.80:19093',
    'group.id': 'simple_consumer',
    'auto.offset.reset': 'latest',
    'security.protocol': 'SSL',
    'ssl.ca.location': './auth/ca.crt',
    'ssl.keystore.location': './auth/kafka.keystore.pkcs12',
    'ssl.keystore.password': 'cc2023',
    'enable.auto.commit': 'true',
    'ssl.endpoint.identification.algorithm': 'none',
})

@click.command()
@click.argument('topic')
def consume(topic: str): 
    c.subscribe([topic])

    while True:
        msg = c.poll(1.0)
        if msg is None:
            continue
        if msg.error():
            print("Consumer error: {}".format(msg.error()))
            continue
        print(msg.value())

consume()
```
it is first configured, and receives as input the topic to subscribe to.
This must be the same as the topic passed to the producer. It then enters an
infinite loop that polls to check whether there are any messages to be
consumed, and if so prints them to standard output. 

# Demo

Start an ssh session with your vm. Change directory into the tutorial
repository's directory, followed by: 
```bash
cd kafka
```

We will start by building our image: 
```bash
docker build -t tkafka/simple simple_producer_consumer
```

We may now start our consumer. Don't forget to change the topic `<topic>`
argument: 
```bash
docker run \
    --rm \
    -d \
    --name simple_consumer \
    -v "$(pwd)/auth":/usr/src/app/auth \
    tkafka/simple consumer.py "<topic>"
```

We can now also run our producer. Also don't forget to change the <topic> field:

> Note: 
> Because our producer is requesting user input, we are running our producer in
> interactive mode with the options `-it`

```bash
docker run \
    --rm \
    -it \
    --name simple_producer \
    -v "$(pwd)/auth":/usr/src/app/auth \
    tkafka/simple producer.py "<topic>"
```

# Commit Offsets

Our goal now is to understand the commit offsets interface. This functionality
exists in Kafka, so it can track where a client left off the last time it
connected to a topic, so it does not have to read the data from the beginning
all the time it reconnects.

The client identifier kafka uses to locate in what offset in each topic
partition a client is located, is the `group.id` configuration parameter.

There are 2 behaviours. If Kafka finds an offset for the client + the topic it
is consuming from, then it starts where it last left off. Otherwise, it behaves
based on the parameter `auto.offset.reset`. 

`auto.offset.reset` can be set to `latest` or `earliest`. 

- `latest` is it will ignore any messages published before the time it
  connected, and consume only new messages
- `earliest` will start consuming from the oldest message in each partition of
  the topic.

> Important! 
>
> What is described in this section will be important if you have to delete
> previous messages published into the kafka topic. Kafka does not allow
> deleting any of the messages you published, and therefore you have to rely on
> this behaviour to start consuming only what will be published from that
> moment forward. 

## Demo

To demonstrate this functionality, we will start by creating 2 different
consumers that have never read from the topic before. Their functionality will
be similar to the last consumer's, but we will change the `auto.offset.reset`
parameter:
```bash
docker build -t tkafka/commit_offsets commit_offsets

docker run \
    --rm \
    -d \
    --name consumer_earliest \
    -v "$(pwd)/auth":/usr/src/app/auth \
    tkafka/commit_offsets consumer.py "<topic>" "earliest"

docker run \
    --rm \
    -d \
    --name consumer_latest \
    -v "$(pwd)/auth":/usr/src/app/auth \
    tkafka/commit_offsets consumer.py "<topic>" "latest"
```

If we now look at the logs of each consumer we will see that the consumer
defined with `latest` will not print the messages your wrote on the first part
of the assignment, whereas the consumer with the `earliest` value will print
everything from the beginning: 

> Note:
> Wait until you at least see the on_assign callback printing the partitions
> assigned to each consumer before closing the logs.

```bash
docker logs -f consumer_earliest
```

```bash
docker logs -f consumer_latest
```

We can now delete the 2 consumers we have created: 
```bash
docker stop consumer_earliest consumer_latest
```

We now want to understand how the `group.id` can be used to save a consumer's
position in each topic partition. With each message we read from our topic, we
will pretend our consumer has to make sure it is storing the data in a database
before it commits the offset to Kafka. This is represented with a short sleep:
```python
# commit_offsets/consumer_commit.py
# ...
        print(msg.value())
        store_data()
        c.commit(message=msg)
        print("Message committed")
# ... 
```

We will now start our consumer which should start slowly printing each message
and the other debug information (don't forget to change the `<topic>` and
`<unique-group-id>`):
```bash
docker run \
    --rm \
    -d \
    --name consumer_commit \
    -v "$(pwd)/auth":/usr/src/app/auth \
    tkafka/commit_offsets consumer_commit.py "<topic>" "<unique-group-id>"

docker logs -f consumer_commit
```

Stop the consumer as it between the "Start Storing" and the "End Storing" messages.
```bash
docker stop consumer_commit
```

We will now restart the consumer:
```bash
docker run \
    --rm \
    -d \
    --name consumer_commit \
    -v "$(pwd)/auth":/usr/src/app/auth \
    tkafka/commit_offsets consumer_commit.py "<topic>" "<unique-group-id>"

docker logs -f consumer_commit
```

We expect that the consumer re-reads the same message where it was interrupted
before commiting the message. 

We can now delete the consumer: 
```bash
docker stop consumer_commit
```

# Consumer Group

A consumer group is an abstraction kafka provides that allows parallelizing the
data consumption between the consumers belonging to the same group. The effect
is an increase of the rate at which data can be consumed.

But how do multiple consumers read from the same topic, without reading
overlapping messages? What enables this behaviour is Kafka's topic
partitioning, wherein a single topic can have multiple partitions, e.g., in our
case, each of our topics has 16 partitions. 

To guarantee all messages produced to the topic are read by the group, the rule
is to guarantee each partition is assigned a consumer within the group. When
there are multiple active consumer's belonging to the same group, kafka
attempts to balance out the load between the different consumers by assigning
them an equal amount of partitions. Using our case as an example again, if we
have 2 consumers in our group reading from our topic, then each consumer will
be assigned 8 partitions. 

Partitions is also the unit where kafka guarantees message ordering. What
determines which partition a message will be assigned to is the partition key
associated with a message. If 2 messages have the same partition key, then it
is guaranteed they will be sent to the same partition. In our assignment, the
key of all messages for an experiment is the experiment's identifier, so as to
make sure that the messages will be consumed in the same order as they were
produced. If on the other hand the messages were published into different
partitions, there would be no guarantee that the messages would be read in the
correct order. 

## Demo

We will make use of our producer from before with the added code that includes
a key on each message. The key value increments everytime we write a new
message to try and make it go to different topics. 

```python
# consumer_group/producer.py
# ...
        p.produce(topic, key=f"{i}", value=message.encode('utf-8'))
        p.flush()
        i+=1
# ...
```

We will now start 2 consumers with the `"group.id": "consumer_group_1"`,
meaning they will belong to the same consumer group:

```bash
docker build -t tkafka/consumer_group consumer_group

docker run \
    --rm \
    -d \
    --name consumer_1 \
    -v "$(pwd)/auth":/usr/src/app/auth \
    tkafka/consumer_group consumer.py "<topic>"

docker run \
    --rm \
    -d \
    --name consumer_2 \
    -v "$(pwd)/auth":/usr/src/app/auth \
    tkafka/consumer_group consumer.py "<topic>"
```

If we now check their logs, we may see that after the partitions have been
assigned, each consumer has been assigned 8 of the 16 partitions, without any
overlap.

```bash
docker logs -f consumer_1 # wait for the printed partition assignment
docker logs -f consumer_2 # wait for the printed partition assignment
```

We will now start our producer which will write data to the partitions "at
random":
```bash
docker run \
    --rm \
    -it \
    --name producer \
    -v "$(pwd)/auth":/usr/src/app/auth \
    tkafka/consumer_group producer.py "<topic>"
```

We expect that as you write messages either `consumer_1` or `consumer_2` is
getting them, but NEVER BOTH.

We can now stop our consumers:
```bash
docker stop consumer_1 consumer_2
```
