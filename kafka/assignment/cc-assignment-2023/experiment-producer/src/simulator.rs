use rand::Rng;
use std::time::Duration;
use tokio::time;
use rdkafka::message::OwnedHeaders;

use crate::events::{self, KafkaTopicProducer, RecordData};

#[derive(Clone, Copy)]
pub enum ExperimentStage {
    Uninitialized,
    Configuration,
    Stabilization,
    CarryOut,
    Terminated,
}

#[derive(Clone, Copy, Debug)]
pub struct TempRange {
    lower_threshold: f32,
    upper_threshold: f32,
}

impl TempRange {
    pub fn new(lower_threshold: f32, upper_threshold: f32) -> Option<Self> {
        if lower_threshold > upper_threshold {
            return None;
        }
        Some(Self {
            upper_threshold,
            lower_threshold,
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct TemperatureSample {
    cur: f32,
    temp_range: TempRange,
}

impl TemperatureSample {
    pub fn is_out_of_range(&self) -> bool {
        self.cur > self.temp_range.upper_threshold || self.cur < self.temp_range.lower_threshold
    }

    pub fn cur(&self) -> f32 {
        self.cur
    }

    pub fn iter_mut(&mut self, delta: f32, len: usize, random_range: f32) -> IterMut {
        IterMut {
            sample: self,
            iteration: 0,
            delta,
            len,
            random_range,
        }
    }

    pub fn stabilization_samples(&mut self, len: usize) -> IterMut {
        let TempRange {
            lower_threshold,
            upper_threshold,
        } = self.temp_range;
        let final_temperature = lower_threshold + (upper_threshold - lower_threshold) / 2_f32;
        let delta = (final_temperature - self.cur) / (len as f32);
        self.iter_mut(delta, len, 0.0)
    }

    pub fn carry_out_samples(&mut self, len: usize) -> IterMut {
        let TempRange {
            lower_threshold,
            upper_threshold,
        } = self.temp_range;
        self.iter_mut(0.0, len, upper_threshold - lower_threshold)
    }
}

pub struct ExperimentConfiguration {
    pub experiment_id: String,
    pub researcher: String,
    pub sensors: Vec<String>,
    pub sample_rate: u64,
    pub secret_key: String,
}

pub struct Experiment {
    sample: TemperatureSample,
    stage: ExperimentStage,
    config: ExperimentConfiguration,
    producer: KafkaTopicProducer,
}

impl Experiment {
    pub fn new(
        start: f32,
        temp_range: TempRange,
        config: ExperimentConfiguration,
        producer: KafkaTopicProducer,
    ) -> Self {
        let sample = TemperatureSample {
            cur: start,
            temp_range,
        };
        Experiment {
            stage: ExperimentStage::Uninitialized,
            sample,
            producer,
            config,
        }
    }

    async fn stage_configuration(&mut self) {
        self.stage = ExperimentStage::Configuration;
        let record = RecordData {
            payload: events::experiment_configured_event(
                &self.config.experiment_id,
                &self.config.researcher,
                &self.config.sensors,
                25.5,
                26.5,
            ),
            key: Some(&self.config.experiment_id),
            headers: OwnedHeaders::new().add("record_name", "experiment_configured")
        };
        let delivery_result = self.producer.send_event(record).await;
        println!("Experiment Configured Event {:?}", delivery_result);
    }

    async fn stage_stabilization(&mut self) {
        self.stage = ExperimentStage::Stabilization;
        let record = RecordData {
            payload: events::stabilization_started_event(&self.config.experiment_id),
            key: Some(&self.config.experiment_id),
            headers: OwnedHeaders::new().add("record_name", "stabilization_started")
        };
        let delivery_result = self.producer.send_event(record).await;
        println!("Stabilization Started Event {:?}", delivery_result);

        // Stabilization Temperature Samples
        let stabilization_samples = self.sample.stabilization_samples(2);
        let stabilization_events = events::temperature_events(
            stabilization_samples,
            &self.config.experiment_id,
            &self.config.researcher,
            &self.config.sensors,
            &self.stage,
            &self.config.secret_key,
        );
        for sensor_events in stabilization_events {
            for event in sensor_events {
                let record = RecordData {
                    payload: event,
                    key: Some(&self.config.experiment_id),
                    headers: OwnedHeaders::new().add("record_name", "sensor_temperature_measured")
                };
                let delivery_result = self.producer.send_event(record).await;
                println!("sensor measurement result {:?}", delivery_result);
            }
            println!("Temperature Measured Events");
            time::sleep(Duration::from_millis(self.config.sample_rate)).await;
        }
    }

    async fn stage_carry_out(&mut self) {
        self.stage = ExperimentStage::CarryOut;
        let record = RecordData {
            payload: events::experiment_started_event(&self.config.experiment_id),
            key: Some(&self.config.experiment_id),
            headers: OwnedHeaders::new().add("record_name", "experiment_started")
        };
        let delivery_result = self.producer.send_event(record).await;
        println!("Experiment Started Event {:?}", delivery_result);

        let carry_out_samples = self.sample.carry_out_samples(20);
        let carry_out_events = events::temperature_events(
            carry_out_samples,
            &self.config.experiment_id,
            &self.config.researcher,
            &self.config.sensors,
            &self.stage,
            &self.config.secret_key,
        );
        for sensor_events in carry_out_events {
            for event in sensor_events {
                let record = RecordData {
                    payload: event,
                    key: Some(&self.config.experiment_id),
                    headers: OwnedHeaders::new().add("record_name", "sensor_temperature_measured")
                };
                let delivery_result = self.producer.send_event(record).await;
                println!("sensor measurement result {:?}", delivery_result);
            }
            println!("Temperature Measured Events");
            time::sleep(Duration::from_millis(self.config.sample_rate)).await;
        }

        self.stage = ExperimentStage::Terminated;
        let record = RecordData {
            payload: events::experiment_terminated_event(&self.config.experiment_id),
            key: Some(&self.config.experiment_id),
            headers: OwnedHeaders::new().add("record_name", "experiment_terminated")
        };
        let delivery_result = self.producer.send_event(record).await;
        println!("Experiment Terminated Event {:?}", delivery_result);
    }

    pub async fn run(&mut self) {
        self.stage_configuration().await;
        time::sleep(Duration::from_millis(2000)).await;
        self.stage_stabilization().await;
        self.stage_carry_out().await;
    }
}

pub struct IterMut<'a> {
    sample: &'a mut TemperatureSample,
    delta: f32,
    len: usize,
    iteration: usize,
    random_range: f32,
}

impl<'a> Iterator for IterMut<'a> {
    type Item = TemperatureSample;

    fn next(&mut self) -> Option<Self::Item> {
        if self.iteration >= self.len {
            return None;
        }

        self.sample.cur += self.delta;
        if self.random_range != 0.0 {
            let relative_val = rand::thread_rng().gen_range(-100.0..100.0);
            let absolute_val = relative_val * self.random_range / 100.0;
            self.sample.cur += absolute_val;
        }
        self.iteration += 1;
        Some(*self.sample)
    }
}

pub fn compute_sensor_temperatures(
    sensors: &Vec<String>,
    average_temperature: f32,
) -> Vec<(&'_ str, f32)> {
    let mut cumulative_temperature = 0.0;
    let mut sensor_events = sensors[..sensors.len() - 1]
        .into_iter()
        .map(|sensor_id| {
            let relative_diff = rand::thread_rng().gen_range(-100.0..100.0);
            let sensor_temperature = average_temperature + relative_diff * 1.0 / 100.0;
            cumulative_temperature += sensor_temperature;
            (&**sensor_id, sensor_temperature)
        })
        .collect::<Vec<(&'_ str, f32)>>();
    let last_sensor_id = &sensors[sensors.len() - 1];
    let last_sensor_temperature =
        (sensors.len() as f32) * average_temperature - cumulative_temperature;
    sensor_events.push((last_sensor_id, last_sensor_temperature));
    sensor_events
}
