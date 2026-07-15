use crate::poker::{self, Rules};
use ortalib::{Card, Chips, Edition, Enhancement, Joker, Mult, Rank, Round, Suit};

pub fn score_round(round: &Round) -> (Chips, Mult) {
    let rules = rules_for(round);
    let evaluated = poker::evaluate(&round.cards_played, rules);
    let (mut chips, mut mult) = evaluated.hand.hand_value();

    score_played_cards(
        round,
        &evaluated.scoring_indices,
        rules,
        &mut chips,
        &mut mult,
    );
    score_held_cards(round, &mut mult);
    score_jokers(round, rules, &mut chips, &mut mult);

    (chips, mult)
}

fn rules_for(round: &Round) -> Rules {
    Rules {
        four_fingers: has(round, Joker::FourFingers),
        shortcut: has(round, Joker::Shortcut),
        smeared: has(round, Joker::SmearedJoker),
        splash: has(round, Joker::Splash),
    }
}

fn score_played_cards(
    round: &Round,
    scoring_indices: &[usize],
    rules: Rules,
    chips: &mut Chips,
    mult: &mut Mult,
) {
    let pareidolia = has(round, Joker::Pareidolia);
    let first_face = scoring_indices
        .iter()
        .copied()
        .find(|&index| poker::is_face(&round.cards_played[index], pareidolia));

    for &index in scoring_indices {
        let card = &round.cards_played[index];
        let trigger_count =
            scored_card_trigger_count(round, card, index, scoring_indices, pareidolia);

        for _ in 0..trigger_count {
            score_card(card, chips, mult);
            for joker in active_jokers(round) {
                on_scored(
                    joker,
                    card,
                    Some(index) == first_face,
                    pareidolia,
                    rules,
                    chips,
                    mult,
                );
            }
        }
    }
}

fn scored_card_trigger_count(
    round: &Round,
    card: &Card,
    index: usize,
    scoring_indices: &[usize],
    pareidolia: bool,
) -> usize {
    let mut trigger_count = 1;
    for joker in active_jokers(round) {
        match joker {
            Joker::SockAndBuskin if poker::is_face(card, pareidolia) => trigger_count += 1,
            Joker::Hack
                if card.enhancement != Some(Enhancement::Stone)
                    && matches!(card.rank, Rank::Two | Rank::Three | Rank::Four | Rank::Five) =>
            {
                trigger_count += 1;
            }
            Joker::HangingChad if scoring_indices.first().copied() == Some(index) => {
                trigger_count += 2;
            }
            _ => {}
        }
    }
    trigger_count
}

fn score_held_cards(round: &Round, mult: &mut Mult) {
    let lowest_rank = round
        .cards_held_in_hand
        .iter()
        .filter(|card| card.enhancement != Some(Enhancement::Stone))
        .map(|card| card.rank)
        .min();
    let mime_count = active_jokers(round)
        .filter(|&joker| joker == Joker::Mime)
        .count();

    for (index, card) in round.cards_held_in_hand.iter().enumerate() {
        for _ in 0..=mime_count {
            score_held_card(round, card, index, lowest_rank, mult);
        }
    }
}

fn score_held_card(
    round: &Round,
    card: &Card,
    index: usize,
    lowest_rank: Option<Rank>,
    mult: &mut Mult,
) {
    if card.enhancement == Some(Enhancement::Steel) {
        *mult *= 1.5;
    }

    for joker in active_jokers(round) {
        match joker {
            Joker::RaisedFist
                if Some(card.rank) == lowest_rank
                    && rightmost_card_with_rank(&round.cards_held_in_hand, card.rank)
                        == Some(index) =>
            {
                *mult += 2.0 * card.rank.rank_value();
            }
            Joker::Baron if card.rank == Rank::King => *mult *= 1.5,
            Joker::ShootTheMoon if card.rank == Rank::Queen => *mult += 13.0,
            _ => {}
        }
    }
}

fn rightmost_card_with_rank(cards: &[Card], rank: Rank) -> Option<usize> {
    cards
        .iter()
        .enumerate()
        .rev()
        .find_map(|(index, card)| (card.rank == rank).then_some(index))
}

fn score_jokers(round: &Round, rules: Rules, chips: &mut Chips, mult: &mut Mult) {
    for (index, joker_card) in round.jokers.iter().enumerate() {
        match joker_card.edition {
            Some(Edition::Foil) => *chips += 50.0,
            Some(Edition::Holographic) => *mult += 10.0,
            _ => {}
        }

        if let Some(joker) = resolve(round, index) {
            independent(joker, round, rules, chips, mult);
        }

        if joker_card.edition == Some(Edition::Polychrome) {
            *mult *= 1.5;
        }
    }
}

fn active_jokers(round: &Round) -> impl Iterator<Item = Joker> + '_ {
    (0..round.jokers.len()).filter_map(|i| resolve(round, i))
}
fn resolve(round: &Round, mut i: usize) -> Option<Joker> {
    let mut copied = false;
    let mut steps = 0;
    loop {
        let j = round.jokers.get(i)?.joker;
        if j != Joker::Blueprint {
            if copied && is_passive(j) {
                return None;
            }
            return Some(j);
        }
        copied = true;
        i += 1;
        steps += 1;
        if steps > round.jokers.len() {
            return None;
        }
    }
}
fn is_passive(j: Joker) -> bool {
    matches!(
        j,
        Joker::FourFingers
            | Joker::Shortcut
            | Joker::Pareidolia
            | Joker::Splash
            | Joker::SmearedJoker
    )
}
fn has(r: &Round, j: Joker) -> bool {
    r.jokers.iter().any(|x| x.joker == j)
}
fn score_card(c: &Card, ch: &mut Chips, m: &mut Mult) {
    if c.enhancement != Some(Enhancement::Stone) {
        *ch += c.rank.rank_value()
    }
    match c.enhancement {
        Some(Enhancement::Bonus) => *ch += 30.0,
        Some(Enhancement::Mult) => *m += 4.0,
        Some(Enhancement::Glass) => *m *= 2.0,
        Some(Enhancement::Stone) => *ch += 50.0,
        _ => {}
    }
    apply_edition(c.edition, ch, m)
}
fn apply_edition(e: Option<Edition>, ch: &mut Chips, m: &mut Mult) {
    match e {
        Some(Edition::Foil) => *ch += 50.0,
        Some(Edition::Holographic) => *m += 10.0,
        Some(Edition::Polychrome) => *m *= 1.5,
        None => {}
    }
}
fn on_scored(
    j: Joker,
    c: &Card,
    first_face: bool,
    pareidolia: bool,
    r: Rules,
    ch: &mut Chips,
    m: &mut Mult,
) {
    use ortalib::Joker::*;
    match j {
        GreedyJoker if poker::suit_match(c, Suit::Diamonds, r.smeared) => *m += 3.0,
        LustyJoker if poker::suit_match(c, Suit::Hearts, r.smeared) => *m += 3.0,
        Arrowhead if poker::suit_match(c, Suit::Spades, r.smeared) => *ch += 50.0,
        OnyxAgate if poker::suit_match(c, Suit::Clubs, r.smeared) => *m += 7.0,
        Fibonacci
            if matches!(
                c.rank,
                Rank::Ace | Rank::Two | Rank::Three | Rank::Five | Rank::Eight
            ) && c.enhancement != Some(Enhancement::Stone) =>
        {
            *m += 8.0
        }
        ScaryFace if poker::is_face(c, pareidolia) => *ch += 30.0,
        EvenSteven
            if matches!(
                c.rank,
                Rank::Two | Rank::Four | Rank::Six | Rank::Eight | Rank::Ten
            ) && c.enhancement != Some(Enhancement::Stone) =>
        {
            *m += 4.0
        }
        OddTodd
            if matches!(
                c.rank,
                Rank::Ace | Rank::Three | Rank::Five | Rank::Seven | Rank::Nine
            ) && c.enhancement != Some(Enhancement::Stone) =>
        {
            *ch += 31.0
        }
        Scholar if c.rank == Rank::Ace && c.enhancement != Some(Enhancement::Stone) => {
            *ch += 20.0;
            *m += 4.0
        }
        WalkieTalkie
            if matches!(c.rank, Rank::Ten | Rank::Four)
                && c.enhancement != Some(Enhancement::Stone) =>
        {
            *ch += 10.0;
            *m += 4.0
        }
        Photograph if first_face => *m *= 2.0,
        SmileyFace if poker::is_face(c, pareidolia) => *m += 5.0,
        _ => {}
    }
}
fn independent(j: Joker, round: &Round, r: Rules, ch: &mut Chips, m: &mut Mult) {
    use ortalib::Joker::*;
    let c = &round.cards_played;
    match j {
        Joker => *m += 4.0,
        JollyJoker if poker::contains_pair(c) => *m += 8.0,
        ZanyJoker if poker::contains_three(c) => *m += 12.0,
        MadJoker if poker::contains_two_pair(c) => *m += 10.0,
        TheOrder if poker::contains_straight(c, r) => *m *= 3.0,
        TheTribe if poker::contains_flush(c, r) => *m *= 2.0,
        SlyJoker if poker::contains_pair(c) => *ch += 50.0,
        WilyJoker if poker::contains_three(c) => *ch += 100.0,
        CleverJoker if poker::contains_two_pair(c) => *ch += 80.0,
        DeviousJoker if poker::contains_straight(c, r) => *ch += 100.0,
        CraftyJoker if poker::contains_flush(c, r) => *ch += 80.0,
        AbstractJoker => *m += 3.0 * round.jokers.len() as f64,
        Blackboard
            if round.cards_held_in_hand.iter().all(|x| {
                poker::suit_match(x, Suit::Spades, r.smeared)
                    || poker::suit_match(x, Suit::Clubs, r.smeared)
            }) =>
        {
            *m *= 3.0
        }
        FlowerPot => {
            let scoring = poker::evaluate(c, r).scoring_indices;
            if flower_pot_matches(c, &scoring, r.smeared) {
                *m *= 3.0
            }
        }
        _ => {}
    }
}

fn flower_pot_matches(cards: &[Card], scoring: &[usize], smeared: bool) -> bool {
    fn assign(
        cards: &[Card],
        scoring: &[usize],
        suits: &[Suit],
        smeared: bool,
        used: &mut [bool],
        pos: usize,
    ) -> bool {
        if pos == suits.len() {
            return true;
        }
        for (slot, &index) in scoring.iter().enumerate() {
            if !used[slot] && poker::suit_match(&cards[index], suits[pos], smeared) {
                used[slot] = true;
                if assign(cards, scoring, suits, smeared, used, pos + 1) {
                    return true;
                }
                used[slot] = false;
            }
        }
        false
    }
    if scoring.len() < 4 {
        return false;
    }
    let suits = [Suit::Diamonds, Suit::Clubs, Suit::Hearts, Suit::Spades];
    assign(
        cards,
        scoring,
        &suits,
        smeared,
        &mut vec![false; scoring.len()],
        0,
    )
}
#[cfg(test)]
mod tests {
    use super::score_round;
    use ortalib::Round;

    fn score_fixture(yaml: &str) -> f64 {
        let round: Round = serde_yaml::from_str(yaml).expect("fixture should be valid YAML");
        let (chips, mult) = score_round(&round);
        (chips * mult).floor()
    }

    macro_rules! scoring_cases {
        ($( $name:ident: ($fixture:literal, $expected:expr) ),+ $(,)?) => {
            $(
                #[test]
                fn $name() {
                    let yaml = include_str!(concat!("../tests/fixtures/", $fixture));
                    assert_eq!(score_fixture(yaml), $expected);
                }
            )+
        };
    }

    scoring_cases! {
        case_01_high_card_jack: ("01_high_card_jack.yml", 15.0),
        case_01_pair_aces: ("01_pair_aces.yml", 64.0),
        case_01_straight_high: ("01_straight_high.yml", 324.0),
        case_01_flush_spades: ("01_flush_spades.yml", 308.0),
        case_01_full_house_medium: ("01_full_house_medium.yml", 360.0),
        case_01_four_of_a_kind_high: ("01_four_of_a_kind_high.yml", 728.0),
        case_02_bonus: ("02_bonus.yml", 444.0),
        case_02_mult: ("02_mult.yml", 648.0),
        case_02_glass: ("02_glass.yml", 648.0),
        case_02_steel: ("02_steel.yml", 486.0),
        case_02_wild_flush: ("02_wild_flush.yml", 1208.0),
        case_02_stone: ("02_stone.yml", 160.0),
        case_03_joker_with_high_card: ("03_joker_with_high_card.yml", 80.0),
        case_03_jolly_joker_with_pair: ("03_jolly_joker_with_pair.yml", 300.0),
        case_03_zany_joker_with_three_of_a_kind: ("03_zany_joker_with_three_of_a_kind.yml", 900.0),
        case_03_mad_joker_with_two_pair: ("03_mad_joker_with_two_pair.yml", 600.0),
        case_03_the_order_with_straight: ("03_the_order_with_straight.yml", 780.0),
        case_03_the_tribe_with_flush: ("03_the_tribe_with_flush.yml", 600.0),
        case_03_abstract_joker_multiple_jokers: ("03_abstract_joker_multiple_jokers.yml", 1377.0),
        case_03_all_editions: ("03_all_editions.yml", 9502.0),
        case_04_raised_fist_low_card: ("04_raised_fist_low_card.yml", 648.0),
        case_04_blackboard_all_black: ("04_blackboard_all_black.yml", 972.0),
        case_04_baron_multiple_kings: ("04_baron_multiple_kings.yml", 54.0),
        case_04_greedy_joker_all_diamonds: ("04_greedy_joker_all_diamonds.yml", 3473.0),
        case_04_fibonacci_all_fibonacci: ("04_fibonacci_all_fibonacci.yml", 144.0),
        case_04_photograph_face_first: ("04_photograph_face_first.yml", 30.0),
        case_04_flower_pot_all_suits: ("04_flower_pot_all_suits.yml", 972.0),
        case_04_walkie_talkie_tens_and_fours: ("04_walkie_talkie_tens_and_fours.yml", 1584.0),
        case_05_four_finger_straight: ("05_four_finger_straight.yml", 284.0),
        case_05_four_finger_flush: ("05_four_finger_flush.yml", 1208.0),
        case_05_shortcut_straight: ("05_shortcut_straight.yml", 308.0),
        case_05_mime: ("05_mime.yml", 729.0),
        case_05_pareidolia: ("05_pareidolia.yml", 46.0),
        case_05_splash: ("05_splash.yml", 55.0),
        case_05_sock_and_buskin: ("05_sock_and_buskin.yml", 1156.0),
        case_05_hack_low_cards: ("05_hack_low_cards.yml", 256.0),
        case_05_hanging_chad: ("05_hanging_chad.yml", 108.0),
        case_05_smeared_flush: ("05_smeared_flush.yml", 1208.0),
        case_05_blueprint_chain: ("05_blueprint_chain.yml", 1620.0),
        case_05_retrigger_chain: ("05_retrigger_chain.yml", 22108.0),
    }
}
