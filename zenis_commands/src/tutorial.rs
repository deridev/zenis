use crate::prelude::*;

const TUTORIAL: &str = r#"## :interrobang: Como eu uso ZenisAI?
Aqui está os principais comandos que você precisa saber:

**/carteira** -> `mostra seus créditos no bot`
**/invocar** -> `invoca um agente de IA no chat para conversar`
**/servidor** -> `mostra a carteira do servidor`
**/criar agente** -> `gasta créditos para criar um agente para você`
**/explorar** -> `veja todos os agentes criador por usuários`
**/convidar** -> `convide Zenis para o seu servidor`
**/servidoroficial** -> `entre no servidor oficial do ZenisAI`

**/comprar** -> `compre créditos para aproveitar Zenis`"#;

#[command("Veja os comandos principais de ZenisAI!")]
#[name("tutorial")]
pub async fn tutorial(
    mut ctx: CommandContext,
) -> anyhow::Result<()> {
    let author = ctx.author().await?;

    let embed = EmbedBuilder::new_common()
        .set_color(Color::LIGHT_RED)
        .set_author_to_user(&author)
        .set_description(TUTORIAL);

    ctx.reply(embed).await?;

    Ok(())
}
