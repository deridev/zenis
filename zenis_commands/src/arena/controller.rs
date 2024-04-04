use std::time::Duration;

use anyhow::bail;
use rand::{rngs::StdRng, Rng, SeedableRng};
use zenis_ai::{
    common::{ArenaCharacter, ArenaInput, ArenaMessage, ArenaOutput},
    util::get_brain,
};
use zenis_database::instance_model::InstanceBrain;
use zenis_framework::watcher::WatcherOptions;

use crate::prelude::*;

use super::common::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArenaFighter {
    pub user_id: Id<UserMarker>,
    pub user: User,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArenaController {
    pub fighters: Vec<ArenaFighter>,
    pub is_active: bool,
    pub current_fighter_index: usize,
    pub context: String,
    pub history: Vec<ArenaMessage>,
    pub brain: InstanceBrain,
    pub winner: Option<String>,
    pub error_counter: u32,
}

impl ArenaController {
    pub fn current_fighter(&self) -> &ArenaFighter {
        &self.fighters[self.current_fighter_index]
    }

    pub fn generate_embed(&self, short: bool) -> EmbedBuilder {
        let current_fighter = self.current_fighter();

        let mut history = self
            .history
            .iter()
            .filter_map(|m| match m {
                ArenaMessage::Input(input) => Some(format!(
                    "\n### **{}**: `{}`",
                    input.character_name, input.action
                )),
                ArenaMessage::Output(output) => Some(format!(
                    "{} ➡️ **{}**",
                    output.output_message, output.consequences
                )),
                ArenaMessage::Error(..) => None,
            })
            .collect::<Vec<_>>();

        let max_len = if short { 2 } else { 4 };

        while history.len() > max_len {
            history.remove(0);
        }

        let mut context = self.context.clone();
        if !self.history.is_empty() && context.len() > 200 {
            context.truncate(200);
            context.push_str("...");
        }

        EmbedBuilder::new_common()
            .set_color(Color::LIGHT_ORANGE)
            .set_author(EmbedAuthor {
                name: "Arena!".to_string(),
                icon_url: Some(current_fighter.user.avatar_url()),
            })
            .set_description(format!(
                "📄 ```\n{}\n```\n\n{}\n\n### {}",
                self.context,
                history.join("\n"),
                if let Some(winner) = &self.winner {
                    format!("**{}** venceu!", winner)
                } else {
                    format!("**{}**: `<escreva sua ação no chat>`", current_fighter.name)
                }
            ))
    }

    pub async fn generate_output(&mut self) -> anyhow::Result<ArenaOutput> {
        let brain = get_brain(self.brain);
        let characters = self
            .fighters
            .iter()
            .map(|f| ArenaCharacter {
                name: f.name.clone(),
                description: f.description.clone(),
            })
            .collect();

        let output = brain
            .prompt_arena(
                brain.default_parameters(),
                self.context.clone(),
                characters,
                self.history.clone(),
            )
            .await?;

        self.history.push(output.clone());

        while self.history.len() > 6 {
            self.history.remove(0);
        }

        match output {
            ArenaMessage::Output(output) => Ok(output),
            ArenaMessage::Input(_) => unreachable!(),
            ArenaMessage::Error(_) => unreachable!(),
        }
    }

    pub fn next_fighter(&mut self) {
        self.current_fighter_index = (self.current_fighter_index + 1) % self.fighters.len();
    }
}

pub async fn run_arena(
    ctx: &mut CommandContext,
    context: Option<String>,
    payment_method: ArenaPaymentMethod,
    fighters: Vec<ArenaFighter>,
) -> anyhow::Result<()> {
    let brain = InstanceBrain::ClaudeHaiku;

    match payment_method {
        ArenaPaymentMethod::User(user_id) => {
            let mut user_data = ctx.db().users().get_by_user(user_id).await?;
            user_data.credits -= PRICE_PER_ARENA;
            ctx.db().users().save(user_data).await?;
        }
        ArenaPaymentMethod::EveryoneHalf => {
            let cost = PRICE_PER_ARENA / fighters.len() as i64;
            for fighter in fighters.iter() {
                let mut user_data = ctx.db().users().get_by_user(fighter.user_id).await?;
                user_data.credits -= cost;
                ctx.db().users().save(user_data).await?;
            }
        }
    }

    let context = match context {
        Some(context) => context,
        None => {
            let brain = get_brain(brain);

            brain
                .generate_context(
                    fighters
                        .iter()
                        .map(|f| ArenaCharacter {
                            name: f.name.clone(),
                            description: f.description.clone(),
                        })
                        .collect(),
                )
                .await?
        }
    };

    let mut controller = ArenaController {
        current_fighter_index: 0,
        fighters: fighters.clone(),
        is_active: true,
        context,
        brain,
        history: vec![],
        winner: None,
        error_counter: 0,
    };

    let mut rng = StdRng::from_entropy();

    while controller.is_active {
        let input = get_fighter_input(ctx, &controller, controller.current_fighter()).await?;

        controller.history.push(ArenaMessage::Input(ArenaInput {
            character_name: controller.current_fighter().name.clone(),
            action: input.clone(),
            luck: rng.gen_range(0..=100),
        }));

        let output = match controller.generate_output().await {
            Ok(output) => output,
            Err(e) => {
                let history_backup = controller.history.clone();

                controller
                    .history
                    .push(ArenaMessage::Output(ArenaOutput::make_invalid(
                        "INVALID_INPUT_READ_ERROR",
                    )));

                let mut error = e.to_string();
                error.truncate(256);
                controller
                    .history
                    .push(ArenaMessage::Error(format!("{}...", error)));

                let output = controller.generate_output().await;
                match output {
                    Ok(output) => {
                        controller.history = history_backup;
                        controller
                            .history
                            .push(ArenaMessage::Output(output.clone()));
                        output
                    }
                    Err(e) => {
                        bail!("Failed to generate arena output after error. Error: {}", e);
                    }
                }
            }
        };

        if let Some(winner) = output.winner {
            controller.winner = Some(winner);
            ctx.send(
                controller
                    .generate_embed(true)
                    .set_color(Color::GREEN)
                    .set_title("A luta acabou!"),
            )
            .await?;
            controller.is_active = false;
            break;
        }

        controller.next_fighter();

        match payment_method {
            ArenaPaymentMethod::User(user_id) => {
                let mut user_data = ctx.db().users().get_by_user(user_id).await?;
                let user = ctx.client.get_user(user_id).await?;

                if user_data.credits < PRICE_PER_ACTION {
                    ctx.send(
                        Response::new_user_reply(
                            &user,
                            "você não tem suficientes créditos para pagar a arena! Use **/comprar** para adquirir mais créditos e aproveitar ZenisAI!\nA arena foi encerrada por falta de créditos.",
                        )
                        .add_emoji_prefix(emojis::ERROR),
                    )
                    .await?;
                    return Ok(());
                }

                user_data.credits -= PRICE_PER_ACTION;
                ctx.db().users().save(user_data).await?;
            }
            ArenaPaymentMethod::EveryoneHalf => {
                let cost = PRICE_PER_ACTION / fighters.len() as i64;
                for fighter in fighters.iter() {
                    let mut user_data = ctx.db().users().get_by_user(fighter.user_id).await?;
                    let user = ctx.client.get_user(fighter.user_id).await?;

                    if user_data.credits < cost {
                        ctx.send(
                            Response::new_user_reply(
                                &user,
                                "você não tem suficientes créditos para pagar a arena de forma dividida! Use **/comprar** para adquirir mais créditos e aproveitar ZenisAI!\nA arena foi encerrada por falta de créditos.",
                            )
                            .add_emoji_prefix(emojis::ERROR),
                        )
                        .await?;
                        return Ok(());
                    }

                    user_data.credits -= cost;
                    ctx.db().users().save(user_data).await?;
                }
            }
        }
    }

    Ok(())
}

async fn get_fighter_input(
    ctx: &mut CommandContext,
    controller: &ArenaController,
    fighter: &ArenaFighter,
) -> anyhow::Result<String> {
    let embed = controller.generate_embed(false);

    let fighter_user_id = fighter.user_id;
    let message = ctx
        .send(
            Response::new_user_reply(&fighter.user, "escreva a ação do seu personagem:")
                .add_emoji_prefix("🎮")
                .add_embed(embed),
        )
        .await?;

    let Ok(Some(mut message)) = ctx
        .watcher
        .await_single_message(
            message.channel_id,
            move |message| message.author.id == fighter_user_id,
            WatcherOptions {
                timeout: Duration::from_secs(120),
            },
        )
        .await
    else {
        return Ok("Não fazer nada".to_string());
    };

    message.content.truncate(128);

    Ok(if message.content.is_empty() {
        "Não fazer nada".to_string()
    } else {
        message.content.trim().to_owned()
    })
}
