use rust_decimal::Decimal;

fn main() {
    let decimal = Decimal::try_from(0.1).unwrap();

    let result = decimal + Decimal::try_from(0.2).unwrap();

    println!("{}", result.to_string().parse::<f64>().unwrap());

    println!("{}", 0.1 + 0.2)
}
