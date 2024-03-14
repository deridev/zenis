use std::time::Duration;

use zenis_database::agent_model::{AgentModel, AgentPricing};
use zenis_framework::watcher::WatcherOptions;

use crate::prelude::*;

#[command("Crie um agente personalizado!")]
#[name("criar agente")]
pub async fn create_agent(mut ctx: CommandContext) -> anyhow::Result<()> {
    let author = ctx.author().await?;
    let author_id = author.id;

    let channel = ctx
        .interaction
        .channel
        .clone()
        .context("Expected a channel ID")?;
    let channel_id = channel.id;

    let user_data = ctx.db().users().get_by_user(author_id).await?;
    if user_data.credits < 10 {
        ctx.send(Response::new_user_reply(&author, "você não tem suficientes créditos para criar esse agente! Criar agentes custa 10 créditos.").add_emoji_prefix(emojis::ERROR)).await?;
        return Ok(());
    }

    // Name
    ctx.send(Response::new_user_reply(
        &author,
        "escreva o nome do agente que você quer criar (ex: Monki, Pablo):",
    ))
    .await?;

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
        return Ok(());
    };

    let mut agent_name = message.content.trim().to_owned();
    agent_name = agent_name.replace('\n', " ");

    if agent_name.is_empty() {
        ctx.send(
            Response::new_user_reply(&author, "o nome do agente não pode ser vazio!")
                .add_emoji_prefix(emojis::ERROR),
        )
        .await?;
        return Ok(());
    }

    if !config::NAME_SIZE.contains(&agent_name.len()) {
        ctx.send(
            Response::new_user_reply(
                &author,
                format!(
                    "o nome do agente deve ter no máximo {} caracteres!",
                    config::NAME_SIZE.end()
                ),
            )
            .add_emoji_prefix(emojis::ERROR),
        )
        .await?;
        return Ok(());
    }

    // Description
    ctx.send(Response::new_user_reply(&author, "escreva a descrição da personalidade do agente que você quer criar (ex: Você é Bob, um genial cientista [...]):")).await?;

    let Ok(Some(message)) = ctx
        .watcher
        .await_single_message(
            channel_id,
            move |message| message.author.id == author_id,
            WatcherOptions {
                timeout: Duration::from_secs(512),
            },
        )
        .await
    else {
        return Ok(());
    };

    let agent_description = message.content.trim().to_owned();
    if agent_description.is_empty() {
        ctx.send(
            Response::new_user_reply(&author, "a descrição do agente não pode ser vazia!")
                .add_emoji_prefix(emojis::ERROR),
        )
        .await?;
        return Ok(());
    }

    if !config::DESCRIPTION_SIZE.contains(&agent_description.len()) {
        ctx.send(
            Response::new_user_reply(
                &author,
                format!(
                    "a descrição do agente deve ter no máximo {} caracteres!",
                    config::DESCRIPTION_SIZE.end()
                ),
            )
            .add_emoji_prefix(emojis::ERROR),
        )
        .await?;
        return Ok(());
    }

    // Introduction message
    ctx.send(Response::new_user_reply(&author, "escreva uma mensagem de introdução para o agente que você quer criar (ex: `Olá amigos! Entrei no chat.`):")).await?;

    let Ok(Some(message)) = ctx
        .watcher
        .await_single_message(
            channel_id,
            move |message| message.author.id == author_id,
            WatcherOptions {
                timeout: Duration::from_secs(512),
            },
        )
        .await
    else {
        return Ok(());
    };

    let agent_introduction_message = message.content.trim().to_owned();
    if !config::INTRODUCTION_MESSAGE_SIZE.contains(&agent_introduction_message.len()) {
        ctx.send(
            Response::new_user_reply(
                &author,
                format!(
                    "a introdução do agente deve ter no máximo {} caracteres!",
                    config::INTRODUCTION_MESSAGE_SIZE.end()
                ),
            )
            .add_emoji_prefix(emojis::ERROR),
        )
        .await?;
        return Ok(());
    }

    // Image URL
    ctx.send(Response::new_user_reply(&author, "escreva a URL da imagem (PNG) do agente que você quer criar ou envie um **.** para não ter imagem: (ex: https://imagem/meupersonagem.png)")).await?;

    let Ok(Some(message)) = ctx
        .watcher
        .await_single_message(
            channel_id,
            move |message| message.author.id == author_id,
            WatcherOptions {
                timeout: Duration::from_secs(128),
            },
        )
        .await
    else {
        return Ok(());
    };

    let agent_image_url = message.content.trim().to_owned();
    if agent_image_url != "."
        && ctx
            .client
            .load_url_image(agent_image_url.to_owned())
            .await
            .is_none()
    {
        ctx.send(
            Response::new_user_reply(&author, "URL de imagem inválida!")
                .add_emoji_prefix(emojis::ERROR),
        )
        .await?;
        return Ok(());
    }

    // Invocation price
    ctx.send(Response::new_user_reply(&author, "escreva o preço em créditos por invocar o seu agente (ex: 0, 3, 8): (você receberá 25% desse valor por cada invocação)")).await?;

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
        return Ok(());
    };

    let Some(agent_price) = message.content.trim().parse::<u8>().ok() else {
        ctx.send(
            Response::new_user_reply(&author, "o preço deve ser um número positivo válido!")
                .add_emoji_prefix(emojis::ERROR),
        )
        .await?;
        return Ok(());
    };

    let agent_price = agent_price.clamp(0, 100) as i64;

    let mut name_for_identifier = clear_string(&agent_name);
    name_for_identifier = name_for_identifier.replace(' ', "_");
    name_for_identifier = name_for_identifier.replace('/', "-");

    let identifier = format!("{}/{}", clear_string(&author.name), name_for_identifier);

    if ctx
        .db()
        .agents()
        .get_by_identifier(&identifier)
        .await?
        .is_some()
    {
        ctx.send(Response::new_user_reply(&author, format!("o identificador gerado para o seu agente é **{identifier}**. Mas já existe um agente com esse identificador! Nomeie seu agente com outro nome.")).add_emoji_prefix(emojis::ERROR)).await?;
        return Ok(());
    }

    let mut display_description = agent_description.clone();
    display_description.truncate(60);

    let mut display_introduction_message = agent_introduction_message.clone();
    display_introduction_message.truncate(60);

    let mut confirmation_embed = EmbedBuilder::new_common()
        .set_color(Color::YELLOW)
        .set_author(EmbedAuthor {
            name: format!("Criação do agente {}", agent_name),
            icon_url: Some(author.avatar_url()),
        })
        .add_inlined_field("📄 Nome", format!("**{}**", agent_name))
        .add_inlined_field("😀 Descrição", display_description)
        .add_inlined_field("📢 Mensagem de introdução", display_introduction_message)
        .add_inlined_field(
            format!("{} Preço por invocação", emojis::CREDIT),
            format!("**{}₢**", agent_price),
        )
        .add_footer_text(format!("ID do agente: {}", identifier));

    if agent_image_url != "." {
        confirmation_embed = confirmation_embed.set_thumbnail(&agent_image_url);
    }

    let confirmation = ctx
        .helper()
        .create_confirmation(
            author_id,
            false,
            Response::new_user_reply(
                &author,
                "você quer mesmo criar esse agente? **Criar um agente custa 10₢**.",
            )
            .add_embed(confirmation_embed),
        )
        .await?;
    if !confirmation {
        return Ok(());
    }

    let mut user_data = ctx.db().users().get_by_user(author_id).await?;
    if user_data.credits < 10 {
        ctx.send(
            Response::new_user_reply(
                &author,
                "você não tem suficientes créditos para criar esse agente!",
            )
            .add_emoji_prefix(emojis::ERROR),
        )
        .await?;
        return Ok(());
    }

    user_data.remove_credits(10);
    ctx.db().users().save(user_data).await?;

    let mut agent_model = AgentModel::new(
        author_id.get(),
        &identifier,
        &agent_name,
        &agent_description,
        &agent_introduction_message,
        AgentPricing {
            price_per_invocation: agent_price,
            ..Default::default()
        },
    );

    if let Some(guild_id) = channel.guild_id {
        agent_model = agent_model.with_guild_id(guild_id.get());
    }

    if agent_image_url != "." {
        agent_model = agent_model.with_url_image(agent_image_url);
    }

    ctx.db().agents().create_agent(agent_model).await?;

    ctx.send(
        Response::new_user_reply(
            &author,
            format!(
                "**{}** foi criado com sucesso! Ao invocar ele, use o ID: `{identifier}`\n-> Use **/meus agentes** para ver todos os seus agentes.\n-> Use **/configurar agente** para configurar seu agente.",
                agent_name
            ),
        )
        .add_emoji_prefix(emojis::SUCCESS),
    )
    .await?;

    Ok(())
}
