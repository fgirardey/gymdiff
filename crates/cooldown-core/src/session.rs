//! Sessions: runs of games separated by less than an hour (see `docs/Product/Features/History.md`).

use crate::game::{Game, GameSource};

/// A break longer than this starts a new session.
pub const SESSION_BREAK_SECS: i64 = 60 * 60;

/// A run of games, newest first. Never stored: always computed from the games.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    games: Vec<Game>,
}

impl Session {
    /// The session's games, newest first.
    #[must_use]
    pub fn games(&self) -> &[Game] {
        &self.games
    }

    /// Start of the session's first game.
    #[must_use]
    pub fn started_at(&self) -> i64 {
        self.games.last().map_or(0, |game| game.started_at)
    }

    /// End of the session's last game.
    #[must_use]
    pub fn ended_at(&self) -> i64 {
        self.games.first().map_or(0, Game::ended_at)
    }

    /// True when no game was tracked by cooldown.lol: the session has no workouts, so its header is
    /// neutral (no ring, no totals).
    #[must_use]
    pub fn is_before_cooldown(&self) -> bool {
        self.games
            .iter()
            .all(|game| game.source == GameSource::Imported)
    }
}

/// Groups games into sessions, newest session first. The input order does not matter; a game
/// that starts less than [`SESSION_BREAK_SECS`] after the previous one ended joins its session.
#[must_use]
pub fn group_into_sessions(games: &[Game]) -> Vec<Session> {
    let mut oldest_first: Vec<&Game> = games.iter().collect();
    oldest_first.sort_by_key(|game| (game.started_at, game.game_id));

    let mut sessions: Vec<Vec<Game>> = Vec::new();
    for game in oldest_first {
        let joins_last = sessions
            .last()
            .and_then(|session| session.last())
            .is_some_and(|previous| game.started_at - previous.ended_at() < SESSION_BREAK_SECS);
        match sessions.last_mut() {
            Some(session) if joins_last => session.push(game.clone()),
            _ => sessions.push(vec![game.clone()]),
        }
    }

    sessions
        .into_iter()
        .rev()
        .map(|mut games| {
            games.reverse();
            Session { games }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::tests::game;

    const HOUR: i64 = 3_600;

    fn ids(session: &Session) -> Vec<u64> {
        session.games().iter().map(|game| game.game_id).collect()
    }

    #[test]
    fn no_games_no_sessions() {
        assert_eq!(group_into_sessions(&[]), Vec::<Session>::new());
    }

    #[test]
    fn a_short_break_keeps_the_same_session() {
        // Game 1 ends at 1_800; game 2 starts 59 minutes later.
        let games = [game(1, 0, 1_800), game(2, 1_800 + HOUR - 60, 1_800)];
        let sessions = group_into_sessions(&games);
        assert_eq!(sessions.len(), 1);
        assert_eq!(ids(&sessions[0]), [2, 1]);
    }

    #[test]
    fn a_break_of_exactly_one_hour_starts_a_new_session() {
        let games = [game(1, 0, 1_800), game(2, 1_800 + HOUR, 1_800)];
        let sessions = group_into_sessions(&games);
        assert_eq!(sessions.len(), 2);
        assert_eq!(ids(&sessions[0]), [2]);
        assert_eq!(ids(&sessions[1]), [1]);
    }

    #[test]
    fn input_order_does_not_matter_and_sessions_are_newest_first() {
        let games = [
            game(3, 10 * HOUR, 1_800),
            game(1, 0, 1_800),
            game(2, 1_900, 1_800),
        ];
        let sessions = group_into_sessions(&games);
        assert_eq!(sessions.len(), 2);
        assert_eq!(ids(&sessions[0]), [3]);
        assert_eq!(ids(&sessions[1]), [2, 1]);
    }

    #[test]
    fn a_session_knows_when_it_started_and_ended() {
        let games = [game(1, 100, 1_800), game(2, 2_000, 1_000)];
        let session = &group_into_sessions(&games)[0];
        assert_eq!(session.started_at(), 100);
        assert_eq!(session.ended_at(), 3_000);
    }

    #[test]
    fn a_session_is_before_cooldown_until_a_game_is_tracked() {
        let imported = game(1, 0, 1_800);
        let tracked = Game {
            source: GameSource::Tracked,
            ..game(2, 2_000, 1_800)
        };
        assert!(group_into_sessions(std::slice::from_ref(&imported))[0].is_before_cooldown());
        assert!(!group_into_sessions(&[imported, tracked])[0].is_before_cooldown());
    }
}
