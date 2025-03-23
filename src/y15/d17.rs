use anyhow::Result;
use itertools::Itertools;

type Liters = u32;

const EGGNOG_VOLUME: Liters = 150;

async fn read_containers() -> Result<Vec<Liters>> {
    let input = tokio::fs::read_to_string("inputs/y15_d17.txt").await?;
    input
        .lines()
        .map(|line| line.parse::<Liters>().map_err(anyhow::Error::from))
        .try_collect()
}

fn get_valid_combos(containers: &[Liters]) -> Vec<Vec<&Liters>> {
    (1..=containers.len())
        .flat_map(|len| {
            containers
                .iter()
                .combinations(len)
                .filter(move |combo| combo.iter().copied().sum::<Liters>() == EGGNOG_VOLUME)
        })
        .collect()
}

pub async fn p1() -> Result<()> {
    let containers = read_containers().await?;
    let valid_combos = get_valid_combos(&containers);
    println!("Answer: {}", valid_combos.len());
    Ok(())
}

pub async fn p2() -> Result<()> {
    let containers = read_containers().await?;
    let valid_combos = get_valid_combos(&containers);

    let min_container_count = valid_combos
        .iter()
        .map(|combo| combo.len())
        .min()
        .unwrap_or(0);

    let count = valid_combos
        .into_iter()
        .filter(|combo| combo.len() == min_container_count)
        .count();

    println!("Answer: {}", count);
    Ok(())
}
