use hdrhistogram::Histogram;

fn main() {
    let mut histogram = Histogram::<u64>::new_with_bounds(1, 1000000, 3).unwrap();
    histogram.record(100).unwrap();
    histogram.record(200).unwrap();
    histogram.record(300).unwrap();

    println!("{:?}", histogram.max());
    println!("{:?}", histogram.min());
    println!("{:?}", histogram.value_at_quantile(0.7));
}
