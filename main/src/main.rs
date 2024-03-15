mod command_handler;
mod event_handler;

use std::{net::SocketAddr, str::FromStr, sync::Arc, time::Duration};

use chrono::Utc;
pub use event_handler::EventHandler;

use warp::{reply::Response, Filter};
use zenis_ai::{
    common::{ChatMessage, Role},
    util::process_instance_message_queue,
};
use zenis_common::{config, Color};
use zenis_data::products::PRODUCTS;
use zenis_database::{
    bson::oid::ObjectId,
    instance_model::{CreditsPaymentMethod, InstanceModel},
    transaction::CreditDestination,
    DatabaseState, ZenisDatabase,
};
use zenis_discord::{
    twilight_gateway::{
        stream::{self, ShardEventStream},
        Config, Intents,
    },
    twilight_model::id::Id,
    DiscordHttpClient, EmbedBuilder,
};

use tokio_stream::StreamExt;
use zenis_framework::{watcher::Watcher, ZenisClient};
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

    let intents = Intents::GUILD_MESSAGES
        | Intents::MESSAGE_CONTENT
        | Intents::GUILD_MEMBERS
        | Intents::GUILDS;
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
                .and(warp::path::param::<String>())
                .and(warp::post())
                .and(warp::body::json())
                .map(move |transaction_id, payload: serde_json::Value| {
                    (transaction_id, payload, client.clone(), db.clone())
                })
                .and_then(
                    |(transaction_id, payload, client, database): (
                        String,
                        serde_json::Value,
                        Arc<ZenisClient>,
                        Arc<ZenisDatabase>,
                    )| async move {
                        println!(">>>>>>>>>>>>> {:?}", payload);

                        if payload.get("action").is_some() {
                            let notification_payload = match serde_json::from_value::<
                                NotificationPayload,
                            >(
                                payload.clone()
                            ) {
                                Ok(notification_payload) => notification_payload,
                                Err(e) => {
                                    eprintln!("Failed to parse notification payload: {:?}", e);
                                    return Ok::<Response, warp::Rejection>(
                                        WarpResponse::builder()
                                            .status(500)
                                            .body("Failed to parse notification payload".into())
                                            .expect("Building WarpResponse failed"),
                                    );
                                }
                            };

                            if let Err(e) = process_mp_notification(
                                transaction_id,
                                notification_payload.clone(),
                                client.clone(),
                                database.clone(),
                            )
                            .await
                            {
                                eprintln!("[NOTIFICATION ERROR]\n{:?}", e);
                                client
                                    .emit_error_hook(
                                        format!(
                                            "NOTIFICATION PAYLOAD:\n{:?}",
                                            notification_payload
                                        ),
                                        e,
                                    )
                                    .await
                                    .ok();
                            }
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
                let instances = db.instances().all_actives().await.unwrap_or_default();
                for instance in instances {
                    process_instance(http.clone(), client.clone(), db.clone(), instance.clone())
                        .await
                        .ok();

                    let last_message_timestamp = instance.last_received_message_timestamp;
                    let now = chrono::Utc::now();

                    const MINUTES_TOLERANCE: i64 = 8;
                    if (now.timestamp() - last_message_timestamp) > 60 * MINUTES_TOLERANCE {
                        let Ok(Some(mut instance)) = db.instances().get_by_id(instance.id).await
                        else {
                            continue;
                        };

                        instance.exit_reason = Some("Inatividade".to_string());
                        db.instances().save(instance).await.ok();
                    }
                }

                client.delete_off_instances(db.clone()).await.ok();
                db.transactions().delete_expired_transactions().await.ok();
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

async fn process_instance(
    http: Arc<DiscordHttpClient>,
    client: Arc<ZenisClient>,
    database: Arc<ZenisDatabase>,
    mut instance: InstanceModel,
) -> anyhow::Result<()> {
    if instance.history.is_empty() || instance.is_awaiting_new_messages {
        return Ok(());
    }

    let diff = Utc::now().timestamp() - instance.last_sent_message_timestamp;
    if diff < 10 {
        return Ok(());
    }

    instance.is_awaiting_new_messages = true;

    let mut image_processed = false;

    let messages = instance
        .history
        .iter()
        .enumerate()
        .map(|(index, m)| {
            let is_last = index == instance.history.len() - 1;
            ChatMessage {
                role: if m.is_user {
                    Role::User
                } else {
                    Role::Assistant
                },
                content: m.content.clone(),
                image_url: if is_last {
                    if m.image_url.is_some() && !image_processed {
                        image_processed = true;
                    }

                    m.image_url.clone()
                } else {
                    None
                },
            }
        })
        .collect();

    let response = process_instance_message_queue(&mut instance, messages, config::DEBUG).await;

    let response = match response {
        Ok(response) => response,
        Err(e) => {
            client
                .emit_error_hook(
                    format!(
                        "Internal agent error. Agent ID: {}, brain used: {}",
                        instance.agent_identifier,
                        instance.brain.name()
                    ),
                    e,
                )
                .await?;
            instance.exit_reason =
                Some("Erro interno. Desenvolvedor foi contactado sobre o problema.".to_string());
            database.instances().save(instance).await?;
            return Ok(());
        }
    };

    if let Ok(Some(mut agent)) = database
        .agents()
        .get_by_identifier(&instance.agent_identifier)
        .await
    {
        agent.stats.replies += 1;
        database.agents().save(agent).await.ok();
    }

    let response_content = response.message.content.clone();

    if response_content.is_empty() || response_content.contains("{AWAIT}") {
        return Ok(());
    }

    if response_content.contains("{EXIT}") {
        instance.exit_reason = Some("O agente saiu por conta própria".to_string());
        database.instances().save(instance).await?;
        return Ok(());
    }

    let (webhook_id, token) = (instance.webhook_id, instance.webhook_token.clone());

    http.execute_webhook(Id::new(webhook_id), &token)
        .content(&response_content)?
        .await
        .ok();

    process_instance_credits_payment(&mut instance, database.clone(), image_processed).await?;
    database.instances().save(instance).await?;

    Ok(())
}

async fn process_instance_credits_payment(
    instance: &mut InstanceModel,
    database: Arc<ZenisDatabase>,
    image_processed: bool,
) -> anyhow::Result<()> {
    let payment_method = instance.payment_method;
    let price_per_reply = instance.pricing.price_per_reply + if image_processed { 5 } else { 0 };

    match payment_method {
        CreditsPaymentMethod::UserCredits(user_id) => {
            let mut user_data = database.users().get_by_user(Id::new(user_id)).await?;
            user_data.remove_credits(price_per_reply);

            if user_data.credits <= 0 {
                instance.exit_reason =
                    Some("O usuário não tem mais créditos para pagar o agente".to_string());
            }

            database.users().save(user_data).await?;
        }
        CreditsPaymentMethod::GuildPublicCredits(guild_id) => {
            let mut guild_data = database.guilds().get_by_guild(Id::new(guild_id)).await?;
            guild_data.remove_public_credits(price_per_reply);

            if guild_data.public_credits <= 0 {
                instance.exit_reason = Some(
                    "O servidor não tem mais créditos públicos para pagar o agente".to_string(),
                );
            }

            database.guilds().save(guild_data).await?;
        }
    }

    Ok(())
}

pub async fn process_mp_notification(
    transaction_id: String,
    payload: NotificationPayload,
    client: Arc<ZenisClient>,
    database: Arc<ZenisDatabase>,
) -> anyhow::Result<()> {
    let Ok(transaction_id) = ObjectId::from_str(&transaction_id) else {
        client
            .emit_error_hook(
                "Invalid transaction ID".to_string(),
                anyhow::anyhow!("Invalid transaction ID"),
            )
            .await?;
        return Ok(());
    };

    let Some(transaction) = database.transactions().get_by_id(transaction_id).await? else {
        client
            .emit_error_hook(
                format!("Transaction not found with ID: {}", transaction_id),
                anyhow::anyhow!("Transaction not found"),
            )
            .await?;
        return Ok(());
    };

    let transaction_user_id = Id::new(transaction.discord_user_id);

    let payment = match client.mp_client.get_payment(payload.data.id.clone()).await {
        Ok(payment) => payment,
        Err(e) => {
            client
                .emit_error_hook(
                    format!("MP payment not found with ID: {}", payload.data.id),
                    e,
                )
                .await?;
            return Ok(());
        }
    };

    macro_rules! get_dm_channel {
        () => {{
            client
                .http
                .create_private_channel(transaction_user_id)
                .await?
                .model()
                .await?
        }};
    }

    let Some(product) = PRODUCTS
        .iter()
        .find(|product| product.id == transaction.product_id)
    else {
        return Ok(());
    };

    match payment.status.as_str() {
        "approved" => {
            // Continue
        }
        "pending" => {
            let dm_channel = get_dm_channel!();

            let embed = EmbedBuilder::new_common()
                .set_color(Color::YELLOW)
                .set_description("## ⏳ Pagamento pendente! Mantenha sua DM aberta para receber notificações sobre o status do pagamento.")
                .add_footer_text("Algum problema? Contate o suporte do ZenisAI no servidor oficial (/servidoroficial)!");
            client
                .http
                .create_message(dm_channel.id)
                .embeds(&[embed.build()])?
                .await?;
            return Ok(());
        }
        "cancelled" | "rejected" => {
            let dm_channel = get_dm_channel!();

            let embed = EmbedBuilder::new_common()
                .set_color(Color::RED)
                .set_description("## ❌ Pagamento recusado!\n\nO pagamento foi recusado ou cancelado. Tente novamente mais tarde.")
                .add_footer_text("Algum problema? Contate o suporte do ZenisAI no servidor oficial (/servidoroficial)!");

            client
                .http
                .create_message(dm_channel.id)
                .embeds(&[embed.build()])?
                .await?;

            client
                .emit_transaction_hook(false, 0.0, product, transaction_user_id, payment.id)
                .await
                .ok();

            return Ok(());
        }
        _ => {
            return Ok(());
        }
    }

    database
        .transactions()
        .delete_transaction(transaction_id)
        .await?;

    let mut embed = EmbedBuilder::new_common()
        .set_color(Color::GREEN)
        .set_description(format!("## ✅ Pagamento aprovado!\n\nVocê efetuou a compra do produto **{}** no valor de **R$ {:.2?}**.\nAproveite ZenisAI!", product.name, product.effective_price()))
        .add_footer_text("Algum problema? Contate o suporte do ZenisAI no servidor oficial (/servidoroficial)!");

    match transaction.credit_destination {
        CreditDestination::User(user_id) => {
            let mut user_data = database.users().get_by_user(Id::new(user_id)).await?;
            user_data.add_credits(product.amount_of_credits);
            database.users().save(user_data).await?;

            embed = embed.add_inlined_field("Destinatário", format!("Usuário: `{}`", user_id));
        }
        CreditDestination::PublicGuild(guild_id) => {
            let mut guild_data = database.guilds().get_by_guild(Id::new(guild_id)).await?;
            guild_data.add_public_credits(product.amount_of_credits);
            database.guilds().save(guild_data).await?;

            embed = embed.add_inlined_field(
                "Destinatário",
                format!("Créditos Públicos do Servidor: `{}`", guild_id),
            );
        }
        CreditDestination::PrivateGuild(guild_id) => {
            let mut guild_data = database.guilds().get_by_guild(Id::new(guild_id)).await?;
            guild_data.add_credits(product.amount_of_credits);
            database.guilds().save(guild_data).await?;

            embed = embed.add_inlined_field(
                "Destinatário",
                format!("Créditos Privados do Servidor: `{}`", guild_id),
            );
        }
    }

    let dm_channel = get_dm_channel!();

    client
        .http
        .create_message(dm_channel.id)
        .embeds(&[embed.build()])?
        .await?;

    client
        .emit_transaction_hook(
            true,
            payment.transaction_amount as f64,
            product,
            transaction_user_id,
            payment.id,
        )
        .await
        .ok();

    Ok(())
}
