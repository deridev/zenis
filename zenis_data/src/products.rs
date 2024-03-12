#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct Product {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub price: f64,
    pub amount_of_credits: i64,
}

const DEFAULT_DESCRIPTION: &str = "Créditos são usados para usar Zenis AI";

pub const PRODUCTS: &[Product] = &[
    Product {
        id: "500_credits",
        name: "500 Créditos",
        description: DEFAULT_DESCRIPTION,
        price: 8.0,
        amount_of_credits: 500,
    },
    Product {
        id: "1000_credits",
        name: "1000 Créditos",
        description: DEFAULT_DESCRIPTION,
        price: 15.0,
        amount_of_credits: 1000,
    },
    Product {
        id: "2000_credits",
        name: "2000 Créditos",
        description: DEFAULT_DESCRIPTION,
        price: 26.0,
        amount_of_credits: 2000,
    },
    Product {
        id: "5000_credits",
        name: "5000 Créditos",
        description: DEFAULT_DESCRIPTION,
        price: 64.0,
        amount_of_credits: 5000,
    },
    Product {
        id: "10000_credits",
        name: "10000 Créditos",
        description: DEFAULT_DESCRIPTION,
        price: 125.0,
        amount_of_credits: 10000,
    },
];

#[test]
fn calculate_price_per_product() {
    for product in PRODUCTS {
        let credit_cost = 0.01;
        let ppc = product.price / product.amount_of_credits as f64;
        let ppm = ppc * 5.0;

        let lpc = ppc - credit_cost;
        let lpm = lpc * 5.0;
        let total_profit = lpc * product.amount_of_credits as f64;
        println!("{}\n   >>> -[PPC: R$ {ppc} ______ PPM: R$ {ppm}]\n   >>> +[LPC: R$ {lpc} ______ LPM: R$ {lpm}]\n   Total profit: R$ {total_profit}\n", product.id);
    }
}
