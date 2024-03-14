use zenis_common::Color;
use zenis_data::products::PRODUCTS;
use zenis_discord::{EmbedAuthor, EmbedBuilder};

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
