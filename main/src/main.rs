mod command_handler;
mod event_handler;

use std::{net::SocketAddr, sync::Arc, time::Duration};

use chrono::Utc;
pub use event_handler::EventHandler;

use rand::{rngs::StdRng, Rng, SeedableRng};
use warp::{reply::Response, Filter};
use zenis_ai::PaymentMethod;
use zenis_common::config;
use zenis_database::{DatabaseState, ZenisDatabase};
use zenis_discord::{
    twilight_gateway::{
        stream::{self, ShardEventStream},
        Config, Intents,
    },
    DiscordHttpClient,
};

use tokio_stream::StreamExt;
use zenis_framework::{watcher::Watcher, ZenisAgent, ZenisClient};
use zenis_payment::mp::{client::MercadoPagoClient, notification::NotificationPayload};

use warp::http::Response as WarpResponse;

#[tokio::main]
async fn main() {
    dotenv::dotenv().unwrap();

    let discord_token = std::env::var(if config::DEBUG {
        "DEBUG_DISCORD_TOKEN"
    } else {
        "DISCORD_TOKEN"
    })
    .expect("expected a valid Discord token");

    let intents = Intents::GUILD_MESSAGES | Intents::MESSAGE_CONTENT;
    let config = Config::new(discord_token.clone(), intents);

    let mp_client = MercadoPagoClient::new(
        config::DEBUG,
        if config::DEBUG {
            std::env::var("DEBUG_MERCADO_PAGO_ACCESS_TOKEN").unwrap()
        } else {
            std::env::var("MERCADO_PAGO_ACCESS_TOKEN").unwrap()
        },
    )
    .await
    .unwrap();

    let database = Arc::new(
        ZenisDatabase::new(if config::DEBUG {
            DatabaseState::Debug
        } else {
            DatabaseState::Release
        })
        .await,
    );

    database.setup().await;

    let client = Arc::new(ZenisClient::new(discord_token, Arc::new(mp_client)));
    let watcher = Arc::new(Watcher::new());

    // Load a single shard
    let mut shards =
        stream::create_range(0..1, 1, config, |_, builder| builder.build()).collect::<Vec<_>>();

    let mut stream = ShardEventStream::new(shards.iter_mut());

    // Payment API thread
    {
        let client = client.clone();
        let db = database.clone();
        tokio::spawn(async move {
            let root_route = warp::path::end()
                .map(|| warp::reply::html("This is the API for Zenis AI. Get out of here."));

            let notification_route = warp::path("notifications")
                .and(warp::post())
                .and(warp::body::json())
                .map(move |payload: serde_json::Value| (payload, client.clone(), db.clone()))
                .and_then(
                    |(payload, client, database): (
                        serde_json::Value,
                        Arc<ZenisClient>,
                        Arc<ZenisDatabase>,
                    )| async move {
                        println!("Received notification:\n{:?}", payload);

                        if payload.get("action").is_some() {
                            let Ok(notification_payload) =
                                serde_json::from_value::<NotificationPayload>(payload.clone())
                            else {
                                eprintln!("Failed to parse notification payload: {:?}", payload);
                                return Ok::<Response, warp::Rejection>(
                                    WarpResponse::builder()
                                        .status(500)
                                        .body("Failed to parse notification payload".into())
                                        .expect("Building WarpResponse failed"),
                                );
                            };

                            process_mp_notification(
                                notification_payload,
                                client.clone(),
                                database.clone(),
                            )
                            .await
                            .ok();
                        } else {
                            println!("Received notification is not a payment notification");
                        }

                        Ok(WarpResponse::builder()
                            .status(200)
                            .body("OK".into())
                            .expect("Building WarpResponse failed"))
                    },
                );

            let addr = SocketAddr::from(([0, 0, 0, 0], 80));

            let routes = root_route.or(notification_route);
            warp::serve(routes).run(addr).await;
        });
    }

    // Agent reply thread
    {
        let client = client.clone();
        let db = database.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(10)).await;

                let http = client.http.clone();
                for agent in client.agents.read().await.values().flatten() {
                    process_agent(http.clone(), agent.clone(), db.clone())
                        .await
                        .ok();

                    let last_message_timestamp =
                        agent.1.read().await.last_received_message_timestamp;
                    let now = chrono::Utc::now();

                    const MINUTES_TOLERANCE: i64 = 8;
                    if (now.timestamp() - last_message_timestamp) > 60 * MINUTES_TOLERANCE {
                        agent.1.write().await.exit_reason = Some("Inatividade".to_string());
                    }
                }

                client.delete_inactive_agents().await.unwrap();
            }
        });
    }

    while let Some((_shard, event)) = stream.next().await {
        let event = match event {
            std::result::Result::Ok(event) => event,
            Err(source) => {
                if source.is_fatal() {
                    eprintln!("FATAL ERROR: {:?}", source);
                    break;
                }

                continue;
            }
        };

        let event_handler = EventHandler::new(client.clone(), watcher.clone(), database.clone());
        tokio::spawn(event_handler.handle(event));
    }
}

async fn process_agent(
    http: Arc<DiscordHttpClient>,
    zenis_agent: ZenisAgent,
    database: Arc<ZenisDatabase>,
) -> anyhow::Result<()> {
    let mut agent = zenis_agent.1.write().await;
    if agent.message_queue.is_empty() || agent.awaiting_message {
        return Ok(());
    }

    let diff = Utc::now().timestamp() - agent.last_sent_message_timestamp;
    if diff < 7 {
        println!("It will generate a message in seconds");
        return Ok(());
    }

    agent.awaiting_message = true;

    let response = agent.process_message_queue().await?;
    agent.last_sent_message_timestamp = Utc::now().timestamp();

    let Some(response_content) = response.content.first().cloned() else {
        return Ok(());
    };

    if response_content.text.is_empty() || response_content.text.contains("<AWAIT>") {
        return Ok(());
    }

    if response_content.text.contains("<EXIT>") {
        agent.exit_reason = Some("O agente saiu por conta própria".to_string());
        return Ok(());
    }

    let (webhook_id, token) = (agent.webhook_id, agent.webhook_token.clone());
    http.execute_webhook(webhook_id, &token)
        .content(&response_content.text)?
        .await?;
    agent.last_sent_message_timestamp =
        Utc::now().timestamp() + StdRng::from_entropy().gen_range(1..6);

    drop(agent);
    process_agent_credits_payment(zenis_agent, database).await?;

    Ok(())
}

async fn process_agent_credits_payment(
    agent: ZenisAgent,
    database: Arc<ZenisDatabase>,
) -> anyhow::Result<()> {
    let mut agent = agent.1.write().await;
    let payment_method = agent.agent_payment_method;

    let price_per_reply = agent.agent_pricing.price_per_reply;

    match payment_method {
        PaymentMethod::UserCredits(user_id) => {
            let mut user_data = database.users().get_by_user(user_id).await?;
            user_data.remove_credits(price_per_reply);

            if user_data.credits <= 0 {
                agent.exit_reason =
                    Some("O usuário não tem mais créditos para pagar o agente".to_string());
            }

            database.users().save(user_data).await?;
        }
        PaymentMethod::GuildPublicCredits(guild_id) => {
            let mut guild_data = database.guilds().get_by_guild(guild_id).await?;
            guild_data.remove_public_credits(price_per_reply);

            if guild_data.public_credits <= 0 {
                agent.exit_reason = Some(
                    "O servidor não tem mais créditos públicos para pagar o agente".to_string(),
                );
            }

            database.guilds().save(guild_data).await?;
        }
    }

    Ok(())
}

pub async fn process_mp_notification(
    payload: NotificationPayload,
    client: Arc<ZenisClient>,
    database: Arc<ZenisDatabase>,
) -> Result<(), String> {
    println!("Processing notification: {:?}", payload);
    Ok(())
}
