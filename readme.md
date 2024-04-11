
# ZenisAI 🧠

![zenis](https://i.imgur.com/5mCHsmI.png)

Zenis é um bot do Discord experimental inspirado no site [character.ai](https://character.ai), com suporte ao modelos **[Claude 3 - Haiku](https://www.anthropic.com/news/claude-3-haiku)** e **[Command-R](https://txt.cohere.com/command-r/)**. Você invoca um agente em um canal de texto e conversa com ele como se fosse um usuário normal.

Originalmente a ideia era ser um SaaS integrado no Discord, portanto o bot possui integrado um sistema de pagamento usando a api [CheckoutPro](https://www.mercadopago.com.br/developers/pt/docs/checkout-pro/landing) do **MercadoPago**. O framework que interage com o MercadoPago é bem simples mas funcional (pelo menos em **11/04/2024** funciona).

# Como usar o bot?

Zenis usa **créditos** como economia: com créditos, você é capaz de usar os comandos do bot e usar os serviços de IA. O principal sendo **/invocar**, um comando que invoca um agente AI no chat atual. O agente vai ler mensagens e responder as mais relevantes.

Há também **/criar-agente** para criar agentes customizados, **/arena** para batalhar com outros usuários via IA e **/tutorial** para ver outros comandos.

# Como funciona?

O projeto deve funcionar por padrão ao rodar ele com `cargo run` e ter as credenciais corretas no ``.env`` (se você não sabe o que é cargo run, é bom aprender [Rust](https://doc.rust-lang.org/book/) antes de tentar rodar ele manualmente)

Zenis usa MongoDB como database, e, por ser um bot do Discord, também precisa de um token de um bot para funcionar. Você precisa de um DB MongoDB e [criar um bot Discord para ter um token](discord.com/developers/applications) para ligar o projeto.

Você provavelmente não vai precisar usar o sistema do MercadoPago para processar pagamentos, mas caso queira usar, pesquise sobre a API do CheckoutPro e pegue as credenciais certas para botar no **.env**. Se você quiser ligar o bot sem o sistema do MercadoPago você vai precisar manualmente editar o código pra remover partes que usem `MercadoPagoClient` ou outras coisas relacionadas ao MP.

Fora isso, também há obviamente as keys do Claude 3 e Command-R, que você precisará ter para o bot funcionar. O bot também possui 3 webhooks do Discord úteis para alguns logs automáticos. Você provavelmente não vai precisar deles, se remover os códigos relacionados à eles tudo funciona normalmente. (Eu recomendo criar os 3 webhooks e botar no .env, é legal ver isso funcionando)


![example](https://i.imgur.com/3PeACZT.png)
![example2](https://i.imgur.com/BZNovIZ.png)
![example3](https://i.imgur.com/iEdbGKZ.png)