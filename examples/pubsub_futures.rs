use async_std::task;
use futures::{future::FutureExt, stream::StreamExt};
use lapin::{
    options::*, types::FieldTable, BasicProperties, Connection, ConnectionProperties, Error,
};
use log::info;

fn main() -> Result<(), Error> {
    env_logger::init();

    let addr = std::env::var("AMQP_ADDR").unwrap_or_else(|_| "amqp://127.0.0.1:5672/%2f".into());

    task::block_on(async {
        let conn = Connection::connect(&addr, ConnectionProperties::default()).await?;

        info!("CONNECTED");

        let channel_a = conn.create_channel().await?;
        let channel_b = conn.create_channel().await?;

        channel_a
            .queue_declare(
                "hello",
                QueueDeclareOptions::default(),
                FieldTable::default(),
            )
            .await?;
        let queue = channel_b
            .queue_declare(
                "hello",
                QueueDeclareOptions::default(),
                FieldTable::default(),
            )
            .await?;

        let consumer = channel_b
            .clone()
            .basic_consume(
                &queue,
                "my_consumer",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await?;
        let _consumer = task::spawn(async move {
            info!("will consume");
            consumer
                .for_each(move |delivery| {
                    let delivery = delivery.expect("error caught in in consumer");
                    channel_b
                        .basic_ack(delivery.delivery_tag, BasicAckOptions::default())
                        .map(|_| ())
                })
                .await
        });

        let payload = b"Hello world!";

        loop {
            channel_a
                .basic_publish(
                    "",
                    "hello",
                    BasicPublishOptions::default(),
                    payload.to_vec(),
                    BasicProperties::default(),
                )
                .await?;
        }
    })
}
