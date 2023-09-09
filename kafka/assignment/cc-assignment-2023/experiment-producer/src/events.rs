use apache_avro::types::{Record, Value};
use apache_avro::{Schema, Writer, Reader};
use rdkafka::{
    config::ClientConfig,
    error::KafkaError,
    message::{OwnedMessage, ToBytes, OwnedHeaders},
    producer::{FutureProducer, FutureRecord},
};
use std::{fs, time::Duration};
use std::fmt::Debug;
use uuid::Uuid;

use event_hash::{HashData, NotificationType};

use crate::simulator::{self, ExperimentStage, IterMut, TemperatureSample};
use crate::time;

/// `Vec<u8>` wrapper
///
/// FutureRecord::payload requires a type that implements the trait `ToBytes` as an argument. This is our
/// custom type to implement the trait.
pub struct EventWrapper(Vec<u8>);

impl<'a> ToBytes for EventWrapper {
    fn to_bytes(&self) -> &[u8] {
        &self.0
    }
}

pub fn experiment_configured_event(
    experiment_id: &str,
    researcher: &str,
    sensors: &Vec<String>,
    upper_threshold: f32,
    lower_threshold: f32,
) -> EventWrapper {
    let raw_schema =
        fs::read_to_string("experiment-producer/schemas/experiment_configured.avsc").unwrap();
    let schema = Schema::parse_str(&raw_schema).unwrap();
    let mut writer = Writer::new(&schema, Vec::new());

    let mut record = Record::new(writer.schema()).unwrap();
    record.put("experiment", experiment_id);
    record.put("researcher", researcher);
    let sensors = Value::Array(sensors.into_iter().map(|v| (&**v).into()).collect());
    record.put("sensors", sensors);

    let schema_json: serde_json::Value = serde_json::from_str(&raw_schema).unwrap();
    let temp_schema_json = &schema_json["fields"][3]["type"];

    let temp_schema = Schema::parse_str(&temp_schema_json.to_string()).unwrap();
    let mut temp_range = Record::new(&temp_schema).unwrap();
    temp_range.put("upper_threshold", Value::Float(upper_threshold));
    temp_range.put("lower_threshold", Value::Float(lower_threshold));
    record.put("temperature_range", temp_range);
    writer.append(record).unwrap();

    EventWrapper(writer.into_inner().unwrap())
}

pub fn stabilization_started_event(experiment_id: &str) -> EventWrapper {
    let raw_schema =
        fs::read_to_string("experiment-producer/schemas/stabilization_started.avsc").unwrap();
    let schema = Schema::parse_str(&raw_schema).unwrap();
    let mut writer = Writer::new(&schema, Vec::new());

    let mut record = Record::new(writer.schema()).unwrap();
    record.put("experiment", experiment_id);

    let current_time = time::current_epoch();
    record.put("timestamp", Value::Double(current_time));
    writer.append(record).unwrap();

    EventWrapper(writer.into_inner().unwrap())
}

pub fn experiment_started_event(experiment_id: &str) -> EventWrapper {
    let raw_schema =
        fs::read_to_string("experiment-producer/schemas/experiment_started.avsc").unwrap();
    let schema = Schema::parse_str(&raw_schema).unwrap();
    let mut writer = Writer::new(&schema, Vec::new());

    let mut record = Record::new(writer.schema()).unwrap();
    record.put("experiment", experiment_id);

    let current_time = time::current_epoch();
    record.put("timestamp", Value::Double(current_time));
    writer.append(record).unwrap();

    EventWrapper(writer.into_inner().unwrap())
}

pub fn experiment_terminated_event(experiment_id: &str) -> EventWrapper {
    let raw_schema =
        fs::read_to_string("experiment-producer/schemas/experiment_terminated.avsc").unwrap();
    let schema = Schema::parse_str(&raw_schema).unwrap();
    let mut writer = Writer::new(&schema, Vec::new());

    let mut record = Record::new(writer.schema()).unwrap();
    record.put("experiment", experiment_id);

    let current_time = time::current_epoch();
    record.put("timestamp", Value::Double(current_time));

    writer.append(record).unwrap();
    EventWrapper(writer.into_inner().unwrap())
}

pub fn temperature_measured_event(
    experiment: &str,
    measurement_id: &str,
    sensor: &str,
    temperature: f32,
    timestamp: f64,
    measurement_hash: &str,
) -> EventWrapper {
    let raw_schema =
        fs::read_to_string("experiment-producer/schemas/sensor_temperature_measured.avsc").unwrap();
    let schema = Schema::parse_str(&raw_schema).unwrap();
    let mut writer = Writer::new(&schema, Vec::new());

    let mut record = Record::new(writer.schema()).unwrap();
    record.put("experiment", experiment);
    record.put("sensor", sensor);
    record.put("measurement_id", measurement_id);
    record.put("temperature", temperature);
    record.put("measurement_hash", measurement_hash);
    record.put("timestamp", Value::Double(timestamp));

    writer.append(record).unwrap();
    let encoded = writer.into_inner().unwrap();

    let reader = Reader::new(&encoded[..]).unwrap();
    for value in reader {
        println!("{:#?}", value);
    }
    EventWrapper(encoded)
}

fn compute_notification_type(
    curr_sample: TemperatureSample,
    prev_sample: Option<TemperatureSample>,
    stage: &ExperimentStage,
) -> Option<NotificationType> {
    match (stage, prev_sample) {
        (ExperimentStage::Stabilization, Some(prev_sample)) => {
            if !curr_sample.is_out_of_range() && prev_sample.is_out_of_range() {
                println!("=== Stabilized ===");
                Some(NotificationType::Stabilized)
            } else {
                None
            }
        }
        (ExperimentStage::CarryOut, Some(prev_sample)) => {
            if curr_sample.is_out_of_range() && !prev_sample.is_out_of_range() {
                println!("=== Out of Range ===");
                Some(NotificationType::OutOfRange)
            } else {
                None
            }
        }
        (ExperimentStage::Stabilization, None) => {
            if !curr_sample.is_out_of_range() {
                println!("=== Stabilized ===");
                Some(NotificationType::Stabilized)
            } else {
                None
            }
        }
        (ExperimentStage::CarryOut, None) => {
            if curr_sample.is_out_of_range() {
                println!("=== Out of Range ===");
                Some(NotificationType::OutOfRange)
            } else {
                None
            }
        }
        (_, _) => None,
    }
}

// TODO
// key paramater
pub fn temperature_events<'a>(
    sample_iter: IterMut<'a>,
    experiment_id: &'a str,
    researcher: &'a str,
    sensors: &'a Vec<String>,
    stage: &'a ExperimentStage,
    secret_key: &'a str,
) -> Box<dyn Iterator<Item = Vec<EventWrapper>> + 'a> {
    let mut prev_sample = None;

    Box::new(sample_iter.map(move |sample| {
        let measurement_id = &format!("{}", Uuid::new_v4());
        let current_time = time::current_epoch();

        let hash_data = HashData {
            notification_type: compute_notification_type(sample, prev_sample, &stage),
            timestamp: current_time,
            experiment_id: experiment_id.into(),
            measurement_id: measurement_id.into(),
            researcher: researcher.into(),
        };
        let measurement_hash = hash_data.encrypt(secret_key.as_bytes());
        prev_sample = Some(sample);

        simulator::compute_sensor_temperatures(&sensors, sample.cur())
            .into_iter()
            .map(|(sensor_id, sensor_temperature)| {
                temperature_measured_event(
                    experiment_id,
                    measurement_id,
                    sensor_id,
                    sensor_temperature,
                    current_time,
                    &measurement_hash,
                )
            })
            .collect()
    }))
}

pub struct RecordData<K: ToBytes, T: ToBytes> {
    pub payload: T,
    pub key: Option<K>,
    pub headers: OwnedHeaders,
}

pub struct KafkaTopicProducer {
    topic: String,
    producer: FutureProducer, // partition: Option<usize>
}

impl KafkaTopicProducer {
    pub fn new(brokers: &str, topic: &str) -> Self {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("message.timeout.ms", "5000")
            .set("security.protocol", "SSL")
            .set("ssl.ca.location", "experiment-producer/auth/ca.crt")
            .set("ssl.keystore.location", "experiment-producer/auth/kafka.keystore.pkcs12")
            .set("ssl.keystore.password", "cc2023")
            .create()
            .expect("Producer creation error");
        KafkaTopicProducer {
            topic: topic.into(),
            producer,
        }
    }

    pub async fn send_event<'a, K, T>(
        &self,
        record: RecordData<K, T>,
    ) -> Result<(i32, i64), (KafkaError, OwnedMessage)>
    where
        T: ToBytes,
        K: ToBytes,
    {
        let mut future_record: FutureRecord<'_, K, T> =
            FutureRecord::to(&self.topic).payload(&record.payload).headers(record.headers);
        if record.key.is_some() {
            future_record = future_record.key(record.key.as_ref().unwrap());
        }

        self.producer
            .send(future_record, Duration::from_secs(0))
            .await
    }
}
