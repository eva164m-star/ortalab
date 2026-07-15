use std::collections::BTreeMap;

use ortalib::{Card, PokerHand, Rank};

#[derive(Debug)]
pub struct EvaluatedHand {
    pub hand: PokerHand,
    pub scoring_indices: Vec<usize>,
}

pub fn evaluate(cards: &[Card]) -> EvaluatedHand {
    assert!(
        !cards.is_empty(),
        "a round must contain at least one played card"
    );

    let rank_groups = group_by_rank(cards);
    let flush = is_flush(cards);
    let straight = is_straight(cards);

    let mut groups: Vec<(Rank, Vec<usize>)> = rank_groups.into_iter().collect();
    groups.sort_by(|(rank_a, indices_a), (rank_b, indices_b)| {
        indices_b
            .len()
            .cmp(&indices_a.len())
            .then_with(|| rank_b.cmp(rank_a))
    });

    let sizes: Vec<usize> = groups.iter().map(|(_, indices)| indices.len()).collect();

    if cards.len() == 5 && sizes.first() == Some(&5) && flush {
        return all_cards(PokerHand::FlushFive, cards);
    }

    if cards.len() == 5 && sizes.as_slice() == [3, 2] && flush {
        return all_cards(PokerHand::FlushHouse, cards);
    }

    if cards.len() == 5 && sizes.first() == Some(&5) {
        return all_cards(PokerHand::FiveOfAKind, cards);
    }

    if cards.len() == 5 && straight && flush {
        return all_cards(PokerHand::StraightFlush, cards);
    }

    if let Some((_, indices)) = groups.iter().find(|(_, indices)| indices.len() == 4) {
        return EvaluatedHand {
            hand: PokerHand::FourOfAKind,
            scoring_indices: indices.clone(),
        };
    }

    if cards.len() == 5 && sizes.as_slice() == [3, 2] {
        return all_cards(PokerHand::FullHouse, cards);
    }

    if cards.len() == 5 && flush {
        return all_cards(PokerHand::Flush, cards);
    }

    if cards.len() == 5 && straight {
        return all_cards(PokerHand::Straight, cards);
    }

    if let Some((_, indices)) = groups.iter().find(|(_, indices)| indices.len() == 3) {
        return EvaluatedHand {
            hand: PokerHand::ThreeOfAKind,
            scoring_indices: indices.clone(),
        };
    }

    let pairs: Vec<&Vec<usize>> = groups
        .iter()
        .filter_map(|(_, indices)| (indices.len() == 2).then_some(indices))
        .collect();

    if pairs.len() >= 2 {
        let mut scoring_indices = pairs
            .iter()
            .take(2)
            .flat_map(|indices| indices.iter().copied())
            .collect::<Vec<_>>();
        scoring_indices.sort_unstable();

        return EvaluatedHand {
            hand: PokerHand::TwoPair,
            scoring_indices,
        };
    }

    if let Some(indices) = pairs.first() {
        return EvaluatedHand {
            hand: PokerHand::Pair,
            scoring_indices: (*indices).clone(),
        };
    }

    let highest_index = cards
        .iter()
        .enumerate()
        .max_by_key(|(_, card)| card.rank)
        .map(|(index, _)| index)
        .expect("cards is known to be non-empty");

    EvaluatedHand {
        hand: PokerHand::HighCard,
        scoring_indices: vec![highest_index],
    }
}

fn all_cards(hand: PokerHand, cards: &[Card]) -> EvaluatedHand {
    EvaluatedHand {
        hand,
        scoring_indices: (0..cards.len()).collect(),
    }
}

fn group_by_rank(cards: &[Card]) -> BTreeMap<Rank, Vec<usize>> {
    let mut groups = BTreeMap::new();
    for (index, card) in cards.iter().enumerate() {
        groups.entry(card.rank).or_insert_with(Vec::new).push(index);
    }
    groups
}

fn is_flush(cards: &[Card]) -> bool {
    let Some(first) = cards.first() else {
        return false;
    };
    cards.iter().all(|card| card.suit == first.suit)
}

fn is_straight(cards: &[Card]) -> bool {
    if cards.len() != 5 {
        return false;
    }

    let mut ranks: Vec<u8> = cards.iter().map(|card| rank_index(card.rank)).collect();
    ranks.sort_unstable();
    ranks.dedup();

    if ranks.len() != 5 {
        return false;
    }

    let ordinary = ranks.windows(2).all(|pair| pair[1] == pair[0] + 1);
    let ace_low = ranks == [0, 1, 2, 3, 12];
    ordinary || ace_low
}

fn rank_index(rank: Rank) -> u8 {
    use Rank::*;
    match rank {
        Two => 0,
        Three => 1,
        Four => 2,
        Five => 3,
        Six => 4,
        Seven => 5,
        Eight => 6,
        Nine => 7,
        Ten => 8,
        Jack => 9,
        Queen => 10,
        King => 11,
        Ace => 12,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ortalib::{Card, Rank::*, Suit, Suit::*};

    fn card(rank: Rank, suit: Suit) -> Card {
        Card::new(rank, suit, None, None)
    }

    #[test]
    fn ace_can_be_low_in_a_straight() {
        let cards = [
            card(Ace, Spades),
            card(Two, Hearts),
            card(Three, Clubs),
            card(Four, Diamonds),
            card(Five, Spades),
        ];
        assert_eq!(evaluate(&cards).hand, PokerHand::Straight);
    }

    #[test]
    fn ace_cannot_wrap_in_a_straight() {
        let cards = [
            card(Queen, Spades),
            card(King, Hearts),
            card(Ace, Clubs),
            card(Two, Diamonds),
            card(Three, Spades),
        ];
        assert_eq!(evaluate(&cards).hand, PokerHand::HighCard);
    }

    #[test]
    fn higher_tier_beats_flush() {
        let cards = [
            card(King, Hearts),
            card(King, Hearts),
            card(King, Hearts),
            card(King, Hearts),
            card(Two, Hearts),
        ];
        assert_eq!(evaluate(&cards).hand, PokerHand::FourOfAKind);
    }
}
