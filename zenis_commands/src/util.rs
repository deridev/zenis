use std::time::Duration;

use zenis_data::products::PRODUCTS;
use zenis_framework::watcher::WatcherOptions;

use crate::prelude::*;

pub fn format_duration(date: chrono::Duration) -> String {
    let mut string = String::with_capacity(64);

    if date.num_days() > 0 {
        string.push_str(&format!("{} dias, ", date.num_days()));
    }

    if date.num_hours() > 0 {
        string.push_str(&format!("{} horas, ", date.num_hours()));
    }

    string.push_str(&format!(
        "{} minutos e {} segundos",
        date.num_minutes() % 60,
        date.num_seconds() % 60
    ));

    string
}

pub fn generate_products_embed() -> EmbedBuilder {
    let mut embed = EmbedBuilder::new_common()
        .set_color(Color::YELLOW)
        .set_author(EmbedAuthor {
            name: "Lista de Produtos".to_string(),
            icon_url: None,
        });

    for product in PRODUCTS.iter() {
        embed = embed.add_inlined_field(
            product.name,
            if product.effective_price() == product.price {
                format!("R$ {:.2?}", product.price)
            } else {
                format!(
                    "*~~`R$ {:.2?}`~~*\n**R$ {:.2?}**!\n({}% de desconto)",
                    product.price,
                    product.effective_price(),
                    (product.discount * 100.0) as i64
                )
            },
        );
    }

    embed
}

pub async fn get_input(
    ctx: &mut CommandContext,
    author: &User,
    message: impl Into<Response>,
) -> anyhow::Result<Option<String>> {
    let author_id = author.id;
    let message = ctx.send(message).await?;

    let Ok(Some(message)) = ctx
        .watcher
        .await_single_message(
            message.channel_id,
            move |message| message.author.id == author_id,
            WatcherOptions {
                timeout: Duration::from_secs(120),
            },
        )
        .await
    else {
        return Ok(None);
    };

    Ok(Some(message.content.trim().to_owned()))
}
