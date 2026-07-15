use ortalib::{Card, Chips, Edition, Enhancement, Mult, Round};

use crate::poker;

pub fn score_round(round: &Round) -> (Chips, Mult) {
    let evaluated = poker::evaluate(&round.cards_played);
    let (mut chips, mut mult) = evaluated.hand.hand_value();

    // Played cards resolve from left to right. Each scoring card contributes
    // its base rank (unless Stone), then its enhancement, then its edition.
    for &index in &evaluated.scoring_indices {
        score_played_card(&round.cards_played[index], &mut chips, &mut mult);
    }

    // Stage 2 held-card behaviour: only Steel activates while held in hand.
    for card in &round.cards_held_in_hand {
        if card.enhancement == Some(Enhancement::Steel) {
            mult *= 1.5;
        }
    }

    (chips, mult)
}

fn score_played_card(card: &Card, chips: &mut Chips, mult: &mut Mult) {
    if card.enhancement != Some(Enhancement::Stone) {
        *chips += card.rank.rank_value();
    }

    match card.enhancement {
        Some(Enhancement::Bonus) => *chips += 30.0,
        Some(Enhancement::Mult) => *mult += 4.0,
        Some(Enhancement::Glass) => *mult *= 2.0,
        Some(Enhancement::Stone) => *chips += 50.0,
        Some(Enhancement::Wild) | Some(Enhancement::Steel) | None => {}
    }

    apply_edition(card.edition, chips, mult);
}

fn apply_edition(edition: Option<Edition>, chips: &mut Chips, mult: &mut Mult) {
    match edition {
        Some(Edition::Foil) => *chips += 50.0,
        Some(Edition::Holographic) => *mult += 10.0,
        Some(Edition::Polychrome) => *mult *= 1.5,
        None => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ortalib::{Card, Rank::*, Suit::*};

    #[test]
    fn only_pair_cards_score() {
        let round = Round {
            cards_played: vec![
                Card::new(Ace, Hearts, None, None),
                Card::new(Ace, Spades, None, None),
                Card::new(King, Diamonds, None, None),
                Card::new(Nine, Clubs, None, None),
                Card::new(Five, Hearts, None, None),
            ],
            cards_held_in_hand: vec![],
            jokers: vec![],
        };

        assert_eq!(score_round(&round), (32.0, 2.0));
    }

    #[test]
    fn stone_always_scores_without_rank_chips() {
        let round = Round {
            cards_played: vec![
                Card::new(King, Hearts, None, None),
                Card::new(King, Spades, None, None),
                Card::new(Seven, Diamonds, Some(Enhancement::Stone), None),
                Card::new(Jack, Clubs, None, None),
                Card::new(Nine, Hearts, None, None),
            ],
            cards_held_in_hand: vec![],
            jokers: vec![],
        };

        assert_eq!(score_round(&round), (80.0, 2.0));
    }

    #[test]
    fn held_steel_multiplies_mult() {
        let round = Round {
            cards_played: vec![
                Card::new(Ace, Hearts, None, None),
                Card::new(King, Spades, None, None),
                Card::new(Queen, Diamonds, None, None),
                Card::new(Jack, Clubs, None, None),
                Card::new(Ten, Hearts, None, None),
            ],
            cards_held_in_hand: vec![Card::new(King, Spades, Some(Enhancement::Steel), None)],
            jokers: vec![],
        };

        assert_eq!(score_round(&round), (81.0, 6.0));
    }
}
