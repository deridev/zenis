#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct Product {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub price: f64,
    pub amount_of_credits: u64,
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
        let price_per_credit = product.price / product.amount_of_credits as f64;
        println!("{} -> PRICE PER CREDIT: {}", product.id, price_per_credit);
    }
}
