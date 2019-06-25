use diesel::prelude::*;

use strava::oauth::AccessTokenResponse;

use crate::database::Connection;
use crate::schema::oauth_tokens;
use crate::schema::oauth_tokens::dsl::*;

pub mod commands {
    use crate::{DataError, RequestContext};

    use strava::oauth::RedirectQuery;

    pub fn update_from_strava(
        context: &RequestContext,
        oauth_resp: &RedirectQuery,
    ) -> Result<usize, DataError> {
        let resp = context
            .strava_api()
            .parsed_oauth_response(&oauth_resp)
            .map_err(DataError::StravaError)?;

        context
            .tokens_repo()
            .upsert(&resp)
            .map_err(DataError::QueryError)
    }
}

#[derive(Queryable)]
pub struct OauthToken {
    pub id: i32,
    pub token: String,
    pub refresh: String,
    pub remote_athlete_id: i32,
}

pub struct Repo<'a> {
    pub connection: &'a Connection,
}

impl<'a> Repo<'a> {
    pub fn get(&self) -> QueryResult<OauthToken> {
        oauth_tokens.first(self.connection)
    }

    pub fn upsert(&self, resp: &AccessTokenResponse) -> QueryResult<usize> {
        let new_token = NewOauthToken::from(resp);

        diesel::insert_into(oauth_tokens::table)
            .values(&new_token)
            .on_conflict(remote_athlete_id)
            .do_update()
            .set(&new_token)
            .execute(self.connection)
    }
}

#[derive(AsChangeset, Insertable)]
#[table_name = "oauth_tokens"]
pub struct NewOauthToken {
    pub token: String,
    pub refresh: String,
    pub remote_athlete_id: i32,
}

impl<'a> From<&AccessTokenResponse> for NewOauthToken {
    fn from(resp: &AccessTokenResponse) -> Self {
        Self {
            token: resp.oauth_token(),
            refresh: resp.refresh_token(),
            remote_athlete_id: resp.athlete.id as i32,
        }
    }
}
