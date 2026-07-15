use crate::poker::{self, Rules};
use ortalib::{Card, Chips, Edition, Enhancement, Joker, Mult, Rank, Round, Suit};

pub fn score_round(round: &Round) -> (Chips, Mult) {
    let rules = Rules {
        four_fingers: has(round, Joker::FourFingers),
        shortcut: has(round, Joker::Shortcut),
        smeared: has(round, Joker::SmearedJoker),
        splash: has(round, Joker::Splash),
    };
    let e = poker::evaluate(&round.cards_played, rules);
    let (mut chips, mut mult) = e.hand.hand_value();
    let pareidolia = has(round, Joker::Pareidolia);
    let first_face = e
        .scoring_indices
        .iter()
        .copied()
        .find(|&i| poker::is_face(&round.cards_played[i], pareidolia));
    for &i in &e.scoring_indices {
        let card = &round.cards_played[i];
        let mut times = 1;
        for j in active_jokers(round) {
            match j {
                Joker::SockAndBuskin if poker::is_face(card, pareidolia) => times += 1,
                Joker::Hack
                    if card.enhancement != Some(Enhancement::Stone)
                        && matches!(
                            card.rank,
                            Rank::Two | Rank::Three | Rank::Four | Rank::Five
                        ) =>
                {
                    times += 1
                }
                Joker::HangingChad if Some(i) == e.scoring_indices.first().copied() => times += 2,
                _ => {}
            }
        }
        for _ in 0..times {
            score_card(card, &mut chips, &mut mult);
            for j in active_jokers(round) {
                on_scored(
                    j,
                    card,
                    Some(i) == first_face,
                    pareidolia,
                    rules,
                    &mut chips,
                    &mut mult,
                )
            }
        }
    }
    let lowest = round
        .cards_held_in_hand
        .iter()
        .enumerate()
        .filter(|(_, c)| c.enhancement != Some(Enhancement::Stone))
        .min_by_key(|(_, c)| c.rank)
        .map(|(_, c)| c.rank);
    for (i, c) in round.cards_held_in_hand.iter().enumerate() {
        let times = 1 + active_jokers(round).filter(|&j| j == Joker::Mime).count();
        for _ in 0..times {
            if c.enhancement == Some(Enhancement::Steel) {
                mult *= 1.5
            }
            for j in active_jokers(round) {
                match j {
                    Joker::RaisedFist
                        if Some(c.rank) == lowest
                            && round
                                .cards_held_in_hand
                                .iter()
                                .enumerate()
                                .filter(|(_, x)| x.rank == c.rank)
                                .map(|x| x.0)
                                .max()
                                == Some(i) =>
                    {
                        mult += 2.0 * c.rank.rank_value()
                    }
                    Joker::Baron if c.rank == Rank::King => mult *= 1.5,
                    Joker::ShootTheMoon if c.rank == Rank::Queen => mult += 13.0,
                    _ => {}
                }
            }
        }
    }
    for (idx, jc) in round.jokers.iter().enumerate() {
        match jc.edition {
            Some(Edition::Foil) => chips += 50.0,
            Some(Edition::Holographic) => mult += 10.0,
            _ => {}
        }
        if let Some(j) = resolve(round, idx) {
            independent(j, round, rules, &mut chips, &mut mult)
        }
        if jc.edition == Some(Edition::Polychrome) {
            mult *= 1.5
        }
    }
    (chips, mult)
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
