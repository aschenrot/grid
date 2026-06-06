use grid_generation::{NoiseFillSettings, fill_with_noise};
use grid_prelude::*;
use std::process::ExitCode;

const WIDTH: usize = 16;
const HEIGHT: usize = 16;
const DEFAULT_SEED: u32 = 42;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum SeedSelection {
    Fixed(u32),
    Random,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum CliAction {
    Run(SeedSelection),
    Help,
}

fn main() -> ExitCode {
    match parse_args(std::env::args().skip(1)) {
        Ok(CliAction::Run(selection)) => {
            run(resolve_seed(selection));
            ExitCode::SUCCESS
        }
        Ok(CliAction::Help) => {
            print_help();
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("error: {error}");
            eprintln!("try: cli_dual_grid_demo --help");
            ExitCode::FAILURE
        }
    }
}

fn run(seed: u32) {
    let mut grid = DenseGrid2D::new(WIDTH, HEIGHT, CellType::Empty);
    fill_with_noise(&mut grid, NoiseFillSettings::new(0.1, 0.8, seed));

    println!("Seed: {seed}");
    println!("Logic grid:");
    print_logic_grid(&grid);

    let visual = form_visual_tiles(&grid);
    println!();
    println!("Visual dual grid:");
    print_visual_legend();
    print_visual_grid(&visual);
}

fn resolve_seed(selection: SeedSelection) -> u32 {
    match selection {
        SeedSelection::Fixed(seed) => seed,
        SeedSelection::Random => rand::random(),
    }
}

fn parse_args(args: impl IntoIterator<Item = String>) -> Result<CliAction, String> {
    let args: Vec<_> = args.into_iter().collect();
    match args.as_slice() {
        [] => Ok(CliAction::Run(SeedSelection::Fixed(DEFAULT_SEED))),
        [arg] if arg == "--help" || arg == "-h" => Ok(CliAction::Help),
        [arg] if arg == "--random-seed" => Ok(CliAction::Run(SeedSelection::Random)),
        [arg] if arg.starts_with("--seed=") => parse_seed_argument(&arg["--seed=".len()..]),
        [seed] => parse_seed_value(seed).map(|seed| CliAction::Run(SeedSelection::Fixed(seed))),
        [flag, value] if flag == "--seed" || flag == "-s" => parse_seed_argument(value),
        _ => Err(format!("unsupported arguments: {}", args.join(" "))),
    }
}

fn parse_seed_argument(value: &str) -> Result<CliAction, String> {
    if value.eq_ignore_ascii_case("random") {
        Ok(CliAction::Run(SeedSelection::Random))
    } else {
        parse_seed_value(value).map(|seed| CliAction::Run(SeedSelection::Fixed(seed)))
    }
}

fn parse_seed_value(value: &str) -> Result<u32, String> {
    value
        .parse()
        .map_err(|_| format!("seed must be a u32 or 'random', got '{value}'"))
}

fn print_help() {
    println!("Usage: cli_dual_grid_demo [SEED]");
    println!("       cli_dual_grid_demo --seed <u32|random>");
    println!("       cli_dual_grid_demo --random-seed");
}

fn print_logic_grid(grid: &DenseGrid2D<CellType>) {
    for y in 0..grid.height() {
        for x in 0..grid.width() {
            let coord = coord_from_indices(x, y);
            let symbol = match grid.get(coord) {
                Some(CellType::Wall) => '#',
                _ => '.',
            };
            print!("{symbol} ");
        }
        println!();
    }
}

fn print_visual_grid(grid: &DenseGrid2D<VisualTile>) {
    for y in 0..grid.height() {
        for x in 0..grid.width() {
            let coord = coord_from_indices(x, y);
            let tile = grid.get(coord).copied().unwrap_or_default();
            print!("{tile} ");
        }
        println!();
    }
}

fn print_visual_legend() {
    println!("Legend: . empty, C corner, - edge, D diagonal, T t-junction, F full, ? debug");
}

fn coord_from_indices(x: usize, y: usize) -> GridCoord2 {
    GridCoord2::new(
        i32::try_from(x).expect("demo grid width exceeds i32::MAX"),
        i32::try_from(y).expect("demo grid height exceeds i32::MAX"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<CliAction, String> {
        parse_args(args.iter().map(ToString::to_string))
    }

    #[test]
    fn defaults_to_stable_seed() {
        assert_eq!(
            parse(&[]),
            Ok(CliAction::Run(SeedSelection::Fixed(DEFAULT_SEED)))
        );
    }

    #[test]
    fn accepts_positional_seed_for_backwards_compatibility() {
        assert_eq!(parse(&["7"]), Ok(CliAction::Run(SeedSelection::Fixed(7))));
    }

    #[test]
    fn accepts_explicit_seed_flags() {
        assert_eq!(
            parse(&["--seed", "9"]),
            Ok(CliAction::Run(SeedSelection::Fixed(9)))
        );
        assert_eq!(
            parse(&["--seed=10"]),
            Ok(CliAction::Run(SeedSelection::Fixed(10)))
        );
        assert_eq!(
            parse(&["-s", "11"]),
            Ok(CliAction::Run(SeedSelection::Fixed(11)))
        );
    }

    #[test]
    fn accepts_random_seed_flags() {
        assert_eq!(
            parse(&["--random-seed"]),
            Ok(CliAction::Run(SeedSelection::Random))
        );
        assert_eq!(
            parse(&["--seed", "random"]),
            Ok(CliAction::Run(SeedSelection::Random))
        );
    }

    #[test]
    fn rejects_invalid_seed() {
        assert!(parse(&["--seed", "nope"]).is_err());
    }
}
