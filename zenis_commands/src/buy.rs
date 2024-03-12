use std::time::Duration;

use anyhow::bail;
use zenis_data::products::PRODUCTS;
use zenis_discord::twilight_model::channel::message::component::ButtonStyle;
use zenis_framework::{util::make_multiple_rows, watcher::WatcherOptions};
use zenis_payment::mp::client::CreditDestination;

use crate::{prelude::*, util::generate_products_embed};

#[command("Compre créditos para utilizar os serviços ZenisAI!")]
#[name("comprar")]
pub async fn buy(mut ctx: CommandContext) -> anyhow::Result<()> {
    let author = ctx.author().await?;
    if ctx.interaction.guild_id.is_some() {
        ctx.reply(
            Response::new_user_reply(
                &author,
                "**por motivos de segurança, você só pode usar esse comando nas minhas mensagens diretas.**\nEnvie uma DM para mim e use o comando **/comprar** lá! O processo é rápido e seguro.",
            )
            .add_emoji_prefix("🛡️"),
        )
        .await?;
        return Ok(());
    }

    let destination = get_destination(&mut ctx, &author).await?;

    let products_buttons = PRODUCTS
        .iter()
        .map(|product| {
            ButtonBuilder::new()
                .set_custom_id(product.id)
                .set_label(product.name)
                .set_style(ButtonStyle::Secondary)
        })
        .collect::<Vec<_>>();

    let products_embed = generate_products_embed();

    let message = ctx
        .send(
            Response::new_user_reply(&author, "escolha um produto para comprar:")
                .add_embed(products_embed)
                .set_components(make_multiple_rows(products_buttons.clone())),
        )
        .await?;

    let Ok(Some(interaction)) = ctx
        .watcher
        .await_single_component(
            message.id,
            move |interaction| interaction.author_id() == Some(author.id),
            WatcherOptions {
                timeout: Duration::from_secs(60),
            },
        )
        .await
    else {
        return Ok(());
    };

    let data = interaction.parse_message_component_data()?;

    let products_buttons = products_buttons
        .iter()
        .map(|b| {
            let id = b.data.custom_id.as_ref();
            b.clone()
                .set_disabled(true)
                .set_style(if id == Some(&data.custom_id) {
                    ButtonStyle::Success
                } else {
                    ButtonStyle::Secondary
                })
        })
        .collect::<Vec<_>>();

    let mut ctx = CommandContext::from_with_interaction(&ctx, Box::new(interaction));
    ctx.update_message(Response::default().set_components(make_multiple_rows(products_buttons)))
        .await?;

    let Some(product) = PRODUCTS.iter().find(|product| product.id == data.custom_id) else {
        bail!("Product not found")
    };

    // TODO: almost there 🙋
    let (_, checkout_url) = ctx
        .client
        .create_transaction(author.id, product, destination)
        .await?;

    let payment_embed = EmbedBuilder::new_common()
        .set_color(Color::CYAN_GREEN)
        .set_author(EmbedAuthor {
            name: format!("Compra de {}", author.display_name()),
            icon_url: Some(author.avatar_url()),
        })
        .set_description(format!(
            "### Você está comprando {}!\n\n**Preço:** R$ {}\n\n* *O processo de compra é rápido e seguro.*",
            product.name, product.price
        ))
        .add_footer_text("O botão de pagamento expira em 30 minutos.");

    ctx.send(
        Response::from(payment_embed).set_components(make_multiple_rows(vec![ButtonBuilder::new(
        )
        .set_label("Pagar!")
        .set_url(checkout_url)
        .set_style(ButtonStyle::Link)])),
    )
    .await?;

    Ok(())
}

async fn get_destination(
    ctx: &mut CommandContext,
    author: &User,
) -> anyhow::Result<CreditDestination> {
    let buttons = vec![
        ButtonBuilder::new()
            .set_custom_id("user")
            .set_label("Minha carteira")
            .set_style(ButtonStyle::Secondary),
        ButtonBuilder::new()
            .set_custom_id("guild")
            .set_label("Um servidor")
            .set_style(ButtonStyle::Secondary),
    ];

    let message = ctx
        .send(
            Response::new_user_reply(author, "você quer comprar créditos para **a sua carteira (/carteira)** ou para a carteira de um servidor?")
                .set_components(make_multiple_rows(buttons.clone()))
                .add_emoji_prefix("⁉️"),
        )
        .await?;

    let author_id = author.id;
    let Ok(Some(interaction)) = ctx
        .watcher
        .await_single_component(
            message.id,
            move |interaction| interaction.author_id() == Some(author_id),
            WatcherOptions {
                timeout: Duration::from_secs(60),
            },
        )
        .await
    else {
        bail!("No response.")
    };

    let data = interaction.parse_message_component_data()?;

    let buttons = buttons
        .iter()
        .map(|b| {
            let id = b.data.custom_id.as_ref();
            b.clone()
                .set_disabled(true)
                .set_style(if id == Some(&data.custom_id) {
                    ButtonStyle::Success
                } else {
                    ButtonStyle::Secondary
                })
        })
        .collect::<Vec<_>>();

    let interaction_ctx = CommandContext::from_with_interaction(ctx, Box::new(interaction));
    *ctx = interaction_ctx;

    ctx.update_message(Response::default().set_components(make_multiple_rows(buttons)))
        .await?;

    if data.custom_id == "user" {
        return Ok(CreditDestination::User(author.id));
    }

    // Guild selection
    let buttons = vec![
        ButtonBuilder::new()
            .set_custom_id("public")
            .set_label("Créditos Públicos")
            .set_style(ButtonStyle::Secondary),
        ButtonBuilder::new()
            .set_custom_id("private")
            .set_label("Créditos Privados")
            .set_style(ButtonStyle::Secondary),
    ];

    let message = ctx
        .send(
            Response::new_user_reply(author, "você quer comprar créditos para a **carteira pública** ou **carteira privada** do servidor?\n*\\* A carteira pública todos os membros podem usar. A privada é controlada apenas pelos administradores do servidor.*")
                .set_components(make_multiple_rows(buttons.clone()))
                .add_emoji_prefix("⁉️"),
        )
        .await?;

    let Ok(Some(interaction)) = ctx
        .watcher
        .await_single_component(
            message.id,
            move |interaction| interaction.author_id() == Some(author_id),
            WatcherOptions {
                timeout: Duration::from_secs(60),
            },
        )
        .await
    else {
        bail!("No response.")
    };

    let data = interaction.parse_message_component_data()?;

    let buttons = buttons
        .iter()
        .map(|b| {
            let id = b.data.custom_id.as_ref();
            b.clone()
                .set_disabled(true)
                .set_style(if id == Some(&data.custom_id) {
                    ButtonStyle::Success
                } else {
                    ButtonStyle::Secondary
                })
        })
        .collect::<Vec<_>>();

    let interaction_ctx = CommandContext::from_with_interaction(ctx, Box::new(interaction));
    *ctx = interaction_ctx;

    let channel_id = ctx
        .interaction
        .channel
        .clone()
        .context("Expected a channel ID")?
        .id;

    ctx.update_message(Response::default().set_components(make_multiple_rows(buttons)))
        .await?;

    ctx.send(Response::new_user_reply(author, "escreva o ID do servidor que você quer comprar créditos: ||*(Se você não sabe o ID, use **/servidor** no servidor desejado para ver o ID)*||")).await?;

    let author_id = author.id;
    let Ok(Some(message)) = ctx
        .watcher
        .await_single_message(
            channel_id,
            move |message| message.author.id == author_id,
            WatcherOptions {
                timeout: Duration::from_secs(60),
            },
        )
        .await
    else {
        bail!("No response.")
    };

    let guild_id = message.content.parse::<u64>()?;
    let Some(guild_id) = Id::new_checked(guild_id) else {
        ctx.send_in_channel("ID inválido.").await?;
        bail!("Invalid guild ID")
    };

    if data.custom_id == "public" {
        Ok(CreditDestination::PublicGuild(guild_id))
    } else {
        Ok(CreditDestination::PrivateGuild(guild_id))
    }
}
