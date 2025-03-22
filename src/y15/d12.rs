use anyhow::Result;
use serde_json::Value;

fn sum(value: &Value) -> i64 {
    match value {
        Value::Number(number) => number.as_i64().unwrap(),
        Value::Array(values) => values.iter().map(sum).sum(),
        Value::Object(map) => map.values().map(sum).sum(),
        _ => 0,
    }
}

pub async fn p1() -> Result<()> {
    let input = tokio::fs::read_to_string("inputs/y15_d12.txt").await?;
    let value: Value = serde_json::from_str(&input)?;
    let sum = sum(&value);
    println!("Answer: {}", sum);
    Ok(())
}

fn sum_not_reds(value: &Value) -> i64 {
    match value {
        Value::Number(number) => number.as_i64().unwrap(),
        Value::Array(values) => values.iter().map(sum_not_reds).sum(),
        Value::Object(map) => {
            if map.values().any(|v| v == "red") {
                0
            } else {
                map.values().map(sum_not_reds).sum()
            }
        }
        _ => 0,
    }
}

pub async fn p2() -> Result<()> {
    let input = tokio::fs::read_to_string("inputs/y15_d12.txt").await?;
    let value: Value = serde_json::from_str(&input)?;
    let sum = sum_not_reds(&value);
    println!("Answer: {}", sum);
    Ok(())
}
